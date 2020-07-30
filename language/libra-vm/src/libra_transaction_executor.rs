// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    data_cache::StateViewCache,
    libra_vm::{
        charge_global_write_gas_usage, get_transaction_output,
        txn_effects_to_writeset_and_events_cached, LibraVMImpl, LibraVMInternals,
    },
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
    txn_effects_to_writeset_and_events, VMExecutor,
};
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_trace::prelude::*;
use libra_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        ChangeSet, Module, Script, SignatureCheckedTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus, WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
    write_set::{WriteSet, WriteSetMut},
};
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasCarrier, GasUnits},
    identifier::IdentStr,
};
use move_vm_runtime::{data_cache::RemoteCache, session::Session};

use move_vm_types::{
    gas_schedule::{zero_cost_schedule, CostStrategy},
    values::Value,
};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    convert::{AsMut, AsRef, TryFrom},
};

pub struct LibraVM(LibraVMImpl);

impl LibraVM {
    pub fn new<S: StateView>(state: &S) -> Self {
        Self(LibraVMImpl::new(state))
    }

    pub fn internals(&self) -> LibraVMInternals {
        LibraVMInternals::new(&self.0)
    }

    /// Generates a transaction output for a transaction that encountered errors during the
    /// execution process. This is public for now only for tests.
    pub fn failed_transaction_cleanup(
        &self,
        error_code: VMStatus,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &StateViewCache<'_>,
        account_currency_symbol: &IdentStr,
    ) -> TransactionOutput {
        self.failed_transaction_cleanup_and_keep_vm_status(
            error_code,
            gas_schedule,
            gas_left,
            txn_data,
            remote_cache,
            account_currency_symbol,
        )
        .1
    }

