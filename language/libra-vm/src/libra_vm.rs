// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters::*, system_module_names::*, VMExecutor, VMValidator};
use debug_interface::prelude::*;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    block_metadata::BlockMetadata,
    language_storage::{ResourceKey, StructTag, TypeTag},
    move_resource::MoveResource,
    on_chain_config::{LibraVersion, OnChainConfig, VMConfig},
    transaction::{
        ChangeSet, Module, Script, SignatureCheckedTransaction, SignedTransaction, Transaction,
        TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
        VMValidatorResult,
    },
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::{WriteSet, WriteSetMut},
};
use move_core_types::{
    gas_schedule::{AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    identifier::{IdentStr, Identifier},
};
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::{BlockDataCache, RemoteCache, RemoteStorage},
    execution_context::{ExecutionContext, SystemExecutionContext, TransactionExecutionContext},
};
use move_vm_types::{
    chain_state::ChainState,
    gas_schedule::{calculate_intrinsic_gas, zero_cost_schedule},
    values::Value,
};
use rayon::prelude::*;
use std::{collections::HashSet, convert::TryFrom, sync::Arc};
use vm::{
    errors::{convert_prologue_runtime_error, VMResult},
    transaction_metadata::TransactionMetadata,
};

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct LibraVM {
    move_vm: Arc<MoveVM>,
    on_chain_config: Option<VMConfig>,
    version: Option<LibraVersion>,
}

