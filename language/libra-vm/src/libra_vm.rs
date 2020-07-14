// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path_cache::AccessPathCache,
    counters::*,
    data_cache::{RemoteStorage, StateViewCache},
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
};

use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    account_config,
    contract_event::ContractEvent,
    event::EventKey,
    on_chain_config::{ConfigStorage, LibraVersion, OnChainConfig, VMConfig},
    transaction::{ChangeSet, Script, TransactionOutput, TransactionStatus},
    vm_status::{convert_prologue_runtime_error, KeptVMStatus, StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use move_core_types::{
    gas_schedule::{CostTable, GasAlgebra, GasUnits},
    identifier::IdentStr,
    language_storage::TypeTag,
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
use std::{convert::TryFrom, sync::Arc};
use vm::errors::Location;

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct LibraVMImpl {
    move_vm: Arc<MoveVM>,
    on_chain_config: Option<VMConfig>,
    version: Option<LibraVersion>,
}

impl LibraVMImpl {
    #[allow(clippy::new_without_default)]
    pub fn new<S: StateView>(state: &S) -> Self {
        let inner = MoveVM::new();
        let mut vm = Self {
            move_vm: Arc::new(inner),
            on_chain_config: None,
            version: None,
        };
        vm.load_configs_impl(&RemoteStorage::new(state));
        vm
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

    pub(crate) fn on_chain_config(&self) -> Result<&VMConfig, VMStatus> {
        self.on_chain_config.as_ref().ok_or_else(|| {
            error!("VM Startup Failed. On Chain Config Not Found");
            VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
        })
    }

    fn load_configs_impl<S: ConfigStorage>(&mut self, data_cache: &S) {
        self.on_chain_config = VMConfig::fetch_config(data_cache);
        self.version = LibraVersion::fetch_config(data_cache);
    }

    pub fn get_gas_schedule(&self) -> Result<&CostTable, VMStatus> {
        self.on_chain_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| {
                error!("VM Startup Failed. Gas Schedule Not Found");
                VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
            })
    }

    pub fn get_libra_version(&self) -> Result<LibraVersion, VMStatus> {
        self.version.clone().ok_or_else(|| {
            error!("VM Startup Failed. Libra Version Not Found");
            VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
        })
    }

    pub fn check_gas(&self, txn_data: &TransactionMetadata) -> Result<(), VMStatus> {
        let gas_constants = &self.get_gas_schedule()?.gas_constants;
        let raw_bytes_len = txn_data.transaction_size;
        // The transaction is too large.
        if txn_data.transaction_size.get() > gas_constants.max_transaction_size_in_bytes {
            warn!(
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                gas_constants.max_transaction_size_in_bytes
            );
            return Err(VMStatus::Error(StatusCode::EXCEEDED_MAX_TRANSACTION_SIZE));
        }

        // Check is performed on `txn.raw_txn_bytes_len()` which is the same as
        // `raw_bytes_len`
        assume!(raw_bytes_len.get() <= gas_constants.max_transaction_size_in_bytes);

        // The submitted max gas units that the transaction can consume is greater than the
        // maximum number of gas units bound that we have set for any
        // transaction.
        if txn_data.max_gas_amount().get() > gas_constants.maximum_number_of_gas_units.get() {
            warn!(
                "[VM] Gas unit error; max {}, submitted {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn_data.max_gas_amount().get()
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
            ));
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee = calculate_intrinsic_gas(raw_bytes_len, gas_constants);
        if txn_data.max_gas_amount().get() < min_txn_fee.get() {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn_data.max_gas_amount().get()
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS,
            ));
        }

        // The submitted gas price is less than the minimum gas unit price set by the VM.
        // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
        // we turn off the clippy warning.
        #[allow(clippy::absurd_extreme_comparisons)]
        let below_min_bound =
            txn_data.gas_unit_price().get() < gas_constants.min_price_per_gas_unit.get();
        if below_min_bound {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND));
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn_data.gas_unit_price().get() > gas_constants.max_price_per_gas_unit.get() {
            warn!(
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get()
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND));
        }
        Ok(())
    }

    pub(crate) fn is_allowed_script(&self, script: &Script) -> Result<(), VMStatus> {
        if !self
            .on_chain_config()?
            .publishing_option
            .is_allowed_script(&script.code())
        {
            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
            Err(VMStatus::Error(StatusCode::UNKNOWN_SCRIPT))
        } else {
            Ok(())
        }
    }

    pub(crate) fn is_allowed_module(
        &self,
        txn_data: &TransactionMetadata,
        _remote_cache: &StateViewCache,
    ) -> Result<(), VMStatus> {
        if !&self
            .on_chain_config()?
            .publishing_option
            .is_allowed_module(&txn_data.sender)
        {
            warn!("[VM] Custom modules not allowed");
            Err(VMStatus::Error(StatusCode::INVALID_MODULE_PUBLISHER))
        } else {
            Ok(())
        }
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_prologue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_time = txn_data.expiration_time();
        let chain_id = txn_data.chain_id();
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
                    Value::u8(chain_id.id()),
                ],
                txn_data.sender,
                cost_strategy,
            )
            .map_err(|e| convert_prologue_runtime_error(e.into_vm_status()))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_success_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session
            .execute_function(
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
            .map_err(|e| e.into_vm_status())
    }

    /// Run the failure epilogue of a transaction by calling into `FAILURE_EPILOGUE_NAME` function
    /// stored in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_failure_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        cost_strategy: &mut CostStrategy,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = cost_strategy.remaining_gas().get();
        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session
            .execute_function(
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
            .map_err(|e| e.into_vm_status())
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    pub(crate) fn run_writeset_prologue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
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
            .map_err(|e| convert_prologue_runtime_error(e.into_vm_status()))
    }

    /// Run the epilogue of a transaction by calling into `WRITESET_EPILOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    pub(crate) fn run_writeset_epilogue<R: RemoteCache>(
        &self,
        session: &mut Session<R>,
        change_set: &ChangeSet,
        txn_data: &TransactionMetadata,
    ) -> Result<(), VMStatus> {
        let change_set_bytes =
            lcs::to_bytes(change_set).map_err(|_| VMStatus::Error(StatusCode::INVALID_DATA))?;
        let gas_schedule = zero_cost_schedule();
        let mut cost_strategy = CostStrategy::system(&gas_schedule, GasUnits::new(0));

        let _timer = TXN_EPILOGUE_SECONDS.start_timer();
        session
            .execute_function(
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
            .map_err(|e| e.into_vm_status())
    }

    pub fn new_session<'r, R: RemoteCache>(&self, r: &'r R) -> Session<'r, '_, R> {
        self.move_vm.new_session(r)
    }
}