    fn failed_transaction_cleanup_and_keep_vm_status(
        &self,
        error_code: VMStatus,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &StateViewCache<'_>,
        account_currency_symbol: &IdentStr,
    ) -> (VMStatus, TransactionOutput) {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        let mut session = self.0.new_session(remote_cache);
        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                if let Err(e) = self.0.run_failure_epilogue(
                    &mut session,
                    &mut cost_strategy,
                    txn_data,
                    account_currency_symbol,
                ) {
                    return discard_error_vm_status(e);
                }
                let txn_output =
                    get_transaction_output(&mut (), session, &cost_strategy, txn_data, status)
                        .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
            TransactionStatus::Retry => unreachable!(),
        }
    }

    fn success_transaction_cleanup<R: RemoteCache>(
        &self,
        mut session: Session<R>,
        gas_schedule: &CostTable,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        self.0.run_success_epilogue(
            &mut session,
            &mut cost_strategy,
            txn_data,
            account_currency_symbol,
        )?;

        Ok((
            VMStatus::Executed,
            get_transaction_output(
                &mut (),
                session,
                &cost_strategy,
                txn_data,
                KeptVMStatus::Executed,
            )?,
        ))
    }

    fn execute_script(
        &self,
        remote_cache: &StateViewCache<'_>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        script: &Script,
        account_currency_symbol: &IdentStr,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let gas_schedule = self.0.get_gas_schedule()?;
        let mut session = self.0.new_session(remote_cache);

        // Run the validation logic
        {
            cost_strategy.disable_metering();
            self.0.check_gas(txn_data)?;
            self.0.is_allowed_script(script)?;
            self.0.run_prologue(
                &mut session,
                cost_strategy,
                &txn_data,
                account_currency_symbol,
            )?;
        }

        // Run the execution logic
        {
            cost_strategy.enable_metering();
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;
            session
                .execute_script(
                    script.code().to_vec(),
                    script.ty_args().to_vec(),
                    convert_txn_args(script.args()),
                    txn_data.sender(),
                    cost_strategy,
                )
                .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(cost_strategy, &session)?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
                account_currency_symbol,
            )
        }
    }

    fn execute_module(
        &self,
        remote_cache: &StateViewCache<'_>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        module: &Module,
        account_currency_symbol: &IdentStr,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let gas_schedule = self.0.get_gas_schedule()?;
        let mut session = self.0.new_session(remote_cache);

        // Run validation logic
        cost_strategy.disable_metering();
        self.0.check_gas(txn_data)?;
        self.0.is_allowed_module(txn_data, remote_cache)?;
        self.0.run_prologue(
            &mut session,
            cost_strategy,
            txn_data,
            account_currency_symbol,
        )?;

        // Publish the module
        let module_address = if self.0.on_chain_config()?.publishing_option.is_open_module() {
            txn_data.sender()
        } else {
            account_config::CORE_CODE_ADDRESS
        };

        cost_strategy.enable_metering();
        cost_strategy
            .charge_intrinsic_gas(txn_data.transaction_size())
            .map_err(|e| e.into_vm_status())?;
        session
            .publish_module(module.code().to_vec(), module_address, cost_strategy)
            .map_err(|e| e.into_vm_status())?;

        charge_global_write_gas_usage(cost_strategy, &session)?;

        self.success_transaction_cleanup(
            session,
            gas_schedule,
            cost_strategy.remaining_gas(),
            txn_data,
            account_currency_symbol,
        )
    }

    fn execute_user_transaction(
        &mut self,
        remote_cache: &StateViewCache<'_>,
        txn: &SignatureCheckedTransaction,
    ) -> (VMStatus, TransactionOutput) {
        macro_rules! unwrap_or_discard {
            ($res: expr) => {
                match $res {
                    Ok(s) => s,
                    Err(e) => return discard_error_vm_status(e),
                }
            };
        }

        let gas_schedule = unwrap_or_discard!(self.0.get_gas_schedule());
        let txn_data = TransactionMetadata::new(txn);
        let mut cost_strategy = CostStrategy::system(gas_schedule, txn_data.max_gas_amount());
        let account_currency_symbol = unwrap_or_discard!(
            account_config::from_currency_code_string(txn.gas_currency_code())
                .map_err(|_| VMStatus::Error(StatusCode::INVALID_GAS_SPECIFIER))
        );
        let result = match txn.payload() {
            TransactionPayload::Script(s) => self.execute_script(
                remote_cache,
                &mut cost_strategy,
                &txn_data,
                s,
                account_currency_symbol.as_ident_str(),
            ),
            TransactionPayload::Module(m) => self.execute_module(
                remote_cache,
                &mut cost_strategy,
                &txn_data,
                m,
                account_currency_symbol.as_ident_str(),
            ),
            TransactionPayload::WriteSet(_) => {
                return discard_error_vm_status(VMStatus::Error(StatusCode::UNREACHABLE))
            }
        };

        let gas_usage = txn_data
            .max_gas_amount()
            .sub(cost_strategy.remaining_gas())
            .get();
        TXN_GAS_USAGE.observe(gas_usage as f64);

        match result {
            Ok(output) => output,
            Err(err) => {
                let txn_status = TransactionStatus::from(err.clone());
                if txn_status.is_discarded() {
                    discard_error_vm_status(err)
                } else {
                    self.failed_transaction_cleanup_and_keep_vm_status(
                        err,
                        gas_schedule,
                        cost_strategy.remaining_gas(),
                        &txn_data,
                        remote_cache,
                        account_currency_symbol.as_ident_str(),
                    )
                }
            }
        }
    }

    fn execute_writeset(
        &self,
        remote_cache: &StateViewCache<'_>,
        writeset_payload: &WriteSetPayload,
    ) -> Result<ChangeSet, Result<(VMStatus, TransactionOutput), VMStatus>> {
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        Ok(match writeset_payload {
            WriteSetPayload::Direct(change_set) => change_set.clone(),
            WriteSetPayload::Script {
                script,
                execute_as: signer,
            } => {
                let mut tmp_session = self.0.new_session(remote_cache);
                let execution_result = tmp_session
                    .execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        convert_txn_args(script.args()),
                        *signer,
                        &mut cost_strategy,
                    )
                    .and_then(|_| tmp_session.finish())
                    .map_err(|e| e.into_vm_status());
                match execution_result {
                    Ok(effect) => {
                        let (cs, events) =
                            txn_effects_to_writeset_and_events(effect).map_err(Err)?;
                        ChangeSet::new(cs, events)
                    }
                    Err(e) => {
                        return Err(Ok((e, discard_error_output(StatusCode::INVALID_WRITE_SET))))
                    }
                }
            }
        })
    }

    fn read_writeset(
        &self,
        remote_cache: &StateViewCache<'_>,
        write_set: &WriteSet,
    ) -> Result<(), VMStatus> {
        // All Move executions satisfy the read-before-write property. Thus we need to read each
        // access path that the write set is going to update.
        for (ap, _) in write_set.iter() {
            remote_cache
                .get(ap)
                .map_err(|_| VMStatus::Error(StatusCode::STORAGE_ERROR))?;
        }
        Ok(())
    }

    fn process_waypoint_change_set(
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
        writeset_payload: WriteSetPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let change_set = match self.execute_writeset(remote_cache, &writeset_payload) {
            Ok(cs) => cs,
            Err(e) => return e,
        };
        let (write_set, events) = change_set.into_inner();
        self.read_writeset(remote_cache, &write_set)?;
        SYSTEM_TRANSACTIONS_EXECUTED.inc();
        Ok((
            VMStatus::Executed,
            TransactionOutput::new(write_set, events, 0, VMStatus::Executed.into()),
        ))
    }

    fn process_block_prologue(
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
        block_metadata: BlockMetadata,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut txn_data = TransactionMetadata::default();
        txn_data.sender = account_config::reserved_vm_address();

        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));
        let mut session = self.0.new_session(remote_cache);

        if let Ok((round, timestamp, previous_vote, proposer)) = block_metadata.into_inner() {
            let args = vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::u64(round),
                Value::u64(timestamp),
                Value::vector_address(previous_vote),
                Value::address(proposer),
            ];
            session
                .execute_function(
                    &LIBRA_BLOCK_MODULE,
                    &BLOCK_PROLOGUE,
                    vec![],
                    args,
                    txn_data.sender,
                    &mut cost_strategy,
                )
                .map_err(|e| e.into_vm_status())?
        } else {
            return Err(VMStatus::Error(StatusCode::MALFORMED));
        };
        SYSTEM_TRANSACTIONS_EXECUTED.inc();

        let output = get_transaction_output(
            &mut (),
            session,
            &cost_strategy,
            &txn_data,
            KeptVMStatus::Executed,
        )?;
        Ok((VMStatus::Executed, output))
    }

    fn process_writeset_transaction(
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
        txn: SignatureCheckedTransaction,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let txn_data = TransactionMetadata::new(&txn);

        let mut session = self.0.new_session(remote_cache);

        if let Err(e) = self.0.run_writeset_prologue(&mut session, &txn_data) {
            // Switch any error from the prologue to a reject
            return Ok((e, discard_error_output(StatusCode::REJECTED_WRITE_SET)));
        };

        // Bump the sequence number of sender.
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &BUMP_SEQUENCE_NUMBER_NAME,
                vec![],
                vec![Value::transaction_argument_signer_reference(
                    txn_data.sender,
                )],
                txn_data.sender,
                &mut cost_strategy,
            )
            .map_err(|e| e.into_vm_status())?;

        let change_set = match txn.payload() {
            TransactionPayload::WriteSet(writeset_payload) => {
                match self.execute_writeset(remote_cache, writeset_payload) {
                    Ok(change_set) => change_set,
                    Err(e) => return e,
                }
            }
            TransactionPayload::Module(_) | TransactionPayload::Script(_) => {
                error!("[libra_vm] UNREACHABLE");
                return Ok(discard_error_vm_status(VMStatus::Error(
                    StatusCode::UNREACHABLE,
                )));
            }
        };

        // Emit the reconfiguration event
        self.0
            .run_writeset_epilogue(&mut session, &change_set, &txn_data)?;

        if let Err(e) = self.read_writeset(remote_cache, &change_set.write_set()) {
            // Any error at this point would be an invalid writeset
            return Ok((e, discard_error_output(StatusCode::INVALID_WRITE_SET)));
        };

        let effects = session.finish().map_err(|e| e.into_vm_status())?;
        let (epilogue_writeset, epilogue_events) =
            txn_effects_to_writeset_and_events_cached(&mut (), effects)?;

        // Make sure epilogue WriteSet doesn't intersect with the writeset in TransactionPayload.
        if !epilogue_writeset
            .iter()
            .map(|(ap, _)| ap)
            .collect::<HashSet<_>>()
            .is_disjoint(
                &change_set
                    .write_set()
                    .iter()
                    .map(|(ap, _)| ap)
                    .collect::<HashSet<_>>(),
            )
        {
            let vm_status = VMStatus::Error(StatusCode::INVALID_WRITE_SET);
            return Ok(discard_error_vm_status(vm_status));
        }
        if !epilogue_events
            .iter()
            .map(|event| event.key())
            .collect::<HashSet<_>>()
            .is_disjoint(
                &change_set
                    .events()
                    .iter()
                    .map(|event| event.key())
                    .collect::<HashSet<_>>(),
            )
        {
            let vm_status = VMStatus::Error(StatusCode::INVALID_WRITE_SET);
            return Ok(discard_error_vm_status(vm_status));
        }

        let write_set = WriteSetMut::new(
            epilogue_writeset
                .iter()
                .chain(change_set.write_set().iter())
                .cloned()
                .collect(),
        )
        .freeze()
        .map_err(|_| VMStatus::Error(StatusCode::INVALID_WRITE_SET))?;
        let events = change_set
            .events()
            .iter()
            .chain(epilogue_events.iter())
            .cloned()
            .collect();
        SYSTEM_TRANSACTIONS_EXECUTED.inc();

        Ok((
            VMStatus::Executed,
            TransactionOutput::new(
                write_set,
                events,
                0,
                TransactionStatus::Keep(KeptVMStatus::Executed),
            ),
        ))
    }

    fn execute_block_impl(
        &mut self,
        transactions: Vec<Transaction>,
        data_cache: &mut StateViewCache,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let count = transactions.len();
        let mut result = vec![];
        let mut current_block_id;
        let mut execute_block_trace_guard = vec![];
        let mut should_restart = false;

        let signature_verified_block: Vec<Result<PreprocessedTransaction, VMStatus>>;
        {
            signature_verified_block = transactions
                .into_par_iter()
                .map(preprocess_transaction)
                .collect();
        }

        for txn in signature_verified_block {
            if should_restart {
                let txn_output = TransactionOutput::new(
                    WriteSet::default(),
                    vec![],
                    0,
                    TransactionStatus::Retry,
                );
                result.push((VMStatus::Error(StatusCode::UNKNOWN_STATUS), txn_output));
                continue;
            };
            let (vm_status, output) = match txn {
                Ok(PreprocessedTransaction::BlockPrologue(block_metadata)) => {
                    execute_block_trace_guard.clear();
                    current_block_id = block_metadata.id();
                    trace_code_block!("libra_vm::execute_block_impl", {"block", current_block_id}, execute_block_trace_guard);
                    self.process_block_prologue(data_cache, block_metadata)?
                }
                Ok(PreprocessedTransaction::WaypointWriteSet(write_set_payload)) => self
                    .process_waypoint_change_set(data_cache, write_set_payload)
                    .unwrap_or_else(discard_error_vm_status),
                Ok(PreprocessedTransaction::UserTransaction(txn)) => {
                    let _timer = TXN_TOTAL_SECONDS.start_timer();
                    self.execute_user_transaction(data_cache, &txn)
                }
                Ok(PreprocessedTransaction::WriteSet(txn)) => {
                    self.process_writeset_transaction(data_cache, *txn)?
                }
                Err(e) => discard_error_vm_status(e),
            };
            if !output.status().is_discarded() {
                data_cache.push_write_set(output.write_set());
            }

            if is_reconfiguration(&output) {
                should_restart = true;
            }

            // Increment the counter for transactions executed.
            let counter_label = match output.status() {
                TransactionStatus::Keep(_) => Some("success"),
                TransactionStatus::Discard(_) => Some("discarded"),
                TransactionStatus::Retry => None,
            };
            if let Some(label) = counter_label {
                USER_TRANSACTIONS_EXECUTED.with_label_values(&[label]).inc();
            }

            // `result` is initially empty, a single element is pushed per loop iteration and
            // the number of iterations is bound to the max size of `signature_verified_block`
            assume!(result.len() < usize::max_value());
            result.push((vm_status, output))
        }

        // Record the histogram count for transactions per block.
        match i64::try_from(count) {
            Ok(val) => BLOCK_TRANSACTION_COUNT.set(val),
            Err(_) => BLOCK_TRANSACTION_COUNT.set(std::i64::MAX),
        }

        Ok(result)
    }

    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut state_view_cache = StateViewCache::new(state_view);
        let mut vm = LibraVM::new(&state_view_cache);
        vm.execute_block_impl(transactions, &mut state_view_cache)
    }
}