impl LibraVM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            on_chain_config: None,
            version: None,
        }
    }

    pub fn init_with_config(version: LibraVersion, on_chain_config: VMConfig) -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            on_chain_config: Some(on_chain_config),
            version: Some(version),
        }
    }

    /// Provides access to some internal APIs of the Libra VM.
    pub fn internals(&self) -> LibraVMInternals {
        LibraVMInternals(self)
    }

    pub fn load_configs(&mut self, state: &dyn StateView) {
        self.load_configs_impl(&RemoteStorage::new(state))
    }

    fn on_chain_config(&self) -> VMResult<&VMConfig> {
        self.on_chain_config
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl(&mut self, data_cache: &dyn RemoteCache) {
        self.on_chain_config = VMConfig::fetch_config(data_cache);
        self.version = LibraVersion::fetch_config(data_cache);
    }

    pub fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        self.on_chain_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| {
                VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                    .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND)
            })
    }

    pub fn get_libra_version(&self) -> VMResult<LibraVersion> {
        self.version.clone().ok_or_else(|| {
            VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                .with_sub_status(sub_status::VSF_LIBRA_VERSION_NOT_FOUND)
        })
    }

    fn check_gas(&self, txn: &SignedTransaction) -> VMResult<()> {
        let gas_constants = &self.get_gas_schedule()?.gas_constants;
        let raw_bytes_len = AbstractMemorySize::new(txn.raw_txn_bytes_len() as GasCarrier);
        // The transaction is too large.
        if txn.raw_txn_bytes_len() > gas_constants.max_transaction_size_in_bytes as usize {
            let error_str = format!(
                "max size: {}, txn size: {}",
                gas_constants.max_transaction_size_in_bytes,
                raw_bytes_len.get()
            );
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                gas_constants.max_transaction_size_in_bytes
            );
            return Err(
                VMStatus::new(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE).with_message(error_str)
            );
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assume!(raw_bytes_len.get() <= gas_constants.max_transaction_size_in_bytes);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn.max_gas_amount() > gas_constants.maximum_number_of_gas_units.get() {
            let error_str = format!(
                "max gas units: {}, gas units submitted: {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND)
                    .with_message(error_str),
            );
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = calculate_intrinsic_gas(raw_bytes_len, gas_constants);
        if txn.max_gas_amount() < min_txn_fee.get() {
            let error_str = format!(
                "min gas required for txn: {}, gas submitted: {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn.max_gas_amount()
            );
            return Err(
                VMStatus::new(StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS)
                    .with_message(error_str),
            );
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound = txn.gas_unit_price() < gas_constants.min_price_per_gas_unit.get();
        if below_min_bound {
            let error_str = format!(
                "gas unit min price: {}, submitted price: {}",
                gas_constants.min_transaction_gas_units.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_transaction_gas_units.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND).with_message(error_str)
            );
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn.gas_unit_price() > gas_constants.max_price_per_gas_unit.get() {
            let error_str = format!(
                "gas unit max price: {}, submitted price: {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND).with_message(error_str)
            );
        }
        Ok(())
    }

    fn verify_script(
        &self,
        remote_cache: &dyn RemoteCache,
        script: &Script,
        txn_data: TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        if !self
            .on_chain_config()?
            .publishing_option
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
        };
        self.run_prologue(&mut ctx, &txn_data, account_currency_symbol)?;
        Ok(VerifiedTransactionPayload::Script(
            script.code().to_vec(),
            script.ty_args().to_vec(),
            convert_txn_args(script.args()),
        ))
    }

    fn verify_module(
        &self,
        remote_cache: &dyn RemoteCache,
        module: &Module,
        txn_data: TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        if !&self.on_chain_config()?.publishing_option.is_open() {
            warn!("[VM] Custom modules not allowed");
            return Err(VMStatus::new(StatusCode::UNKNOWN_MODULE));
        };
        self.run_prologue(&mut ctx, &txn_data, account_currency_symbol)?;
        Ok(VerifiedTransactionPayload::Module(module.code().to_vec()))
    }

    fn verify_writeset(
        &self,
        remote_cache: &dyn RemoteCache,
        _change_set: &ChangeSet,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        self.run_writeset_prologue(&mut ctx, &txn_data)?;
        Ok(())
    }

    fn verify_user_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &dyn RemoteCache,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        self.check_gas(transaction)?;
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
            TransactionPayload::Script(script) => {
                self.verify_script(remote_cache, script, txn_data, account_currency_symbol)
            }
            TransactionPayload::Module(module) => {
                self.verify_module(remote_cache, module, txn_data, account_currency_symbol)
            }
            TransactionPayload::WriteSet(_) => Err(VMStatus::new(StatusCode::UNREACHABLE)),
        }
    }

    fn verify_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &dyn RemoteCache,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
            TransactionPayload::Script(script) => {
                self.check_gas(transaction)?;
                self.verify_script(remote_cache, script, txn_data, account_currency_symbol)?;
                Ok(())
            }
            TransactionPayload::Module(module) => {
                self.check_gas(transaction)?;
                self.verify_module(remote_cache, module, txn_data, account_currency_symbol)?;
                Ok(())
            }
            TransactionPayload::WriteSet(cs) => self.verify_writeset(remote_cache, cs, &txn_data),
        }
    }

    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTransactionPayload,
        account_currency_symbol: &IdentStr,
    ) -> TransactionOutput {
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        // TODO: The logic for handling falied transaction fee is pretty ugly right now. Fix it later.
        let mut failed_gas_left = GasUnits::new(0);
        match payload {
            VerifiedTransactionPayload::Module(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTransactionPayload::Script(s, ty_args, args) => {
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => return discard_error_output(e),
                };
                let ret =
                    self.move_vm
                        .execute_script(s, gas_schedule, &mut ctx, txn_data, ty_args, args);
                let gas_usage = txn_data.max_gas_amount().sub(ctx.remaining_gas()).get();
                TXN_EXECUTION_GAS_USAGE.observe(gas_usage as f64);
                ret
            }
        }
        .map_err(|err| {
            failed_gas_left = ctx.remaining_gas();
            err
        })
        .and_then(|_| {
            failed_gas_left = ctx.remaining_gas();
            let mut gas_free_ctx = SystemExecutionContext::from(ctx);
            self.run_epilogue(&mut gas_free_ctx, txn_data, account_currency_symbol)
                .and_then(|_| {
                    get_transaction_output(
                        &mut gas_free_ctx,
                        txn_data,
                        VMStatus::new(StatusCode::EXECUTED),
                    )
                })
        })
        .unwrap_or_else(|err| {
            self.failed_transaction_cleanup(
                err,
                failed_gas_left,
                txn_data,
                remote_cache,
                account_currency_symbol,
            )
        })
    }

    /// Generates a transaction output for a transaction that encountered errors during the
    /// execution process. This is public for now only for tests.
    pub fn failed_transaction_cleanup(
        &self,
        error_code: VMStatus,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &mut BlockDataCache<'_>,
        account_currency_symbol: &IdentStr,
    ) -> TransactionOutput {
        let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, gas_left);
        match TransactionStatus::from(error_code) {
            TransactionStatus::Keep(status) => self
                .run_epilogue(&mut gas_free_ctx, txn_data, account_currency_symbol)
                .and_then(|_| get_transaction_output(&mut gas_free_ctx, txn_data, status))
                .unwrap_or_else(discard_error_output),
            TransactionStatus::Discard(status) => discard_error_output(status),
            TransactionStatus::Retry => unreachable!(),
        }
    }

    fn execute_user_transaction(
        &mut self,
        _state_view: &dyn StateView,
        remote_cache: &mut BlockDataCache<'_>,
        txn: &SignatureCheckedTransaction,
    ) -> TransactionOutput {
        let txn_data = TransactionMetadata::new(txn);
        let account_currency_symbol = account_curreny_currency_code(txn.sender(), remote_cache);
        let verified_payload = account_currency_symbol.clone().and_then(|currency_code| {
            let _timer = TXN_VERIFICATION_SECONDS.start_timer();
            self.verify_user_transaction_impl(txn, remote_cache, &currency_code)
        });
        let result = verified_payload
            .and_then(|verified_payload| {
                account_currency_symbol.and_then(|account_currency_symbol| {
                    let _timer = TXN_EXECUTION_SECONDS.start_timer();
                    Ok(self.execute_verified_payload(
                        remote_cache,
                        &txn_data,
                        verified_payload,
                        &account_currency_symbol,
                    ))
                })
            })
            .unwrap_or_else(discard_error_output);
        if let TransactionStatus::Keep(_) = result.status() {
            remote_cache.push_write_set(result.write_set())
        };
        result
    }

    fn read_writeset(
        &self,
        remote_cache: &BlockDataCache<'_>,
        write_set: &WriteSet,
    ) -> VMResult<()> {
        // All Move executions satisfy the read-before-write property. Thus we need to read each
        // access path that the write set is going to update.
        for (ap, _) in write_set.iter() {
            remote_cache.get(ap)?;
        }
        Ok(())
    }

    fn process_waypoint_change_set(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        change_set: ChangeSet,
    ) -> VMResult<TransactionOutput> {
        let (write_set, events) = change_set.into_inner();
        self.read_writeset(remote_cache, &write_set)?;
        remote_cache.push_write_set(&write_set);
        self.load_configs_impl(remote_cache);
        Ok(TransactionOutput::new(
            write_set,
            events,
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        ))
    }

    fn process_block_prologue(
        &self,
        remote_cache: &mut BlockDataCache<'_>,
        block_metadata: BlockMetadata,
    ) -> VMResult<TransactionOutput> {
        // TODO: How should we setup the metadata here? A couple of thoughts here:
        // 1. We might make the txn_data to be poisoned so that reading anything will result in a panic.
        // 2. The most important consideration is figuring out the sender address.  Having a notion of a
        //    "null address" (probably 0x0...0) that is prohibited from containing modules or resources
        //    might be useful here.
        // 3. We set the max gas to a big number just to get rid of the potential out of gas error.
        let mut txn_data = TransactionMetadata::default();
        txn_data.sender = account_config::CORE_CODE_ADDRESS;
        txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);

        let mut interpreter_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        // TODO: We might need a non zero cost table here so that we can at least bound the execution
        //       time by a reasonable amount.
        let gas_schedule = zero_cost_schedule();

        if let Ok((round, timestamp, previous_vote, proposer)) = block_metadata.into_inner() {
            let args = vec![
                Value::u64(round),
                Value::u64(timestamp),
                Value::vector_address(previous_vote),
                Value::address(proposer),
            ];
            self.move_vm.execute_function(
                &LIBRA_BLOCK_MODULE,
                &BLOCK_PROLOGUE,
                &gas_schedule,
                &mut interpreter_context,
                &txn_data,
                vec![],
                args,
            )?
        } else {
            return Err(VMStatus::new(StatusCode::MALFORMED));
        };
        get_transaction_output(
            &mut interpreter_context,
            &txn_data,
            VMStatus::new(StatusCode::EXECUTED),
        )
        .map(|output| {
            remote_cache.push_write_set(output.write_set());
            output
        })
    }

    fn process_writeset_transaction(
        &self,
        remote_cache: &mut BlockDataCache<'_>,
        txn: SignedTransaction,
    ) -> VMResult<TransactionOutput> {
        let txn = match txn.check_signature() {
            Ok(t) => t,
            _ => {
                return Ok(discard_error_output(VMStatus::new(
                    StatusCode::INVALID_SIGNATURE,
                )))
            }
        };

        let change_set = if let TransactionPayload::WriteSet(change_set) = txn.payload() {
            change_set
        } else {
            error!("[libra_vm] UNREACHABLE");
            return Ok(discard_error_output(VMStatus::new(StatusCode::UNREACHABLE)));
        };

        let txn_data = TransactionMetadata::new(&txn);

        if let Err(e) = self.verify_writeset(remote_cache, change_set, &txn_data) {
            return Ok(discard_error_output(e));
        };

        let mut interpreter_context = SystemExecutionContext::new(remote_cache, GasUnits::new(0));

        self.run_writeset_epilogue(&mut interpreter_context, change_set, &txn_data)?;

        if let Err(e) = self.read_writeset(remote_cache, &change_set.write_set()) {
            return Ok(discard_error_output(e));
        };

        let epilogue_writeset = interpreter_context.make_write_set()?;
        let epilogue_events = interpreter_context.events().to_vec();

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
            return Ok(discard_error_output(VMStatus::new(
                StatusCode::INVALID_WRITE_SET,
            )));
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
            return Ok(discard_error_output(VMStatus::new(
                StatusCode::INVALID_WRITE_SET,
            )));
        }

        let write_set = WriteSetMut::new(
            epilogue_writeset
                .iter()
                .chain(change_set.write_set().iter())
                .cloned()
                .collect(),
        )
        .freeze()
        .map_err(|_| VMStatus::new(StatusCode::INVALID_WRITE_SET))?;
        let events = change_set
            .events()
            .iter()
            .chain(epilogue_events.iter())
            .cloned()
            .collect();

        Ok(TransactionOutput::new(
            write_set,
            events,
            0,
            TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
        ))
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_prologue<T: ChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        let _timer = TXN_PROLOGUE_SECONDS.start_timer();
        self.move_vm
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                self.get_gas_schedule()?,
                chain_state,
                &txn_data,
                vec![gas_currency_ty],
                vec![
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                    Value::u64(txn_expiration_time),
                ],
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<T: ChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = chain_state.remaining_gas().get();
        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        self.move_vm.execute_function(
            &account_config::ACCOUNT_MODULE,
            &EPILOGUE_NAME,
            self.get_gas_schedule()?,
            chain_state,
            &txn_data,
            vec![gas_currency_ty],
            vec![
                Value::u64(txn_sequence_number),
                Value::u64(txn_gas_price),
                Value::u64(txn_max_gas_units),
                Value::u64(gas_remaining),
            ],
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    fn run_writeset_prologue<T: ChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let gas_schedule = zero_cost_schedule();
        let _timer = TXN_PROLOGUE_SECONDS.start_timer();
        self.move_vm
            .execute_function(
                &LIBRA_WRITESET_MANAGER_MODULE,
                &PROLOGUE_NAME,
                &gas_schedule,
                chain_state,
                &txn_data,
                vec![],
                vec![
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                ],
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    fn run_writeset_epilogue<T: ChainState>(
        &self,
        chain_state: &mut T,
        change_set: &ChangeSet,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let change_set_bytes =
            lcs::to_bytes(change_set).map_err(|_| VMStatus::new(StatusCode::INVALID_DATA))?;
        let gas_schedule = zero_cost_schedule();

        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        self.move_vm.execute_function(
            &LIBRA_WRITESET_MANAGER_MODULE,
            &EPILOGUE_NAME,
            &gas_schedule,
            chain_state,
            &txn_data,
            vec![],
            vec![Value::vector_u8(change_set_bytes)],
        )
    }

    fn execute_block_impl(
        &mut self,
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let count = transactions.len();
        let mut result = vec![];
        let blocks = chunk_block_transactions(transactions);
        let mut data_cache = BlockDataCache::new(state_view);
        let mut execute_block_trace_guard = vec![];
        let mut current_block_id = HashValue::zero();
        for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    let mut outs = self.execute_user_transactions(
                        current_block_id,
                        txns,
                        &mut data_cache,
                        state_view,
                    )?;
                    result.append(&mut outs);
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    execute_block_trace_guard.clear();
                    current_block_id = block_metadata.id();
                    trace_code_block!("libra_vm::execute_block_impl", {"block", current_block_id}, execute_block_trace_guard);
                    result.push(self.process_block_prologue(&mut data_cache, block_metadata)?)
                }
                TransactionBlock::WaypointWriteSet(change_set) => result.push(
                    self.process_waypoint_change_set(&mut data_cache, change_set)
                        .unwrap_or_else(discard_error_output),
                ),
                TransactionBlock::WriteSet(txn) => {
                    result.push(self.process_writeset_transaction(&mut data_cache, *txn)?)
                }
            }
        }

        // Record the histogram count for transactions per block.
        match i64::try_from(count) {
            Ok(val) => BLOCK_TRANSACTION_COUNT.set(val),
            Err(_) => BLOCK_TRANSACTION_COUNT.set(std::i64::MAX),
        }

        Ok(result)
    }

    fn execute_user_transactions(
        &mut self,
        block_id: HashValue,
        txn_block: Vec<SignedTransaction>,
        data_cache: &mut BlockDataCache<'_>,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        self.load_configs_impl(data_cache);
        let signature_verified_block: Vec<Result<SignatureCheckedTransaction, VMStatus>>;
        {
            trace_code_block!("libra_vm::verify_signatures", {"block", block_id});
            signature_verified_block = txn_block
                .into_par_iter()
                .map(|txn| {
                    txn.check_signature()
                        .map_err(|_| VMStatus::new(StatusCode::INVALID_SIGNATURE))
                })
                .collect();
        }
        let mut result = vec![];
        trace_code_block!("libra_vm::execute_transactions", {"block", block_id});
        for transaction in signature_verified_block {
            let output = match transaction {
                Ok(txn) => {
                    let _timer = TXN_TOTAL_SECONDS.start_timer();
                    self.execute_user_transaction(state_view, data_cache, &txn)
                }
                Err(e) => discard_error_output(e),
            };

            // Increment the counter for transactions executed.
            let counter_label = match output.status() {
                TransactionStatus::Keep(_) => Some("success"),
                TransactionStatus::Discard(_) => Some("discarded"),
                TransactionStatus::Retry => None,
            };
            if let Some(label) = counter_label {
                TRANSACTIONS_EXECUTED.with_label_values(&[label]).inc();
            }

            // `result` is initially empty, a single element is pushed per loop iteration and
            // the number of iterations is bound to the max size of `signature_verified_block`
            assume!(result.len() < usize::max_value());
            result.push(output);
        }
        Ok(result)
    }
}

