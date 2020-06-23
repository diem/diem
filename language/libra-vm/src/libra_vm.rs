// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    counters::*,
    create_access_path,
    data_cache::{RemoteStorage, StateViewCache},
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
    VMExecutor, VMValidator,
};
use debug_interface::prelude::*;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{self, RoleId},
    block_metadata::BlockMetadata,
    contract_event::ContractEvent,
    event::EventKey,
    on_chain_config::{ConfigStorage, LibraVersion, OnChainConfig, VMConfig},
    transaction::{
        ChangeSet, Module, Script, SignatureCheckedTransaction, SignedTransaction, Transaction,
        TransactionArgument, TransactionOutput, TransactionPayload, TransactionStatus,
        VMValidatorResult,
    },
    vm_status::{sub_status, StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_core_types::{
    gas_schedule::{AbstractMemorySize, CostTable, GasAlgebra, GasCarrier, GasUnits},
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    move_resource::MoveResource,
};

use move_vm_runtime::{
    data_cache::{RemoteCache, TransactionEffects},
    move_vm::MoveVM,
    session::Session,
};
use move_vm_types::{
    gas_schedule::{calculate_intrinsic_gas, zero_cost_schedule, CostStrategy},
    values::Value,
};
use rayon::prelude::*;
use std::{
    collections::{btree_map, BTreeMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};
use vm::errors::{convert_prologue_runtime_error, VMResult};

/// Any transation sent from an account with a role id below this cutoff will be priorited over
/// other transactions.
const PRIORITIZED_TRANSACTION_ROLE_CUTOFF: u64 = 5;

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct LibraVM {
    ap_cache: BTreeAccessPathCache,
    move_vm: Arc<MoveVM>,
    on_chain_config: Option<VMConfig>,
    version: Option<LibraVersion>,
}

macro_rules! gas_schedule {
    ($s: expr) => {
        $s.on_chain_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| {
                VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                    .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND)
            })
    };
}

