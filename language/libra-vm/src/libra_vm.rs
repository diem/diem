// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*, on_chain_configs::VMConfig as OnlineConfig, system_module_names::*, VMExecutor,
    VMVerifier,
};
use debug_interface::prelude::*;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    account_config,
    block_metadata::BlockMetadata,
    language_storage::{ModuleId, TypeTag},
    on_chain_config::LibraVersion,
    transaction::{
        ChangeSet, Module, Script, SignatureCheckedTransaction, SignedTransaction, Transaction,
        TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
        VMValidatorResult, MAX_TRANSACTION_SIZE_IN_BYTES,
    },
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::{WriteSet, WriteSetMut},
};
use move_vm_runtime::MoveVM;
use move_vm_state::{
    data_cache::{BlockDataCache, RemoteCache, RemoteStorage},
    execution_context::{ExecutionContext, SystemExecutionContext, TransactionExecutionContext},
};
use move_vm_types::{
    chain_state::ChainState, identifier::create_access_path, loaded_data::types::Type,
    values::Value,
};
use rayon::prelude::*;
use std::{collections::HashSet, sync::Arc};
use vm::{
    errors::{convert_prologue_runtime_error, VMResult},
    gas_schedule::{
        self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits, GAS_SCHEDULE_NAME,
    },
    transaction_metadata::TransactionMetadata,
};

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct LibraVM {
    move_vm: Arc<MoveVM>,
    gas_schedule: Option<CostTable>,
    on_chain_config: Option<OnlineConfig>,
}