pub(crate) fn discard_error_output(err: VMStatus) -> TransactionOutput {
    // Since this transaction will be discarded, no writeset will be included.
    TransactionOutput::new(
        WriteSet::default(),
        vec![],
        0,
        TransactionStatus::Discard(err),
    )
}

// VMValidator external API
impl VMValidator for LibraVM {
    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. Verification performs the
    /// following steps:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is under given specific configuration.
    /// 3. Invokes `LibraAccount.prologue`, which checks properties such as the transaction has the
    /// right sequence number and the sender has enough balance to pay for the gas.
    /// TBD:
    /// 1. Transaction arguments matches the main function's type signature.
    ///    We don't check this item for now and would execute the check at execution time.
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> VMValidatorResult {
        let data_cache = BlockDataCache::new(state_view);
        let _timer = TXN_VALIDATION_SECONDS.start_timer();
        let gas_price = transaction.gas_unit_price();
        let txn_sender = transaction.sender();
        let signature_verified_txn = if let Ok(t) = transaction.check_signature() {
            t
        } else {
            return VMValidatorResult::new(
                Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
                gas_price,
                false,
            );
        };

        let is_governance_txn = is_governance_txn(txn_sender, &data_cache);
        let currency_code = match account_curreny_currency_code(txn_sender, &data_cache) {
            Ok(code) => code,
            Err(err) => return VMValidatorResult::new(Some(err), gas_price, false),
        };

        let normalized_gas_price = match normalize_gas_price(gas_price, &currency_code, &data_cache)
        {
            Ok(price) => price,
            Err(err) => return VMValidatorResult::new(Some(err), gas_price, false),
        };

        let res = match self.verify_transaction_impl(
            &signature_verified_txn,
            &data_cache,
            &currency_code,
        ) {
            Ok(_) => None,
            Err(err) => {
                if err.major_status == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                    None
                } else {
                    Some(convert_prologue_runtime_error(
                        &err,
                        &signature_verified_txn.sender(),
                    ))
                }
            }
        };

