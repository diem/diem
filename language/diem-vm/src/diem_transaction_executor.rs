// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    data_cache::StateViewCache,
    diem_transaction_validator::validate_signature_checked_transaction,
    diem_vm::{
        charge_global_write_gas_usage, convert_changeset_and_events, get_transaction_output,
        DiemVMImpl, DiemVMInternals,
    },
    errors::expect_only_successful_execution,
    logging::AdapterLogSchema,
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
    VMExecutor,
};
use diem_logger::prelude::*;
use diem_state_view::StateView;
use diem_trace::prelude::*;
use diem_types::{
    account_config,
    block_metadata::BlockMetadata,
    transaction::{
        ChangeSet, Module, SignatureCheckedTransaction, Transaction, TransactionOutput,
        TransactionPayload, TransactionStatus, WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
    write_set::{WriteSet, WriteSetMut},
};
use fail::fail_point;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::{CostTable, GasAlgebra, GasCarrier, GasUnits},
    identifier::IdentStr,
    transaction_argument::convert_txn_args,
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::{data_cache::RemoteCache, logging::LogContext, session::Session};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    convert::{AsMut, AsRef},
};

pub struct DiemVM(DiemVMImpl);

impl DiemVM {
    pub fn new<S: StateView>(state: &S) -> Self {
        Self(DiemVMImpl::new(state))
    }