impl LibraVM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            gas_schedule: None,
            on_chain_config: None,
        }
    }

    pub fn init_with_config(gas_schedule: CostTable, on_chain_config: OnlineConfig) -> Self {
        let inner = MoveVM::new();
        Self {
            move_vm: Arc::new(inner),
            gas_schedule: Some(gas_schedule),
            on_chain_config: Some(on_chain_config),
        }
    }

    pub fn clone_with_config(&self, gas_schedule: CostTable, on_chain_config: OnlineConfig) -> Self {
        Self {
            move_vm: self.move_vm.clone(),
            gas_schedule: Some(gas_schedule),
            on_chain_config: Some(on_chain_config),
        }
    }

    /// Provides access to some internal APIs of the Libra VM.
    pub fn internals(&self) -> LibraVMInternals {
        LibraVMInternals(self)
    }

    pub fn load_configs(&mut self, state: &dyn StateView) {
        self.load_configs_impl(&RemoteStorage::new(state))
    }

    fn on_chain_config(&self) -> VMResult<&OnlineConfig> {
        self.on_chain_config
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl(&mut self, data_cache: &dyn RemoteCache) {
        self.gas_schedule = self.fetch_gas_schedule(data_cache).ok();
        self.on_chain_config = OnlineConfig::load_on_chain_config(data_cache);
    }

    fn fetch_gas_schedule(&mut self, data_cache: &dyn RemoteCache) -> VMResult<CostTable> {
        let address = account_config::association_address();
        let mut ctx = SystemExecutionContext::new(data_cache, GasUnits::new(0));
        let gas_struct_ty = self
            .move_vm
            .resolve_struct_def_by_name(&GAS_SCHEDULE_MODULE, &GAS_SCHEDULE_NAME, &mut ctx, &[])
            .map_err(|_| {
                VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                    .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_MODULE)
            })?;

        let access_path = create_access_path(address, gas_struct_ty.into_struct_tag()?);

        let data_blob = data_cache
            .get(&access_path)
            .map_err(|_| {
                VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                    .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
            })?
            .ok_or_else(|| {
                VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                    .with_sub_status(sub_status::GSE_UNABLE_TO_LOAD_RESOURCE)
            })?;
        let table: CostTable = lcs::from_bytes(&data_blob).map_err(|_| {
            VMStatus::new(StatusCode::GAS_SCHEDULE_ERROR)
                .with_sub_status(sub_status::GSE_UNABLE_TO_DESERIALIZE)
        })?;

        Ok(table)
    }

    pub fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        self.gas_schedule.as_ref().ok_or_else(|| {
            VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND)
        })
    }

    pub fn get_libra_version(&self) -> VMResult<LibraVersion> {
        Ok(self.on_chain_config()?.version.clone())
    }

    fn check_gas(&self, txn: &SignedTransaction) -> VMResult<()> {
        let raw_bytes_len = AbstractMemorySize::new(txn.raw_txn_bytes_len() as GasCarrier);
        // The transaction is too large.
        if txn.raw_txn_bytes_len() > MAX_TRANSACTION_SIZE_IN_BYTES {
            let error_str = format!(
                "max size: {}, txn size: {}",
                MAX_TRANSACTION_SIZE_IN_BYTES,
                raw_bytes_len.get()
            );
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                MAX_TRANSACTION_SIZE_IN_BYTES
            );
            return Err(
                VMStatus::new(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE).with_message(error_str)
            );
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assume!(raw_bytes_len.get() <= MAX_TRANSACTION_SIZE_IN_BYTES as u64);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn.max_gas_amount() > gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get() {
            let error_str = format!(
                "max gas units: {}, gas units submitted: {}",
                gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                txn.max_gas_amount()
            );
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
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
        let min_txn_fee = gas_schedule::calculate_intrinsic_gas(raw_bytes_len);
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
        let below_min_bound = txn.gas_unit_price() < gas_schedule::MIN_PRICE_PER_GAS_UNIT.get();
        if below_min_bound {
            let error_str = format!(
                "gas unit min price: {}, submitted price: {}",
                gas_schedule::MIN_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_schedule::MIN_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND).with_message(error_str)
            );
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn.gas_unit_price() > gas_schedule::MAX_PRICE_PER_GAS_UNIT.get() {
            let error_str = format!(
                "gas unit max price: {}, submitted price: {}",
                gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_schedule::MAX_PRICE_PER_GAS_UNIT.get(),
                txn.gas_unit_price()
            );
            return Err(
                VMStatus::new(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND).with_message(error_str)
            );
        }
        Ok(())
    }

    fn check_change_set(&self, change_set: &ChangeSet, state_view: &dyn StateView) -> VMResult<()> {
        // This function is only invoked by WaypointWriteSet for now. We don't enforce the same
        // check on TransactionPayload::WriteSet.
        if state_view.is_genesis() {
            for (_access_path, write_op) in change_set.write_set() {
                // Genesis transactions only add entries, never delete them.
                if write_op.is_deletion() {
                    error!("[VM] Bad genesis block");
                    return Err(VMStatus::new(StatusCode::INVALID_WRITE_SET));
                }
            }
            Ok(())
        } else {
            Err(VMStatus::new(StatusCode::REJECTED_WRITE_SET))
        }
    }

    fn resolve_type_argument(
        &self,
        ctx: &mut SystemExecutionContext,
        tag: &TypeTag,
    ) -> VMResult<Type> {
        Ok(match tag {
            TypeTag::U8 => Type::U8,
            TypeTag::U64 => Type::U64,
            TypeTag::U128 => Type::U128,
            TypeTag::Bool => Type::Bool,
            TypeTag::Address => Type::Address,
            TypeTag::Vector(tag) => Type::Vector(Box::new(self.resolve_type_argument(ctx, tag)?)),
            TypeTag::Struct(struct_tag) => {
                let module_id = ModuleId::new(struct_tag.address, struct_tag.module.clone());
                let ty_args = struct_tag
                    .type_params
                    .iter()
                    .map(|tag| self.resolve_type_argument(ctx, tag))
                    .collect::<VMResult<Vec<_>>>()?;
                Type::Struct(Box::new(self.move_vm.resolve_struct_def_by_name(
                    &module_id,
                    &struct_tag.name,
                    ctx,
                    &ty_args,
                )?))
            }
        })
    }

    fn verify_script(
        &self,
        remote_cache: &dyn RemoteCache,
        script: &Script,
        txn_data: TransactionMetadata,
    ) -> VMResult<VerifiedTranscationPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        if !self
            .on_chain_config()?
            .publishing_options
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
        };
        self.run_prologue(&mut ctx, &txn_data)?;
        let ty_args = script
            .ty_args()
            .iter()
            .map(|tag| self.resolve_type_argument(&mut ctx, tag))
            .collect::<VMResult<Vec<_>>>()?;
        Ok(VerifiedTranscationPayload::Script(
            script.code().to_vec(),
            ty_args,
            convert_txn_args(script.args()),
        ))
    }

    fn verify_module(
        &self,
        remote_cache: &dyn RemoteCache,
        module: &Module,
        txn_data: TransactionMetadata,
    ) -> VMResult<VerifiedTranscationPayload> {
        let mut ctx = SystemExecutionContext::new(remote_cache, GasUnits::new(0));
        if !&self.on_chain_config()?.publishing_options.is_open() {
            warn!("[VM] Custom modules not allowed");
            return Err(VMStatus::new(StatusCode::UNKNOWN_MODULE));
        };
        self.run_prologue(&mut ctx, &txn_data)?;
        Ok(VerifiedTranscationPayload::Module(module.code().to_vec()))
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
    ) -> VMResult<VerifiedTranscationPayload> {
        self.check_gas(transaction)?;
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
            TransactionPayload::Script(script) => {
                self.verify_script(remote_cache, script, txn_data)
            }
            TransactionPayload::Module(module) => {
                self.verify_module(remote_cache, module, txn_data)
            }
            TransactionPayload::WriteSet(_) => Err(VMStatus::new(StatusCode::UNREACHABLE)),
        }
    }

    fn verify_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &dyn RemoteCache,
    ) -> VMResult<()> {
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
            TransactionPayload::Program => Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT)),
            TransactionPayload::Script(script) => {
                self.check_gas(transaction)?;
                self.verify_script(remote_cache, script, txn_data)?;
                Ok(())
            }
            TransactionPayload::Module(module) => {
                self.check_gas(transaction)?;
                self.verify_module(remote_cache, module, txn_data)?;
                Ok(())
            }
            TransactionPayload::WriteSet(cs) => self.verify_writeset(remote_cache, cs, &txn_data),
        }
    }

    fn execute_verified_payload(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTranscationPayload,
    ) -> TransactionOutput {
        let mut ctx = TransactionExecutionContext::new(txn_data.max_gas_amount(), remote_cache);
        // TODO: The logic for handling falied transaction fee is pretty ugly right now. Fix it later.
        let mut failed_gas_left = GasUnits::new(0);
        match payload {
            VerifiedTranscationPayload::Module(m) => {
                self.move_vm.publish_module(m, &mut ctx, txn_data)
            }
            VerifiedTranscationPayload::Script(s, ty_args, args) => {
                let gas_schedule = match self.get_gas_schedule() {
                    Ok(s) => s,
                    Err(e) => return discard_error_output(e),
                };
                let ret =
                    self.move_vm
                        .execute_script(s, gas_schedule, &mut ctx, txn_data, ty_args, args);
                let gas_usage = txn_data.max_gas_amount().sub(ctx.remaining_gas()).get();
                record_stats!(observe | TXN_EXECUTION_GAS_USAGE | gas_usage);
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
            self.run_epilogue(&mut gas_free_ctx, txn_data)
                .and_then(|_| {
                    get_transaction_output(
                        &mut gas_free_ctx,
                        txn_data,
                        VMStatus::new(StatusCode::EXECUTED),
                    )
                })
        })
        .unwrap_or_else(|err| {
            self.failed_transaction_cleanup(err, failed_gas_left, txn_data, remote_cache)
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
    ) -> TransactionOutput {
        let mut gas_free_ctx = SystemExecutionContext::new(remote_cache, gas_left);
        match TransactionStatus::from(error_code) {
            TransactionStatus::Keep(status) => self
                .run_epilogue(&mut gas_free_ctx, txn_data)
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
        let verified_payload = record_stats! {time_hist | TXN_VERIFICATION_TIME_TAKEN | {
            self.verify_user_transaction_impl(txn, remote_cache)
        }};
        let result = verified_payload
            .and_then(|verified_payload| {
                record_stats! {time_hist | TXN_EXECUTION_TIME_TAKEN | {
                Ok(self.execute_verified_payload(
                    remote_cache,
                    &txn_data,
                    verified_payload,
                ))
                }}
            })
            .unwrap_or_else(discard_error_output);
        if let TransactionStatus::Keep(_) = result.status() {
            remote_cache.push_write_set(result.write_set())
        };
        result
    }

    fn process_waypoint_change_set(
        &mut self,
        remote_cache: &mut BlockDataCache<'_>,
        change_set: ChangeSet,
    ) -> TransactionOutput {
        let (write_set, events) = change_set.into_inner();
        remote_cache.push_write_set(&write_set);
        self.load_configs_impl(remote_cache);
        TransactionOutput::new(
            write_set,
            events,
            0,
            VMStatus::new(StatusCode::EXECUTED).into(),
        )
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
        let gas_schedule = CostTable::zero();

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
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
                self.move_vm
                    .execute_function(
                        &account_config::ACCOUNT_MODULE,
                        &PROLOGUE_NAME,
                        self.get_gas_schedule()?,
                        chain_state,
                        &txn_data,
                        vec![],
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
        }
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_epilogue<T: ChainState>(
        &self,
        chain_state: &mut T,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = chain_state.remaining_gas().get();
        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
                self.move_vm.execute_function(
                    &account_config::ACCOUNT_MODULE,
                    &EPILOGUE_NAME,
                    self.get_gas_schedule()?,
                    chain_state,
                    &txn_data,
                    vec![],
                    vec![
                        Value::u64(txn_sequence_number),
                        Value::u64(txn_gas_price),
                        Value::u64(txn_max_gas_units),
                        Value::u64(gas_remaining),
                    ],
                )
            }
        }
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
        let gas_schedule = CostTable::zero();
        record_stats! {time_hist | TXN_PROLOGUE_TIME_TAKEN | {
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
        }
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
        let gas_schedule = CostTable::zero();

        record_stats! {time_hist | TXN_EPILOGUE_TIME_TAKEN | {
                self.move_vm.execute_function(
                    &LIBRA_WRITESET_MANAGER_MODULE,
                    &EPILOGUE_NAME,
                    &gas_schedule,
                    chain_state,
                    &txn_data,
                    vec![],
                    vec![
                        Value::vector_u8(change_set_bytes),
                    ],
                )
            }
        }
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
                    self.check_change_set(&change_set, state_view)
                        .map(|_| self.process_waypoint_change_set(&mut data_cache, change_set))
                        .unwrap_or_else(discard_error_output),
                ),
                TransactionBlock::WriteSet(txn) => {
                    result.push(self.process_writeset_transaction(&mut data_cache, *txn)?)
                }
            }
        }
        report_block_count(count);
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
            record_stats! {time_hist | TXN_TOTAL_TIME_TAKEN | {
                    let output = match transaction {
                        Ok(txn) => self.execute_user_transaction(state_view, data_cache, &txn),
                        Err(e) => discard_error_output(e),
                    };
                    report_execution_status(output.status());

                    // `result` is initially empty, a single element is pushed per loop iteration and
                    // the number of iterations is bound to the max size of `signature_verified_block`
                    assume!(result.len() < usize::max_value());
                    result.push(output);
                }
            }
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

// Validators external API
impl VMVerifier for LibraVM {
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
        let gas_price = transaction.gas_unit_price();
        record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
                let signature_verified_txn = match transaction.check_signature() {
                    Ok(t) => t,
                    Err(_) => return VMValidatorResult::new(Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)), gas_price),
                };
                let res = match self.verify_transaction_impl(&signature_verified_txn, &data_cache) {
                    Ok(_) => None,
                    Err(err) => {
                        if err.major_status == StatusCode::SEQUENCE_NUMBER_TOO_NEW {
                            None
                        } else {
                            Some(convert_prologue_runtime_error(&err, &signature_verified_txn.sender()))
                        }
                    }
                };
                report_verification_status(&res);
                VMValidatorResult::new(res, gas_price)
            }
        }
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

enum VerifiedTranscationPayload {
    Script(Vec<u8>, Vec<Type>, Vec<Value>),
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
    record_stats!(observe | TXN_TOTAL_GAS_USAGE | gas_used);
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