        // Increment the counter for transactions verified.
        let counter_label = match res {
            None => "success",
            Some(_) => "failure",
        };
        TRANSACTIONS_VERIFIED
            .with_label_values(&[counter_label])
            .inc();

        VMValidatorResult::new(res, normalized_gas_price, is_governance_txn)
    }
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
    ) -> VMResult<Vec<TransactionOutput>> {
        let mut vm = LibraVM::new();
        vm.execute_block_impl(transactions, state_view)
    }
}

/// Internal APIs for the Libra VM, primarily used for testing.
#[derive(Clone, Copy)]
pub struct LibraVMInternals<'a>(&'a LibraVM);

impl<'a> LibraVMInternals<'a> {
    /// Returns the internal Move VM instance.
    pub fn move_vm(self) -> &'a MoveVM {
        &self.0.move_vm
    }

    /// Returns the internal gas schedule if it has been loaded, or an error if it hasn't.
    pub fn gas_schedule(self) -> VMResult<&'a CostTable> {
        self.0.get_gas_schedule()
    }

    /// Returns the version of Move Runtime.
    pub fn libra_version(self) -> VMResult<LibraVersion> {
        self.0.get_libra_version()
    }

    /// Executes the given code within the context of a transaction.
    ///
    /// The `TransactionExecutionContext` can be used as a `ChainState`.
    ///
    /// If you don't care about the transaction metadata, use `TransactionMetadata::default()`.
    pub fn with_txn_context<T>(
        self,
        txn_data: &TransactionMetadata,
        state_view: &dyn StateView,
        f: impl for<'txn> FnOnce(TransactionExecutionContext<'txn>) -> T,
    ) -> T {
        let remote_storage = RemoteStorage::new(state_view);
        let txn_context =
            TransactionExecutionContext::new(txn_data.max_gas_amount(), &remote_storage);
        f(txn_context)
    }
}