impl LibraVM {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let inner = MoveVM::new();
        Self {
            ap_cache: BTreeAccessPathCache::new(),
            move_vm: Arc::new(inner),
            on_chain_config: None,
            version: None,
        }
    }

    pub fn init_with_config(version: LibraVersion, on_chain_config: VMConfig) -> Self {
        let inner = MoveVM::new();
        Self {
            ap_cache: BTreeAccessPathCache::new(),
            move_vm: Arc::new(inner),
            on_chain_config: Some(on_chain_config),
            version: Some(version),
        }
    }

    /// Provides access to some internal APIs of the Libra VM.
    pub fn internals(&self) -> LibraVMInternals {
        LibraVMInternals(self)
    }

    pub fn load_configs<S: StateView>(&mut self, state: &S) {
        self.load_configs_impl(&RemoteStorage::new(state))
    }

    fn on_chain_config(&self) -> VMResult<&VMConfig> {
        self.on_chain_config
            .as_ref()
            .ok_or_else(|| VMStatus::new(StatusCode::VM_STARTUP_FAILURE))
    }

    fn load_configs_impl<S: ConfigStorage>(&mut self, data_cache: &S) {
        self.on_chain_config = VMConfig::fetch_config(data_cache);
        self.version = LibraVersion::fetch_config(data_cache);
    }

    pub fn get_gas_schedule(&self) -> VMResult<&CostTable> {
        gas_schedule!(self)
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
                gas_constants.min_price_per_gas_unit.get(),
                txn.gas_unit_price()
            );
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_price_per_gas_unit.get(),
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
        remote_cache: &StateViewCache,
        script: &Script,
        txn_data: TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        let mut session = self.move_vm.new_session(remote_cache);
        if !self
            .on_chain_config()?
            .publishing_option
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
        };
        self.run_prologue(
            &mut session,
            &mut cost_strategy,
            &txn_data,
            account_currency_symbol,
        )?;
        Ok(VerifiedTransactionPayload::Script(
            script.code().to_vec(),
            script.ty_args().to_vec(),
            convert_txn_args(script.args()),
        ))
    }

    fn verify_module(
        &self,
        remote_cache: &StateViewCache,
        module: &Module,
        txn_data: TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        let mut cost_strategy = CostStrategy::system(self.get_gas_schedule()?, GasUnits::new(0));
        let mut session = self.move_vm.new_session(remote_cache);
        if !&self.on_chain_config()?.publishing_option.is_open()
            && !can_publish_modules(txn_data.sender(), remote_cache)
        {
            warn!("[VM] Custom modules not allowed");
            return Err(VMStatus::new(StatusCode::INVALID_MODULE_PUBLISHER));
        };
        self.run_prologue(
            &mut session,
            &mut cost_strategy,
            &txn_data,
            account_currency_symbol,
        )?;
        Ok(VerifiedTransactionPayload::Module(module.code().to_vec()))
    }

    fn verify_writeset(
        &self,
        remote_cache: &StateViewCache,
        _change_set: &ChangeSet,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let mut session = self.move_vm.new_session(remote_cache);
        self.run_writeset_prologue(&mut session, &txn_data)?;
        Ok(())
    }

    fn verify_user_transaction_impl(
        &self,
        transaction: &SignatureCheckedTransaction,
        remote_cache: &StateViewCache,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<VerifiedTransactionPayload> {
        self.check_gas(transaction)?;
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
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
        remote_cache: &StateViewCache,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let txn_data = TransactionMetadata::new(transaction);
        match transaction.payload() {
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
        remote_cache: &mut StateViewCache<'_>,
        txn_data: &TransactionMetadata,
        payload: VerifiedTransactionPayload,
        account_currency_symbol: &IdentStr,
    ) -> TransactionOutput {
        let gas_schedule = match gas_schedule!(self) {
            Ok(s) => s,
            Err(e) => return discard_error_output(e),
        };
        let mut cost_strategy = CostStrategy::transaction(gas_schedule, txn_data.max_gas_amount());
        let mut session = self.move_vm.new_session(remote_cache);

        macro_rules! unwrap_or_fail {
            ($res: expr) => {
                match $res {
                    Ok(x) => x,
                    Err(e) => {
                        let remaining_gas = cost_strategy.remaining_gas();
                        return self.failed_transaction_cleanup(
                            e,
                            remaining_gas,
                            txn_data,
                            remote_cache,
                            account_currency_symbol,
                        );
                    }
                }
            };
        }

        match payload {
            VerifiedTransactionPayload::Module(m) => {
                unwrap_or_fail!(cost_strategy.charge_intrinsic_gas(txn_data.transaction_size()));
                let module_address = if unwrap_or_fail!(self.on_chain_config())
                    .publishing_option
                    .is_open()
                {
                    txn_data.sender()
                } else {
                    account_config::CORE_CODE_ADDRESS
                };
                unwrap_or_fail!(session.publish_module(m, module_address, &mut cost_strategy,))
            }
            VerifiedTransactionPayload::Script(s, ty_args, args) => {
                unwrap_or_fail!(cost_strategy.charge_intrinsic_gas(txn_data.transaction_size()));
                unwrap_or_fail!(session.execute_script(
                    s,
                    ty_args,
                    args,
                    txn_data.sender(),
                    &mut cost_strategy,
                ));
                let gas_usage = txn_data
                    .max_gas_amount()
                    .sub(cost_strategy.remaining_gas())
                    .get();
                TXN_EXECUTION_GAS_USAGE.observe(gas_usage as f64);
            }
        }

        let mut cost_strategy = CostStrategy::system(gas_schedule, cost_strategy.remaining_gas());
        unwrap_or_fail!(self.run_success_epilogue(
            &mut session,
            &mut cost_strategy,
            txn_data,
            account_currency_symbol,
        ));
        unwrap_or_fail!(get_transaction_output(
            &mut self.ap_cache,
            session,
            &cost_strategy,
            txn_data,
            VMStatus::new(StatusCode::EXECUTED),
        ))
    }

    /// Generates a transaction output for a transaction that encountered errors during the
    /// execution process. This is public for now only for tests.
    pub fn failed_transaction_cleanup(
        &mut self,
        error_code: VMStatus,
        gas_left: GasUnits<GasCarrier>,
        txn_data: &TransactionMetadata,
        remote_cache: &mut StateViewCache<'_>,
        account_currency_symbol: &IdentStr,
    ) -> TransactionOutput {
        let gas_schedule = match gas_schedule!(self) {
            Ok(s) => s,
            Err(e) => return discard_error_output(e),
        };
        let mut cost_strategy = CostStrategy::system(gas_schedule, gas_left);
        let mut session = self.move_vm.new_session(remote_cache);
        match TransactionStatus::from(error_code) {
            TransactionStatus::Keep(status) => {
                if let Err(e) = self.run_failure_epilogue(
                    &mut session,
                    &mut cost_strategy,
                    txn_data,
                    account_currency_symbol,
                ) {
                    return discard_error_output(e);
                }
                get_transaction_output(
                    &mut self.ap_cache,
                    session,
                    &cost_strategy,
                    txn_data,
                    status,
                )
                .unwrap_or_else(discard_error_output)
            }
            TransactionStatus::Discard(status) => discard_error_output(status),
            TransactionStatus::Retry => unreachable!(),
        }
    }

    fn execute_user_transaction(
        &mut self,
        _state_view: &dyn StateView,
        remote_cache: &mut StateViewCache<'_>,
        txn: &SignatureCheckedTransaction,
    ) -> TransactionOutput {
        let txn_data = TransactionMetadata::new(txn);
        let account_currency_symbol =
            account_config::from_currency_code_string(txn.gas_currency_code())
                .map_err(|_| VMStatus::new(StatusCode::INVALID_GAS_SPECIFIER));
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
        remote_cache: &StateViewCache<'_>,
        write_set: &WriteSet,
    ) -> VMResult<()> {
        // All Move executions satisfy the read-before-write property. Thus we need to read each
        // access path that the write set is going to update.
        for (ap, _) in write_set.iter() {
            remote_cache
                .get(ap)
                .map_err(|_| VMStatus::new(StatusCode::STORAGE_ERROR))?;
        }
        Ok(())
    }

    fn process_waypoint_change_set(
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
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
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
        block_metadata: BlockMetadata,
    ) -> VMResult<TransactionOutput> {
        // TODO: How should we setup the metadata here? A couple of thoughts here:
        // 1. We might make the txn_data to be poisoned so that reading anything will result in a panic.
        // 2. The most important consideration is figuring out the sender address.  Having a notion of a
        //    "null address" (probably 0x0...0) that is prohibited from containing modules or resources
        //    might be useful here.
        // 3. We set the max gas to a big number just to get rid of the potential out of gas error.
        let mut txn_data = TransactionMetadata::default();
        txn_data.sender = account_config::reserved_vm_address();
        txn_data.max_gas_amount = GasUnits::new(std::u64::MAX);

        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::transaction(&gas_schedule, txn_data.max_gas_amount());
        cost_strategy.charge_intrinsic_gas(txn_data.transaction_size())?;
        let mut session = self.move_vm.new_session(remote_cache);

        if let Ok((round, timestamp, previous_vote, proposer)) = block_metadata.into_inner() {
            let args = vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::u64(round),
                Value::u64(timestamp),
                Value::vector_address(previous_vote),
                Value::address(proposer),
            ];
            session.execute_function(
                &LIBRA_BLOCK_MODULE,
                &BLOCK_PROLOGUE,
                vec![],
                args,
                txn_data.sender,
                &mut cost_strategy,
            )?
        } else {
            return Err(VMStatus::new(StatusCode::MALFORMED));
        };

        get_transaction_output(
            &mut self.ap_cache,
            session,
            &cost_strategy,
            &txn_data,
            VMStatus::new(StatusCode::EXECUTED),
        )
        .map(|output| {
            remote_cache.push_write_set(output.write_set());
            output
        })
    }

    fn process_writeset_transaction(
        &mut self,
        remote_cache: &mut StateViewCache<'_>,
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

        let mut session = self.move_vm.new_session(remote_cache);

        // Bump the sequence number of sender.
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        session.execute_function(
            &account_config::ACCOUNT_MODULE,
            &BUMP_SEQUENCE_NUMBER_NAME,
            vec![],
            vec![Value::transaction_argument_signer_reference(
                txn_data.sender,
            )],
            txn_data.sender,
            &mut cost_strategy,
        )?;

        // Emit the reconfiguration event
        self.run_writeset_epilogue(&mut session, change_set, &txn_data)?;

        if let Err(e) = self.read_writeset(remote_cache, &change_set.write_set()) {
            return Ok(discard_error_output(e));
        };

        let effects = session.finish()?;
        let (epilogue_writeset, epilogue_events) =
            txn_effects_to_writeset_and_events_cached(&mut self.ap_cache, effects)?;

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
    fn run_prologue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
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
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                &PROLOGUE_NAME,
                vec![gas_currency_ty],
                vec![
                    Value::transaction_argument_signer_reference(txn_data.sender),
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                    Value::u64(txn_gas_price),
                    Value::u64(txn_max_gas_units),
                    Value::u64(txn_expiration_time),
                ],
                txn_data.sender,
                cost_strategy,
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    fn run_success_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session.execute_function(
            &account_config::ACCOUNT_MODULE,
            &SUCCESS_EPILOGUE_NAME,
            vec![gas_currency_ty],
            vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::u64(txn_sequence_number),
                Value::u64(txn_gas_price),
                Value::u64(txn_max_gas_units),
                Value::u64(gas_remaining),
            ],
            txn_data.sender,
            cost_strategy,
        )
    }

    /// Run the failure epilogue of a transaction by calling into `FAILURE_EPILOGUE_NAME` function
    /// stored in the `ACCOUNT_MODULE` on chain.
    fn run_failure_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> VMResult<()> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session.execute_function(
            &account_config::ACCOUNT_MODULE,
            &FAILURE_EPILOGUE_NAME,
            vec![gas_currency_ty],
            vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::u64(txn_sequence_number),
                Value::u64(txn_gas_price),
                Value::u64(txn_max_gas_units),
                Value::u64(gas_remaining),
            ],
            txn_data.sender,
            cost_strategy,
        )
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    fn run_writeset_prologue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));
        let _timer = TXN_PROLOGUE_SECONDS.start_timer();
        session
            .execute_function(
                &LIBRA_WRITESET_MANAGER_MODULE,
                &PROLOGUE_NAME,
                vec![],
                vec![
                    Value::transaction_argument_signer_reference(txn_data.sender),
                    Value::u64(txn_sequence_number),
                    Value::vector_u8(txn_public_key),
                ],
                txn_data.sender,
                &mut cost_strategy,
            )
            .map_err(|err| convert_prologue_runtime_error(&err, &txn_data.sender))
    }

    /// Run the epilogue of a transaction by calling into `WRITESET_EPILOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    fn run_writeset_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        change_set: &ChangeSet,
        txn_data: &TransactionMetadata,
    ) -> VMResult<()> {
        let change_set_bytes =
            lcs::to_bytes(change_set).map_err(|_| VMStatus::new(StatusCode::INVALID_DATA))?;
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session.execute_function(
            &LIBRA_WRITESET_MANAGER_MODULE,
            &WRITESET_EPILOGUE_NAME,
            vec![],
            vec![
                Value::transaction_argument_signer_reference(txn_data.sender),
                Value::vector_u8(change_set_bytes),
            ],
            txn_data.sender,
            &mut cost_strategy,
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
        let mut data_cache = StateViewCache::new(state_view);
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
        data_cache: &mut StateViewCache<'_>,
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
        let data_cache = StateViewCache::new(state_view);
        let _timer = TXN_VALIDATION_SECONDS.start_timer();
        let gas_price = transaction.gas_unit_price();
        let currency_code =
            match account_config::from_currency_code_string(transaction.gas_currency_code()) {
                Ok(code) => code,
                Err(_) => {
                    return VMValidatorResult::new(
                        Some(VMStatus::new(StatusCode::INVALID_GAS_SPECIFIER)),
                        gas_price,
                        false,
                    )
                }
            };

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

        let is_prioritized_txn = is_prioritized_txn(txn_sender, &data_cache);
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

        VMValidatorResult::new(res, normalized_gas_price, is_prioritized_txn)
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
    /// The `TransactionDataCache` can be used as a `ChainState`.
    ///
    /// If you don't care about the transaction metadata, use `TransactionMetadata::default()`.
    pub fn with_txn_data_cache<T, S: StateView>(
        self,
        state_view: &S,
        f: impl for<'txn, 'r> FnOnce(Session<'txn, 'r, RemoteStorage<S>>) -> T,
    ) -> T {
        let remote_storage = RemoteStorage::new(state_view);
        let session = self.move_vm().new_session(&remote_storage);
        f(session)
    }
}

fn is_prioritized_txn(sender: AccountAddress, remote_cache: &StateViewCache) -> bool {
    let role_access_path = create_access_path(sender, RoleId::struct_tag());
    if let Ok(Some(blob)) = remote_cache.get(&role_access_path) {
        return lcs::from_bytes::<account_config::RoleId>(&blob)
            .map(|role_id| role_id.role_id() < PRIORITIZED_TRANSACTION_ROLE_CUTOFF)
            .unwrap_or(false);
    }
    false
}

fn can_publish_modules(sender: AccountAddress, remote_cache: &StateViewCache) -> bool {
    let module_publishing_priv_path =
        create_access_path(sender, module_publishing_capability_struct_tag());
    match remote_cache.get(&module_publishing_priv_path) {
        Ok(Some(_)) => true,
        _ => false,
    }
}

fn normalize_gas_price(
    gas_price: u64,
    currency_code: &IdentStr,
    remote_cache: &StateViewCache,
) -> VMResult<u64> {
    let currency_info_path =
        account_config::CurrencyInfoResource::resource_path_for(currency_code.to_owned());
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
            TransactionArgument::U8(i) => Value::u8(*i),
            TransactionArgument::U64(i) => Value::u64(*i),
            TransactionArgument::U128(i) => Value::u128(*i),
            TransactionArgument::Address(a) => Value::address(*a),
            TransactionArgument::Bool(b) => Value::bool(*b),
            TransactionArgument::U8Vector(v) => Value::vector_u8(v.clone()),
        })
        .collect()
}