fn preprocess_transaction(txn: Transaction) -> Result<PreprocessedTransaction, VMStatus> {
    Ok(match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockPrologue(b),
        Transaction::GenesisTransaction(ws) => PreprocessedTransaction::WaypointWriteSet(ws),
        Transaction::UserTransaction(txn) => {
            let checked_txn = txn
                .check_signature()
                .map_err(|_| VMStatus::Error(StatusCode::INVALID_SIGNATURE))?;
            if let TransactionPayload::WriteSet(_) = checked_txn.payload() {
                PreprocessedTransaction::WriteSet(Box::new(checked_txn))
            } else {
                PreprocessedTransaction::UserTransaction(Box::new(checked_txn))
            }
        }
    })
}

fn is_reconfiguration(vm_output: &TransactionOutput) -> bool {
    let new_epoch_event_key = libra_types::on_chain_config::new_epoch_event_key();
    vm_output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key)
}

/// Transactions divided by transaction flow.
/// Transaction flows are different across different types of transactions.
enum PreprocessedTransaction {
    UserTransaction(Box<SignatureCheckedTransaction>),
    WaypointWriteSet(WriteSetPayload),
    BlockPrologue(BlockMetadata),
    WriteSet(Box<SignatureCheckedTransaction>),
}

// Executor external API
impl VMExecutor for LibraVM {
    /// Execute a block of `transactions`. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty `WriteSet`. Also `state_view` is immutable, and does not have interior
    /// mutability. Writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        let output = Self::execute_block_and_keep_vm_status(transactions, state_view)?;
        Ok(output
            .into_iter()
            .map(|(_vm_status, txn_output)| txn_output)
            .collect())
    }
}

pub(crate) fn discard_error_vm_status(err: VMStatus) -> (VMStatus, TransactionOutput) {
    let vm_status = err.clone();
    let error_code = match err.keep_or_discard() {
        Ok(_) => {
            debug_assert!(false, "discarding non-discardable error: {:?}", vm_status);
            vm_status.status_code()
        }
        Err(code) => code,
    };
    (vm_status, discard_error_output(error_code))
}

pub(crate) fn discard_error_output(err: StatusCode) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

impl AsRef<LibraVMImpl> for LibraVM {
    fn as_ref(&self) -> &LibraVMImpl {
        &self.0
    }
}

impl AsMut<LibraVMImpl> for LibraVM {
    fn as_mut(&mut self) -> &mut LibraVMImpl {
        &mut self.0
    }
}