fn account_curreny_currency_code(
    sender: AccountAddress,
    state_view: &dyn RemoteCache,
) -> VMResult<Identifier> {
    let account_access_path =
        create_access_path(sender, account_config::AccountResource::struct_tag());
    if let Ok(Some(blob)) = state_view.get(&account_access_path) {
        Ok(lcs::from_bytes::<account_config::AccountResource>(&blob)
            .map_err(|_| VMStatus::new(StatusCode::UNABLE_TO_DESERIALIZE_ACCOUNT))?
            .balance_currency_code()
            .to_owned())
    } else {
        Err(VMStatus::new(StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST))
    }
}

fn is_governance_txn(sender: AccountAddress, remote_cache: &dyn RemoteCache) -> bool {
    let association_capability_path =
        create_access_path(sender, association_capability_struct_tag());
    if let Ok(Some(blob)) = remote_cache.get(&association_capability_path) {
        return lcs::from_bytes::<account_config::AssociationCapabilityResource>(&blob)
            .map(|x| x.is_certified())
            .unwrap_or(false);
    }
    false
}

fn normalize_gas_price(
    gas_price: u64,
    account_curreny_currency_code: &IdentStr,
    remote_cache: &dyn RemoteCache,
) -> VMResult<u64> {
    let currency_info_path = account_config::CurrencyInfoResource::resource_path_for(
        account_curreny_currency_code.to_owned(),
    );
    if let Ok(Some(blob)) = remote_cache.get(&currency_info_path) {
        let x = lcs::from_bytes::<account_config::CurrencyInfoResource>(&blob)
            .map_err(|_| VMStatus::new(StatusCode::CURRENCY_INFO_DOES_NOT_EXIST))?;
        Ok(x.convert_to_lbr(gas_price))
    } else {
        Err(VMStatus::new(StatusCode::MISSING_DATA)
            .with_message("Cannot find curency info in gas price normalization".to_string()))
    }
}