/// Internal APIs for the Libra VM, primarily used for testing.
#[derive(Clone, Copy)]
pub struct LibraVMInternals<'a>(&'a LibraVMImpl);

impl<'a> LibraVMInternals<'a> {
    pub fn new(internal: &'a LibraVMImpl) -> Self {
        Self(internal)
    }

    /// Returns the internal Move VM instance.
    pub fn move_vm(self) -> &'a MoveVM {
        &self.0.move_vm
    }

    /// Returns the internal gas schedule if it has been loaded, or an error if it hasn't.
    pub fn gas_schedule(self) -> Result<&'a CostTable, VMStatus> {
        self.0.get_gas_schedule()
    }

    /// Returns the version of Move Runtime.
    pub fn libra_version(self) -> Result<LibraVersion, VMStatus> {
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

pub fn txn_effects_to_writeset_and_events_cached<C: AccessPathCache>(
    ap_cache: &mut C,
    effects: TransactionEffects,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    // TODO: Cache access path computations if necessary.
    let mut ops = vec![];

    for (addr, vals) in effects.resources {
        for (ty_tag, val_opt) in vals {
            let struct_tag = match ty_tag {
                TypeTag::Struct(struct_tag) => struct_tag,
                _ => return Err(VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR)),
            };
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match val_opt {
                None => WriteOp::Deletion,
                Some((ty_layout, val)) => {
                    let blob = val
                        .simple_serialize(&ty_layout)
                        .ok_or_else(|| VMStatus::Error(StatusCode::VALUE_SERIALIZATION_ERROR))?;

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
        .map_err(|_| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;

    let events = effects
        .events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, ty_layout, val)| {
            let msg = val
                .simple_serialize(&ty_layout)
                .ok_or_else(|| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;
            let key = EventKey::try_from(guid.as_slice())
                .map_err(|_| VMStatus::Error(StatusCode::EVENT_KEY_MISMATCH))?;
            Ok(ContractEvent::new(key, seq_num, ty_tag, msg))
        })
        .collect::<Result<Vec<_>, VMStatus>>()?;

    Ok((ws, events))
}

pub(crate) fn charge_global_write_gas_usage<R: RemoteCache>(
    cost_strategy: &mut CostStrategy,
    session: &Session<R>,
) -> Result<(), VMStatus> {
    let total_cost = session.num_mutated_accounts()
        * cost_strategy
            .cost_table()
            .gas_constants
            .global_memory_per_byte_write_cost
            .mul(
                cost_strategy
                    .cost_table()
                    .gas_constants
                    .default_account_size,
            )
            .get();
    cost_strategy
        .deduct_gas(GasUnits::new(total_cost))
        .map_err(|p_err| p_err.finish(Location::Undefined).into_vm_status())
}

pub(crate) fn get_transaction_output<A: AccessPathCache, R: RemoteCache>(
    ap_cache: &mut A,
    session: Session<R>,
    cost_strategy: &CostStrategy,
    txn_data: &TransactionMetadata,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = txn_data
        .max_gas_amount()
        .sub(cost_strategy.remaining_gas())
        .get();

    let effects = session.finish().map_err(|e| e.into_vm_status())?;
    let (write_set, events) = txn_effects_to_writeset_and_events_cached(ap_cache, effects)?;

    Ok(TransactionOutput::new(
        write_set,
        events,
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

pub fn txn_effects_to_writeset_and_events(
    effects: TransactionEffects,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    txn_effects_to_writeset_and_events_cached(&mut (), effects)
}

#[test]
fn vm_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    use crate::{LibraVM, LibraVMValidator};

    assert_send::<LibraVM>();
    assert_sync::<LibraVM>();
    assert_send::<LibraVMValidator>();
    assert_sync::<LibraVMValidator>();
    assert_send::<MoveVM>();
    assert_sync::<MoveVM>();
}