pub trait AccessPathCache {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath;
    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath;
}

impl AccessPathCache for () {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath {
        AccessPath::from(&module_id)
    }

    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath {
        AccessPath::new(address, struct_tag.access_vector())
    }
}

#[derive(Clone)]
pub struct BTreeAccessPathCache {
    modules: BTreeMap<ModuleId, Vec<u8>>,
    resources: BTreeMap<StructTag, Vec<u8>>,
}

impl AccessPathCache for BTreeAccessPathCache {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath {
        let addr = *module_id.address();
        let access_vec = match self.modules.entry(module_id) {
            btree_map::Entry::Vacant(entry) => {
                let v = entry.key().access_vector();
                entry.insert(v).clone()
            }
            btree_map::Entry::Occupied(entry) => entry.get().clone(),
        };
        AccessPath::new(addr, access_vec)
    }

    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath {
        let access_vec = match self.resources.entry(struct_tag) {
            btree_map::Entry::Vacant(entry) => {
                let v = entry.key().access_vector();
                entry.insert(v).clone()
            }
            btree_map::Entry::Occupied(entry) => entry.get().clone(),
        };
        AccessPath::new(address, access_vec)
    }
}

impl BTreeAccessPathCache {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            resources: BTreeMap::new(),
        }
    }
}