    pub fn internals(&self) -> DiemVMInternals {
        DiemVMInternals::new(&self.0)
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
        log_context: &impl LogContext,
    ) -> TransactionOutput {
        self.failed_transaction_cleanup_and_keep_vm_status(
            error_code,
            gas_schedule,
            gas_left,
            txn_data,
            remote_cache,
            account_currency_symbol,
            log_context,
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
        log_context: &impl LogContext,
    ) -> (VMStatus, TransactionOutput) {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        let mut session = self.0.new_session(remote_cache);
        match TransactionStatus::from(error_code.clone()) {
            TransactionStatus::Keep(status) => {
                // The transaction should be charged for gas, so run the epilogue to do that.
                // This is running in a new session that drops any side effects from the
                // attempted transaction (e.g., spending funds that were needed to pay for gas),
                // so even if the previous failure occurred while running the epilogue, it
                // should not fail now. If it somehow fails here, there is no choice but to
                // discard the transaction.
                if let Err(e) = self.0.run_failure_epilogue(
                    &mut session,
                    &mut cost_strategy,
                    txn_data,
                    account_currency_symbol,
                    log_context,
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
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        self.0.run_success_epilogue(
            &mut session,
            &mut cost_strategy,
            txn_data,
            account_currency_symbol,
            log_context,
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

    fn execute_script_or_script_function<R: RemoteCache>(
        &self,
        mut session: Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        payload: &TransactionPayload,
        account_currency_symbol: &IdentStr,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::execute_script_or_script_function", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let gas_schedule = self.0.get_gas_schedule(log_context)?;

        // Run the execution logic
        {
            cost_strategy
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;

            match payload {
                TransactionPayload::Script(script) => session.execute_script(
                    script.code().to_vec(),
                    script.ty_args().to_vec(),
                    convert_txn_args(script.args()),
                    vec![txn_data.sender()],
                    cost_strategy,
                    log_context,
                ),
                TransactionPayload::ScriptFunction(script_fn) => session.execute_script_function(
                    script_fn.module(),
                    script_fn.function(),
                    script_fn.ty_args().to_vec(),
                    convert_txn_args(script_fn.args()),
                    vec![txn_data.sender()],
                    cost_strategy,
                    log_context,
                ),
                TransactionPayload::Module(_) | TransactionPayload::WriteSet(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

            cost_strategy.disable_metering();
            self.success_transaction_cleanup(
                session,
                gas_schedule,
                cost_strategy.remaining_gas(),
                txn_data,
                account_currency_symbol,
                log_context,
            )
        }
    }

    fn execute_module<R: RemoteCache>(
        &self,
        mut session: Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        module: &Module,
        account_currency_symbol: &IdentStr,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::execute_module", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let gas_schedule = self.0.get_gas_schedule(log_context)?;

        // Publish the module
        let module_address = if self.0.publishing_option(log_context)?.is_open_module() {
            txn_data.sender()
        } else {
            account_config::CORE_CODE_ADDRESS
        };

        cost_strategy
            .charge_intrinsic_gas(txn_data.transaction_size())
            .map_err(|e| e.into_vm_status())?;
        session
            .publish_module(
                module.code().to_vec(),
                module_address,
                cost_strategy,
                log_context,
            )
            .map_err(|e| e.into_vm_status())?;

        charge_global_write_gas_usage(cost_strategy, &session, &txn_data.sender())?;

        self.success_transaction_cleanup(
            session,
            gas_schedule,
            cost_strategy.remaining_gas(),
            txn_data,
            account_currency_symbol,
            log_context,
        )
    }

    fn execute_user_transaction(
        &self,
        remote_cache: &StateViewCache<'_>,
        txn: &SignatureCheckedTransaction,
        log_context: &impl LogContext,
    ) -> (VMStatus, TransactionOutput) {
        macro_rules! unwrap_or_discard {
            ($res: expr) => {
                match $res {
                    Ok(s) => s,
                    Err(e) => return discard_error_vm_status(e),
                }
            };
        }

        // Revalidate the transaction.
        let mut session = self.0.new_session(remote_cache);
        let account_currency_symbol = match validate_signature_checked_transaction(
            &self.0,
            &mut session,
            txn,
            remote_cache,
            false,
        ) {
            Ok((_, currency_code)) => currency_code,
            Err(err) => {
                return discard_error_vm_status(err);
            }
        };

        let gas_schedule = unwrap_or_discard!(self.0.get_gas_schedule(log_context));
        let txn_data = TransactionMetadata::new(txn);
        let mut cost_strategy = CostStrategy::transaction(gas_schedule, txn_data.max_gas_amount());

        let result = match txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => self
                .execute_script_or_script_function(
                    session,
                    &mut cost_strategy,
                    &txn_data,
                    payload,
                    &account_currency_symbol,
                    log_context,
                ),
            TransactionPayload::Module(m) => self.execute_module(
                session,
                &mut cost_strategy,
                &txn_data,
                m,
                &account_currency_symbol,
                log_context,
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
                        &account_currency_symbol,
                        log_context,
                    )
                }
            }
        }
    }

    fn execute_writeset(
        &self,
        remote_cache: &StateViewCache<'_>,
        writeset_payload: &WriteSetPayload,
        txn_sender: Option<AccountAddress>,
        log_context: &impl LogContext,
    ) -> Result<ChangeSet, Result<(VMStatus, TransactionOutput), VMStatus>> {
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        Ok(match writeset_payload {
            WriteSetPayload::Direct(change_set) => change_set.clone(),
            WriteSetPayload::Script { script, execute_as } => {
                let mut tmp_session = self.0.new_session(remote_cache);
                let args = convert_txn_args(script.args());
                let senders = match txn_sender {
                    None => vec![*execute_as],
                    Some(sender) => vec![sender, *execute_as],
                };
                let execution_result = tmp_session
                    .execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        args,
                        senders,
                        &mut cost_strategy,
                        log_context,
                    )
                    .and_then(|_| tmp_session.finish())
                    .map_err(|e| e.into_vm_status());
                match execution_result {
                    Ok((changeset, events)) => {
                        let (cs, events) =
                            convert_changeset_and_events(changeset, events).map_err(Err)?;
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
        &self,
        remote_cache: &StateViewCache<'_>,
        writeset_payload: WriteSetPayload,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let change_set =
            match self.execute_writeset(remote_cache, &writeset_payload, None, log_context) {
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
        &self,
        remote_cache: &StateViewCache<'_>,
        block_metadata: BlockMetadata,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::process_block_prologue", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let txn_data = TransactionMetadata {
            sender: account_config::reserved_vm_address(),
            ..Default::default()
        };
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));
        let mut session = self.0.new_session(remote_cache);

        let (round, timestamp, previous_vote, proposer) = block_metadata.into_inner();
        let args = serialize_values(&vec![
            MoveValue::Signer(txn_data.sender),
            MoveValue::U64(round),
            MoveValue::U64(timestamp),
            MoveValue::Vector(previous_vote.into_iter().map(MoveValue::Address).collect()),
            MoveValue::Address(proposer),
        ]);
        session
            .execute_function(
                &DIEM_BLOCK_MODULE,
                &BLOCK_PROLOGUE,
                vec![],
                args,
                &mut cost_strategy,
                log_context,
            )
            .map(|_return_vals| ())
            .or_else(|e| {
                expect_only_successful_execution(e, BLOCK_PROLOGUE.as_str(), log_context)
            })?;
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
        &self,
        remote_cache: &StateViewCache<'_>,
        txn: SignatureCheckedTransaction,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::process_writeset_transaction", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        // Revalidate the transaction.
        let mut session = self.0.new_session(remote_cache);
        if let Err(e) =
            validate_signature_checked_transaction(&self.0, &mut session, &txn, remote_cache, false)
        {
            return Ok(discard_error_vm_status(e));
        };
        self.execute_writeset_transaction(
            remote_cache,
            match txn.payload() {
                TransactionPayload::WriteSet(writeset_payload) => writeset_payload,
                TransactionPayload::Module(_)
                | TransactionPayload::Script(_)
                | TransactionPayload::ScriptFunction(_) => {
                    log_context.alert();
                    error!(*log_context, "[diem_vm] UNREACHABLE");
                    return Ok(discard_error_vm_status(VMStatus::Error(
                        StatusCode::UNREACHABLE,
                    )));
                }
            },
            TransactionMetadata::new(&txn),
            log_context,
        )
    }

    pub fn execute_writeset_transaction(
        &self,
        remote_cache: &StateViewCache<'_>,
        writeset_payload: &WriteSetPayload,
        txn_data: TransactionMetadata,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let change_set = match self.execute_writeset(
            remote_cache,
            writeset_payload,
            Some(txn_data.sender()),
            log_context,
        ) {
            Ok(change_set) => change_set,
            Err(e) => return e,
        };

        // Run the epilogue function.
        let mut session = self.0.new_session(remote_cache);
        self.0.run_writeset_epilogue(
            &mut session,
            &txn_data,
            writeset_payload.should_trigger_reconfiguration_by_default(),
            log_context,
        )?;

        if let Err(e) = self.read_writeset(remote_cache, &change_set.write_set()) {
            // Any error at this point would be an invalid writeset
            return Ok((e, discard_error_output(StatusCode::INVALID_WRITE_SET)));
        };

        let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
        let (epilogue_writeset, epilogue_events) = convert_changeset_and_events(changeset, events)?;

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

    pub(crate) fn execute_block_impl(
        &self,
        transactions: Vec<Transaction>,
        data_cache: &mut StateViewCache,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let count = transactions.len();
        let mut result = vec![];
        let mut current_block_id;
        let mut execute_block_trace_guard = vec![];
        let mut should_restart = false;

        info!(
            AdapterLogSchema::new(data_cache.id(), 0),
            "Executing block, transaction count: {}",
            transactions.len()
        );

        let signature_verified_block: Vec<PreprocessedTransaction>;
        {
            // Verify the signatures of all the transactions in parallel.
            // This is time consuming so don't wait and do the checking
            // sequentially while executing the transactions.
            signature_verified_block = transactions
                .into_par_iter()
                .map(preprocess_transaction)
                .collect();
        }

        for (idx, txn) in signature_verified_block.into_iter().enumerate() {
            let log_context = AdapterLogSchema::new(data_cache.id(), idx);
            if should_restart {
                let txn_output = TransactionOutput::new(
                    WriteSet::default(),
                    vec![],
                    0,
                    TransactionStatus::Retry,
                );
                result.push((VMStatus::Error(StatusCode::UNKNOWN_STATUS), txn_output));
                debug!(log_context, "Retry after reconfiguration");
                continue;
            };
            if let PreprocessedTransaction::BlockMetadata(block_metadata) = &txn {
                execute_block_trace_guard.clear();
                current_block_id = block_metadata.id();
                trace_code_block!("diem_vm::execute_block_impl", {"block", current_block_id}, execute_block_trace_guard);
            };
            let (vm_status, output, sender) =
                self.execute_single_transaction(txn, data_cache, &log_context)?;
            if !output.status().is_discarded() {
                data_cache.push_write_set(output.write_set());
            } else {
                match sender {
                    Some(s) => trace!(
                        log_context,
                        "Transaction discarded, sender: {}, error: {:?}",
                        s,
                        vm_status,
                    ),
                    None => trace!(log_context, "Transaction malformed, error: {:?}", vm_status,),
                }
            }

            if is_reconfiguration(&output) {
                info!(
                    AdapterLogSchema::new(data_cache.id(), 0),
                    "Reconfiguration occurred: restart required",
                );
                should_restart = true;
            }

            // `result` is initially empty, a single element is pushed per loop iteration and
            // the number of iterations is bound to the max size of `signature_verified_block`
            assume!(result.len() < usize::max_value());
            result.push((vm_status, output))
        }

        // Record the histogram count for transactions per block.
        BLOCK_TRANSACTION_COUNT.observe(count as f64);

        Ok(result)
    }

    pub(crate) fn execute_single_transaction(
        &self,
        txn: PreprocessedTransaction,
        data_cache: &StateViewCache<'_>,
        log_context: &impl LogContext,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus> {
        Ok(match txn {
            PreprocessedTransaction::BlockMetadata(block_metadata) => {
                let (vm_status, output) =
                    self.process_block_prologue(data_cache, block_metadata, log_context)?;
                (vm_status, output, Some("block_prologue".to_string()))
            }
            PreprocessedTransaction::WaypointWriteSet(write_set_payload) => {
                let (vm_status, output) =
                    self.process_waypoint_change_set(data_cache, write_set_payload, log_context)?;
                (vm_status, output, Some("waypoint_write_set".to_string()))
            }
            PreprocessedTransaction::UserTransaction(txn) => {
                let sender = txn.sender().to_string();
                let _timer = TXN_TOTAL_SECONDS.start_timer();
                let (vm_status, output) =
                    self.execute_user_transaction(data_cache, &txn, log_context);

                // Increment the counter for user transactions executed.
                let counter_label = match output.status() {
                    TransactionStatus::Keep(_) => Some("success"),
                    TransactionStatus::Discard(_) => Some("discarded"),
                    TransactionStatus::Retry => None,
                };
                if let Some(label) = counter_label {
                    USER_TRANSACTIONS_EXECUTED.with_label_values(&[label]).inc();
                }
                (vm_status, output, Some(sender))
            }
            PreprocessedTransaction::WriteSet(txn) => {
                let (vm_status, output) =
                    self.process_writeset_transaction(data_cache, *txn, log_context)?;
                (vm_status, output, Some("write_set".to_string()))
            }
            PreprocessedTransaction::InvalidSignature => {
                let (vm_status, output) =
                    discard_error_vm_status(VMStatus::Error(StatusCode::INVALID_SIGNATURE));
                (vm_status, output, None)
            }
        })
    }
    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut state_view_cache = StateViewCache::new(state_view);
        let vm = DiemVM::new(&state_view_cache);
        vm.execute_block_impl(transactions, &mut state_view_cache)
    }
}

/// Check the signature (if any) of a transaction. If the signature is OK, the result
/// is a PreprocessedTransaction, where a user transaction is translated to a
/// SignatureCheckedTransaction and also categorized into either a UserTransaction
/// or a WriteSet transaction.
pub(crate) fn preprocess_transaction(txn: Transaction) -> PreprocessedTransaction {
    match txn {
        Transaction::BlockMetadata(b) => PreprocessedTransaction::BlockMetadata(b),
        Transaction::GenesisTransaction(ws) => PreprocessedTransaction::WaypointWriteSet(ws),
        Transaction::UserTransaction(txn) => {
            let checked_txn = if let Ok(checked_txn) = txn.check_signature() {
                checked_txn
            } else {
                return PreprocessedTransaction::InvalidSignature;
            };
            if let TransactionPayload::WriteSet(_) = checked_txn.payload() {
                PreprocessedTransaction::WriteSet(Box::new(checked_txn))
            } else {
                PreprocessedTransaction::UserTransaction(Box::new(checked_txn))
            }
        }
    }
}

fn is_reconfiguration(vm_output: &TransactionOutput) -> bool {
    let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
    vm_output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key)
}

/// Transactions after signature checking:
/// Waypoints and BlockPrologues are not signed and are unaffected by signature checking,
/// but a user transaction or writeset transaction is transformed to a SignatureCheckedTransaction.
#[derive(Debug)]
pub(crate) enum PreprocessedTransaction {
    UserTransaction(Box<SignatureCheckedTransaction>),
    WaypointWriteSet(WriteSetPayload),
    BlockMetadata(BlockMetadata),
    WriteSet(Box<SignatureCheckedTransaction>),
    InvalidSignature,
}

// Executor external API
impl VMExecutor for DiemVM {
    /// Execute a block of `transactions`. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty `WriteSet`. Also `state_view` is immutable, and does not have interior
    /// mutability. Writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        fail_point!("move_adapter::execute_block", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

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

impl AsRef<DiemVMImpl> for DiemVM {
    fn as_ref(&self) -> &DiemVMImpl {
        &self.0
    }
}

impl AsMut<DiemVMImpl> for DiemVM {
    fn as_mut(&mut self) -> &mut DiemVMImpl {
        &mut self.0
    }
}