/// Transactions divided by transaction flow.
/// Transaction flows are different across different types of transactions.
pub enum TransactionBlock {
    UserTransaction(Vec<SignedTransaction>),
    WaypointWriteSet(ChangeSet),
    BlockPrologue(BlockMetadata),
    WriteSet(Box<SignedTransaction>),
}

pub fn chunk_block_transactions(txns: Vec<Transaction>) -> Vec<TransactionBlock> {
    let mut blocks = vec![];
    let mut buf = vec![];
    for txn in txns {
        match txn {
            Transaction::BlockMetadata(data) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::BlockPrologue(data));
            }
            Transaction::WaypointWriteSet(cs) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::WaypointWriteSet(cs));
            }
            Transaction::UserTransaction(txn) => {
                if let TransactionPayload::WriteSet(_) = txn.payload() {
                    if !buf.is_empty() {
                        blocks.push(TransactionBlock::UserTransaction(buf));
                        buf = vec![];
                    }
                    blocks.push(TransactionBlock::WriteSet(Box::new(txn)));
                } else {
                    buf.push(txn);
                }
            }
        }
    }
    if !buf.is_empty() {
        blocks.push(TransactionBlock::UserTransaction(buf));
    }
    blocks
}

enum VerifiedTransactionPayload {
    Script(Vec<u8>, Vec<TypeTag>, Vec<Value>),
    Module(Vec<u8>),
}

/// Convert the transaction arguments into move values.
fn convert_txn_args(args: &[TransactionArgument]) -> Vec<Value> {
    args.iter()
        .map(|arg| match arg {
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

/// Get the AccessPath to a resource stored under `address` with type name `tag`
fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    let resource_tag = ResourceKey::new(address, tag);
    AccessPath::resource_access_path(&resource_tag)
}

fn get_transaction_output(
    ctx: &mut (impl ChainState + ExecutionContext),
    txn_data: &TransactionMetadata,
    status: VMStatus,
) -> VMResult<TransactionOutput> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(ctx.remaining_gas())
        .mul(txn_data.gas_unit_price())
        .get();
    let write_set = ctx.make_write_set()?;
    TXN_TOTAL_GAS_USAGE.observe(gas_used as f64);
    Ok(TransactionOutput::new(
        write_set,
        ctx.events().to_vec(),
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

#[test]
fn vm_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<LibraVM>();
    assert_sync::<LibraVM>();
    assert_send::<MoveVM>();
    assert_sync::<MoveVM>();
}