pub fn txn_effects_to_writeset_and_events_cached<C: AccessPathCache>(
    ap_cache: &mut C,
    effects: TransactionEffects,
) -> VMResult<(WriteSet, Vec<ContractEvent>)> {
    //println!("effects");
    // TODO: Cache access path computations if necessary.
    let mut ops = vec![];

    for (addr, vals) in effects.resources {
        for (ty_tag, val_opt) in vals {
            let struct_tag = match ty_tag {
                TypeTag::Struct(struct_tag) => struct_tag,
                _ => return Err(VMStatus::new(StatusCode::VALUE_SERIALIZATION_ERROR)),
            };
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match val_opt {
                None => WriteOp::Deletion,
                Some((ty_layout, val)) => {
                    let blob = val
                        .simple_serialize(&ty_layout)
                        .ok_or_else(|| VMStatus::new(StatusCode::VALUE_SERIALIZATION_ERROR))?;

                    WriteOp::Value(blob)
                }
            };
            ops.push((ap, op))
        }
    }

    for (module_id, blob) in effects.modules {
        ops.push((ap_cache.get_module_path(module_id), WriteOp::Value(blob)))
    }

    let ws = WriteSetMut::new(ops)
        .freeze()
        .map_err(|_| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;

    let events = effects
        .events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, ty_layout, val)| {
            let msg = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| VMStatus::new(StatusCode::DATA_FORMAT_ERROR))?;
            let key = EventKey::try_from(guid.as_slice())
                .map_err(|_| VMStatus::new(StatusCode::EVENT_KEY_MISMATCH))?;
            Ok(ContractEvent::new(key, seq_num, ty_tag, msg))
        })
        .collect::<VMResult<Vec<_>>>()?;

    Ok((ws, events))
}

fn get_transaction_output<A: AccessPathCache, R: RemoteCache>(
    ap_cache: &mut A,
    session: Session<R>,
    cost_strategy: &CostStrategy,
    txn_data: &TransactionMetadata,
    status: VMStatus,
) -> VMResult<TransactionOutput> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(cost_strategy.remaining_gas())
        .get();

    let effects = session.finish()?;
    let (write_set, events) = txn_effects_to_writeset_and_events_cached(ap_cache, effects)?;

    TXN_TOTAL_GAS_USAGE.observe(gas_used as f64);
    Ok(TransactionOutput::new(
        write_set,
        events,
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

pub fn txn_effects_to_writeset_and_events(
    effects: TransactionEffects,
) -> VMResult<(WriteSet, Vec<ContractEvent>)> {
    txn_effects_to_writeset_and_events_cached(&mut (), effects)
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
