// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    adapter_common,
    adapter_common::{
        discard_error_output, discard_error_vm_status, validate_signature_checked_transaction,
        validate_signed_transaction, PreprocessedTransaction, VMAdapter,
    },
    counters::*,
    data_cache::{RemoteStorage, StateViewCache},
    diem_vm_impl::{
        charge_global_write_gas_usage, convert_changeset_and_events, get_currency_info,
        get_gas_currency_code, get_transaction_output, DiemVMImpl, DiemVMInternals,
    },
    errors::expect_only_successful_execution,
    logging::AdapterLogSchema,
    script_to_script_function,
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
    VMExecutor, VMValidator,
};
use anyhow::Result;
use diem_logger::prelude::*;
use diem_state_view::StateView;
use diem_types::{
    account_config,
    block_metadata::BlockMetadata,
    on_chain_config::{
        DiemVersion, OnChainConfig, ParallelExecutionConfig, VMConfig, VMPublishingOption,
        DIEM_VERSION_2, DIEM_VERSION_3,
    },
    transaction::{
        ChangeSet, ModuleBundle, SignatureCheckedTransaction, SignedTransaction, Transaction,
        TransactionOutput, TransactionPayload, TransactionStatus, VMValidatorResult,
        WriteSetPayload,
    },
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
    write_set::{WriteSet, WriteSetMut},
};
use fail::fail_point;
use move_core_types::{
    account_address::AccountAddress,
    gas_schedule::GasAlgebra,
    identifier::IdentStr,
    resolver::MoveResolver,
    transaction_argument::convert_txn_args,
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::session::Session;
use move_vm_types::gas_schedule::GasStatus;
use read_write_set_dynamic::NormalizedReadWriteSetAnalysis;
use std::{
    collections::HashSet,
    convert::{AsMut, AsRef},
};

#[derive(Clone)]
pub struct DiemVM(pub(crate) DiemVMImpl);

impl DiemVM {
    pub fn new<S: StateView>(state: &S) -> Self {
        Self(DiemVMImpl::new(state))
    }

    pub fn new_for_validation<S: StateView>(state: &S) -> Self {
        info!(
            AdapterLogSchema::new(state.id(), 0),
            "Adapter created for Validation"
        );
        Self::new(state)
    }

    pub fn init_with_config(
        version: DiemVersion,
        on_chain_config: VMConfig,
        publishing_option: VMPublishingOption,
    ) -> Self {
        info!("Adapter restarted for Validation");
        DiemVM(DiemVMImpl::init_with_config(
            version,
            on_chain_config,
            publishing_option,
        ))
    }
    pub fn internals(&self) -> DiemVMInternals {
        DiemVMInternals::new(&self.0)
    }

    /// Generates a transaction output for a transaction that encountered errors during the
    /// execution process. This is public for now only for tests.
    pub fn failed_transaction_cleanup<S: MoveResolver>(
        &self,
        error_code: VMStatus,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        storage: &S,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> TransactionOutput {
        self.failed_transaction_cleanup_and_keep_vm_status(
            error_code,
            gas_status,
            txn_data,
            storage,
            account_currency_symbol,
            log_context,
        )
        .1
    }

    fn failed_transaction_cleanup_and_keep_vm_status<S: MoveResolver>(
        &self,
        error_code: VMStatus,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        storage: &S,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> (VMStatus, TransactionOutput) {
        gas_status.set_metering(false);
        let mut session = self.0.new_session(storage);
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
                    gas_status,
                    txn_data,
                    account_currency_symbol,
                    log_context,
                ) {
                    return discard_error_vm_status(e);
                }
                let txn_output = get_transaction_output(
                    &mut (),
                    session,
                    gas_status.remaining_gas(),
                    txn_data,
                    status,
                )
                .unwrap_or_else(|e| discard_error_vm_status(e).1);
                (error_code, txn_output)
            }
            TransactionStatus::Discard(status) => {
                (VMStatus::Error(status), discard_error_output(status))
            }
            TransactionStatus::Retry => unreachable!(),
        }
    }

    fn success_transaction_cleanup<S: MoveResolver>(
        &self,
        mut session: Session<S>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        gas_status.set_metering(false);
        self.0.run_success_epilogue(
            &mut session,
            gas_status,
            txn_data,
            account_currency_symbol,
            log_context,
        )?;

        Ok((
            VMStatus::Executed,
            get_transaction_output(
                &mut (),
                session,
                gas_status.remaining_gas(),
                txn_data,
                KeptVMStatus::Executed,
            )?,
        ))
    }

    fn execute_script_or_script_function<S: MoveResolver>(
        &self,
        mut session: Session<S>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        payload: &TransactionPayload,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::execute_script_or_script_function", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        // Run the execution logic
        {
            gas_status
                .charge_intrinsic_gas(txn_data.transaction_size())
                .map_err(|e| e.into_vm_status())?;

            match payload {
                TransactionPayload::Script(script) => {
                    let diem_version = self.0.get_diem_version()?;
                    let remapped_script =
                        if diem_version < diem_types::on_chain_config::DIEM_VERSION_2 {
                            None
                        } else {
                            script_to_script_function::remapping(script.code())
                        };
                    let mut senders = vec![txn_data.sender()];
                    if diem_version >= DIEM_VERSION_3 {
                        senders.extend(txn_data.secondary_signers());
                    }
                    match remapped_script {
                        // We are in this case before VERSION_2
                        // or if there is no remapping for the script
                        None => session.execute_script(
                            script.code().to_vec(),
                            script.ty_args().to_vec(),
                            convert_txn_args(script.args()),
                            senders,
                            gas_status,
                        ),
                        Some((module, function)) => session.execute_script_function(
                            module,
                            function,
                            script.ty_args().to_vec(),
                            convert_txn_args(script.args()),
                            senders,
                            gas_status,
                        ),
                    }
                }
                TransactionPayload::ScriptFunction(script_fn) => {
                    let diem_version = self.0.get_diem_version()?;
                    let mut senders = vec![txn_data.sender()];
                    if diem_version >= DIEM_VERSION_3 {
                        senders.extend(txn_data.secondary_signers());
                    }
                    session.execute_script_function(
                        script_fn.module(),
                        script_fn.function(),
                        script_fn.ty_args().to_vec(),
                        script_fn.args().to_vec(),
                        senders,
                        gas_status,
                    )
                }
                TransactionPayload::ModuleBundle(_) | TransactionPayload::WriteSet(_) => {
                    return Err(VMStatus::Error(StatusCode::UNREACHABLE));
                }
            }
            .map_err(|e| e.into_vm_status())?;

            charge_global_write_gas_usage(gas_status, &session, &txn_data.sender())?;

            self.success_transaction_cleanup(
                session,
                gas_status,
                txn_data,
                account_currency_symbol,
                log_context,
            )
        }
    }

    fn execute_modules<S: MoveResolver>(
        &self,
        mut session: Session<S>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        modules: &ModuleBundle,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::execute_module", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        // Publish the module
        let module_address = if self.0.publishing_option(log_context)?.is_open_module() {
            txn_data.sender()
        } else {
            account_config::CORE_CODE_ADDRESS
        };

        gas_status
            .charge_intrinsic_gas(txn_data.transaction_size())
            .map_err(|e| e.into_vm_status())?;

        session
            .publish_module_bundle(modules.clone().into_inner(), module_address, gas_status)
            .map_err(|e| e.into_vm_status())?;

        charge_global_write_gas_usage(gas_status, &session, &txn_data.sender())?;

        self.success_transaction_cleanup(
            session,
            gas_status,
            txn_data,
            account_currency_symbol,
            log_context,
        )
    }

    pub(crate) fn execute_user_transaction<S: MoveResolver>(
        &self,
        storage: &S,
        txn: &SignatureCheckedTransaction,
        log_context: &AdapterLogSchema,
    ) -> (VMStatus, TransactionOutput) {
        macro_rules! unwrap_or_discard {
            ($res: expr) => {
                match $res {
                    Ok(s) => s,
                    Err(e) => return discard_error_vm_status(e),
                }
            };
        }

        let account_currency_symbol = match get_gas_currency_code(txn) {
            Ok(symbol) => symbol,
            Err(err) => {
                return discard_error_vm_status(err);
            }
        };

        if let Err(err) = get_currency_info(&account_currency_symbol, storage) {
            return discard_error_vm_status(err);
        }

        // Revalidate the transaction.
        let mut session = self.0.new_session(storage);
        if let Err(err) = validate_signature_checked_transaction::<S, Self>(
            self,
            &mut session,
            txn,
            false,
            log_context,
        ) {
            return discard_error_vm_status(err);
        };

        let gas_schedule = unwrap_or_discard!(self.0.get_gas_schedule(log_context));
        let txn_data = TransactionMetadata::new(txn);
        let mut gas_status = GasStatus::new(gas_schedule, txn_data.max_gas_amount());

        let result = match txn.payload() {
            payload @ TransactionPayload::Script(_)
            | payload @ TransactionPayload::ScriptFunction(_) => self
                .execute_script_or_script_function(
                    session,
                    &mut gas_status,
                    &txn_data,
                    payload,
                    &account_currency_symbol,
                    log_context,
                ),
            TransactionPayload::ModuleBundle(m) => self.execute_modules(
                session,
                &mut gas_status,
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
            .sub(gas_status.remaining_gas())
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
                        &mut gas_status,
                        &txn_data,
                        storage,
                        &account_currency_symbol,
                        log_context,
                    )
                }
            }
        }
    }

    fn execute_writeset<S: MoveResolver>(
        &self,
        storage: &S,
        writeset_payload: &WriteSetPayload,
        txn_sender: Option<AccountAddress>,
    ) -> Result<ChangeSet, Result<(VMStatus, TransactionOutput), VMStatus>> {
        let mut gas_status = GasStatus::new_unmetered();

        Ok(match writeset_payload {
            WriteSetPayload::Direct(change_set) => change_set.clone(),
            WriteSetPayload::Script { script, execute_as } => {
                let mut tmp_session = self.0.new_session(storage);
                let diem_version = self.0.get_diem_version().map_err(Err)?;
                let senders = match txn_sender {
                    None => vec![*execute_as],
                    Some(sender) => vec![sender, *execute_as],
                };
                let remapped_script = if diem_version < diem_types::on_chain_config::DIEM_VERSION_2
                {
                    None
                } else {
                    script_to_script_function::remapping(script.code())
                };
                let execution_result = match remapped_script {
                    // We are in this case before VERSION_2
                    // or if there is no remapping for the script
                    None => tmp_session.execute_script(
                        script.code().to_vec(),
                        script.ty_args().to_vec(),
                        convert_txn_args(script.args()),
                        senders,
                        &mut gas_status,
                    ),
                    Some((module, function)) => tmp_session.execute_script_function(
                        module,
                        function,
                        script.ty_args().to_vec(),
                        convert_txn_args(script.args()),
                        senders,
                        &mut gas_status,
                    ),
                }
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
        state_view: &impl StateView,
        write_set: &WriteSet,
    ) -> Result<(), VMStatus> {
        // All Move executions satisfy the read-before-write property. Thus we need to read each
        // access path that the write set is going to update.
        for (ap, _) in write_set.iter() {
            state_view
                .get(ap)
                .map_err(|_| VMStatus::Error(StatusCode::STORAGE_ERROR))?;
        }
        Ok(())
    }

    pub(crate) fn process_waypoint_change_set<S: MoveResolver + StateView>(
        &self,
        storage: &S,
        writeset_payload: WriteSetPayload,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let change_set = match self.execute_writeset(storage, &writeset_payload, None) {
            Ok(cs) => cs,
            Err(e) => return e,
        };
        let (write_set, events) = change_set.into_inner();
        self.read_writeset(storage, &write_set)?;
        SYSTEM_TRANSACTIONS_EXECUTED.inc();
        Ok((
            VMStatus::Executed,
            TransactionOutput::new(write_set, events, 0, VMStatus::Executed.into()),
        ))
    }

    pub(crate) fn process_block_prologue<S: MoveResolver>(
        &self,
        storage: &S,
        block_metadata: BlockMetadata,
        log_context: &AdapterLogSchema,
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
        let mut gas_status = GasStatus::new_unmetered();
        let mut session = self.0.new_session(storage);

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
                BLOCK_PROLOGUE,
                vec![],
                args,
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .or_else(|e| {
                expect_only_successful_execution(e, BLOCK_PROLOGUE.as_str(), log_context)
            })?;
        SYSTEM_TRANSACTIONS_EXECUTED.inc();

        let output = get_transaction_output(
            &mut (),
            session,
            gas_status.remaining_gas(),
            &txn_data,
            KeptVMStatus::Executed,
        )?;
        Ok((VMStatus::Executed, output))
    }

    pub(crate) fn process_writeset_transaction<S: MoveResolver + StateView>(
        &self,
        storage: &S,
        txn: &SignatureCheckedTransaction,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        fail_point!("move_adapter::process_writeset_transaction", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let account_currency_symbol = match get_gas_currency_code(txn) {
            Ok(symbol) => symbol,
            Err(err) => {
                return Ok(discard_error_vm_status(err));
            }
        };

        if let Err(err) = get_currency_info(&account_currency_symbol, storage) {
            return Ok(discard_error_vm_status(err));
        }

        // Revalidate the transaction.
        let mut session = self.0.new_session(storage);
        if let Err(e) = validate_signature_checked_transaction::<S, Self>(
            self,
            &mut session,
            txn,
            false,
            log_context,
        ) {
            return Ok(discard_error_vm_status(e));
        };
        self.execute_writeset_transaction(
            storage,
            match txn.payload() {
                TransactionPayload::WriteSet(writeset_payload) => writeset_payload,
                TransactionPayload::ModuleBundle(_)
                | TransactionPayload::Script(_)
                | TransactionPayload::ScriptFunction(_) => {
                    log_context.alert();
                    error!(*log_context, "[diem_vm] UNREACHABLE");
                    return Ok(discard_error_vm_status(VMStatus::Error(
                        StatusCode::UNREACHABLE,
                    )));
                }
            },
            TransactionMetadata::new(txn),
            log_context,
        )
    }

    pub fn execute_writeset_transaction<S: MoveResolver + StateView>(
        &self,
        storage: &S,
        writeset_payload: &WriteSetPayload,
        txn_data: TransactionMetadata,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput), VMStatus> {
        let change_set =
            match self.execute_writeset(storage, writeset_payload, Some(txn_data.sender())) {
                Ok(change_set) => change_set,
                Err(e) => return e,
            };

        // Run the epilogue function.
        let mut session = self.0.new_session(storage);
        self.0.run_writeset_epilogue(
            &mut session,
            &txn_data,
            writeset_payload.should_trigger_reconfiguration_by_default(),
            log_context,
        )?;

        if let Err(e) = self.read_writeset(storage, change_set.write_set()) {
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

    /// Alternate form of 'execute_block' that keeps the vm_status before it goes into the
    /// `TransactionOutput`
    pub fn execute_block_and_keep_vm_status(
        transactions: Vec<Transaction>,
        state_view: &impl StateView,
    ) -> Result<Vec<(VMStatus, TransactionOutput)>, VMStatus> {
        let mut state_view_cache = StateViewCache::new(state_view);
        let count = transactions.len();
        let vm = DiemVM::new(&state_view_cache);
        let res = adapter_common::execute_block_impl(&vm, transactions, &mut state_view_cache)?;
        // Record the histogram count for transactions per block.
        BLOCK_TRANSACTION_COUNT.observe(count as f64);
        Ok(res)
    }
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
        state_view: &impl StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        fail_point!("move_adapter::execute_block", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let now = std::time::Instant::now();
        let len = transactions.len();

        // Execute transactions in parallel if on chain config is set and loaded.
        if let Some(read_write_set_analysis) =
            ParallelExecutionConfig::fetch_config(&RemoteStorage::new(state_view))
                .and_then(|config| config.read_write_analysis_result)
                .map(|config| config.into_inner())
        {
            let analysis_reuslt = NormalizedReadWriteSetAnalysis::new(read_write_set_analysis);

            // Note that writeset transactions will be executed sequentially as it won't be inferred
            // by the read write set analysis and thus fall into the sequential path.
            let (result, _) = crate::parallel_executor::ParallelDiemVM::execute_block(
                &analysis_reuslt,
                transactions,
                state_view,
            )?;

            info!(
                AdapterLogSchema::new(state_view.id(), 0),
                "Time to execute transactions in parallel: {:?}, block size: {:?}",
                now.elapsed(),
                len
            );
            Ok(result)
        } else {
            let output = Self::execute_block_and_keep_vm_status(transactions, state_view)?;
            info!(
                AdapterLogSchema::new(state_view.id(), 0),
                "Time to execute transactions: {:?}, block size: {:?}",
                now.elapsed(),
                len
            );
            Ok(output
                .into_iter()
                .map(|(_vm_status, txn_output)| txn_output)
                .collect())
        }
    }
}

// VMValidator external API
impl VMValidator for DiemVM {
    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. Verification performs the
    /// following steps:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is under given specific configuration.
    /// 3. Invokes `DiemAccount.prologue`, which checks properties such as the transaction has the
    /// right sequence number and the sender has enough balance to pay for the gas.
    /// TBD:
    /// 1. Transaction arguments matches the main function's type signature.
    ///    We don't check this item for now and would execute the check at execution time.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &impl StateView,
    ) -> VMValidatorResult {
        validate_signed_transaction(self, transaction, state_view)
    }
}

impl VMAdapter for DiemVM {
    fn new_session<'r, R: MoveResolver>(&self, remote: &'r R) -> Session<'r, '_, R> {
        self.0.new_session(remote)
    }

    fn check_signature(txn: SignedTransaction) -> Result<SignatureCheckedTransaction> {
        txn.check_signature()
    }

    fn check_transaction_format(&self, txn: &SignedTransaction) -> Result<(), VMStatus> {
        if txn.is_multi_agent() && self.0.get_diem_version()? < DIEM_VERSION_3 {
            // Multi agent is not allowed
            return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
        }
        if txn.contains_duplicate_signers() {
            return Err(VMStatus::Error(StatusCode::SIGNERS_CONTAIN_DUPLICATES));
        }

        Ok(())
    }

    fn get_gas_price<S: MoveResolver>(
        &self,
        txn: &SignedTransaction,
        remote_cache: &S,
    ) -> Result<u64, VMStatus> {
        let gas_price = txn.gas_unit_price();
        let currency_code = get_gas_currency_code(txn)?;

        let normalized_gas_price = match get_currency_info(&currency_code, remote_cache) {
            Ok(info) => info.convert_to_xdx(gas_price),
            Err(err) => {
                return Err(err);
            }
        };

        Ok(normalized_gas_price)
    }

    fn run_prologue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        transaction: &SignatureCheckedTransaction,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let currency_code = get_gas_currency_code(transaction)?;
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Script(_) => {
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_script_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::ScriptFunction(_) => {
                // gate the behavior until the Diem version is ready
                if self.0.get_diem_version()? < DIEM_VERSION_2 {
                    return Err(VMStatus::Error(StatusCode::FEATURE_UNDER_GATING));
                }
                // NOTE: Script and ScriptFunction shares the same prologue
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_script_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::ModuleBundle(_module) => {
                self.0.check_gas(&txn_data, log_context)?;
                self.0
                    .run_module_prologue(session, &txn_data, &currency_code, log_context)
            }
            TransactionPayload::WriteSet(_cs) => {
                self.0
                    .run_writeset_prologue(session, &txn_data, log_context)
            }
        }
    }

    fn should_restart_execution(vm_output: &TransactionOutput) -> bool {
        let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
        vm_output
            .events()
            .iter()
            .any(|event| *event.key() == new_epoch_event_key)
    }

    fn execute_single_transaction<S: MoveResolver + StateView>(
        &self,
        txn: &PreprocessedTransaction,
        data_cache: &S,
        log_context: &AdapterLogSchema,
    ) -> Result<(VMStatus, TransactionOutput, Option<String>), VMStatus> {
        Ok(match txn {
            PreprocessedTransaction::BlockMetadata(block_metadata) => {
                let (vm_status, output) =
                    self.process_block_prologue(data_cache, block_metadata.clone(), log_context)?;
                (vm_status, output, Some("block_prologue".to_string()))
            }
            PreprocessedTransaction::WaypointWriteSet(write_set_payload) => {
                let (vm_status, output) =
                    self.process_waypoint_change_set(data_cache, write_set_payload.clone())?;
                (vm_status, output, Some("waypoint_write_set".to_string()))
            }
            PreprocessedTransaction::UserTransaction(txn) => {
                let sender = txn.sender().to_string();
                let _timer = TXN_TOTAL_SECONDS.start_timer();
                let (vm_status, output) =
                    self.execute_user_transaction(data_cache, txn, log_context);

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
                    self.process_writeset_transaction(data_cache, txn, log_context)?;
                (vm_status, output, Some("write_set".to_string()))
            }
            PreprocessedTransaction::InvalidSignature => {
                let (vm_status, output) =
                    discard_error_vm_status(VMStatus::Error(StatusCode::INVALID_SIGNATURE));
                (vm_status, output, None)
            }
        })
    }
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
