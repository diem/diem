// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path_cache::AccessPathCache,
    counters::*,
    data_cache::RemoteStorage,
    errors::{convert_epilogue_error, convert_prologue_error, expect_only_successful_execution},
    logging::AdapterLogSchema,
    natives::diem_natives,
    system_module_names::*,
    transaction_metadata::TransactionMetadata,
};
use diem_crypto::HashValue;
use diem_logger::prelude::*;
use diem_state_view::StateView;
use diem_types::{
    account_config,
    contract_event::ContractEvent,
    event::EventKey,
    on_chain_config::{
        ConfigStorage, DiemVersion, OnChainConfig, VMConfig, VMPublishingOption, DIEM_VERSION_3,
    },
    transaction::{TransactionOutput, TransactionStatus},
    vm_status::{KeptVMStatus, StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use fail::fail_point;
use move_binary_format::errors::Location;
use move_core_types::{
    account_address::AccountAddress,
    effects::{ChangeSet as MoveChangeSet, Event as MoveEvent},
    gas_schedule::{CostTable, GasAlgebra, GasCarrier, GasUnits, InternalGasUnits},
    identifier::IdentStr,
    language_storage::ModuleId,
    resolver::MoveResolver,
    value::{serialize_values, MoveValue},
};
use move_vm_runtime::{logging::expect_no_verification_errors, move_vm::MoveVM, session::Session};
use move_vm_types::gas_schedule::{calculate_intrinsic_gas, GasStatus};
use std::{convert::TryFrom, sync::Arc};

#[derive(Clone)]
/// A wrapper to make VMRuntime standalone and thread safe.
pub struct DiemVMImpl {
    move_vm: Arc<MoveVM>,
    on_chain_config: Option<VMConfig>,
    version: Option<DiemVersion>,
    publishing_option: Option<VMPublishingOption>,
}

impl DiemVMImpl {
    #[allow(clippy::new_without_default)]
    pub fn new<S: StateView>(state: &S) -> Self {
        let inner = MoveVM::new(diem_natives())
            .expect("should be able to create Move VM; check if there are duplicated natives");
        let mut vm = Self {
            move_vm: Arc::new(inner),
            on_chain_config: None,
            version: None,
            publishing_option: None,
        };
        vm.load_configs_impl(&RemoteStorage::new(state));
        vm
    }

    pub fn init_with_config(
        version: DiemVersion,
        on_chain_config: VMConfig,
        publishing_option: VMPublishingOption,
    ) -> Self {
        let inner = MoveVM::new(diem_natives())
            .expect("should be able to create Move VM; check if there are duplicated natives");
        Self {
            move_vm: Arc::new(inner),
            on_chain_config: Some(on_chain_config),
            version: Some(version),
            publishing_option: Some(publishing_option),
        }
    }

    /// Provides access to some internal APIs of the Diem VM.
    pub fn internals(&self) -> DiemVMInternals {
        DiemVMInternals(self)
    }

    pub(crate) fn publishing_option(
        &self,
        log_context: &AdapterLogSchema,
    ) -> Result<&VMPublishingOption, VMStatus> {
        self.publishing_option.as_ref().ok_or_else(|| {
            log_context.alert();
            error!(
                *log_context,
                "VM Startup Failed. PublishingOption Not Found"
            );
            VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
        })
    }

    fn load_configs_impl<S: ConfigStorage>(&mut self, data_cache: &S) {
        self.on_chain_config = VMConfig::fetch_config(data_cache);
        self.version = DiemVersion::fetch_config(data_cache);
        self.publishing_option = VMPublishingOption::fetch_config(data_cache);
    }

    pub fn get_gas_schedule(&self, log_context: &AdapterLogSchema) -> Result<&CostTable, VMStatus> {
        self.on_chain_config
            .as_ref()
            .map(|config| &config.gas_schedule)
            .ok_or_else(|| {
                log_context.alert();
                error!(*log_context, "VM Startup Failed. Gas Schedule Not Found");
                VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
            })
    }

    pub fn get_diem_version(&self) -> Result<DiemVersion, VMStatus> {
        self.version.clone().ok_or_else(|| {
            CRITICAL_ERRORS.inc();
            error!("VM Startup Failed. Diem Version Not Found");
            VMStatus::Error(StatusCode::VM_STARTUP_FAILURE)
        })
    }

    pub fn check_gas(
        &self,
        txn_data: &TransactionMetadata,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let gas_constants = &self.get_gas_schedule(log_context)?.gas_constants;
        let raw_bytes_len = txn_data.transaction_size;
        // The transaction is too large.
        if txn_data.transaction_size.get() > gas_constants.max_transaction_size_in_bytes {
            warn!(
                *log_context,
                "[VM] Transaction size too big {} (max {})",
                raw_bytes_len.get(),
                gas_constants.max_transaction_size_in_bytes,
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
                *log_context,
                "[VM] Gas unit error; max {}, submitted {}",
                gas_constants.maximum_number_of_gas_units.get(),
                txn_data.max_gas_amount().get(),
            );
            return Err(VMStatus::Error(
                StatusCode::MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND,
            ));
        }

        // The submitted transactions max gas units needs to be at least enough to cover the
        // intrinsic cost of the transaction as calculated against the size of the
        // underlying `RawTransaction`
        let min_txn_fee =
            gas_constants.to_external_units(calculate_intrinsic_gas(raw_bytes_len, gas_constants));
        if txn_data.max_gas_amount().get() < min_txn_fee.get() {
            warn!(
                *log_context,
                "[VM] Gas unit error; min {}, submitted {}",
                min_txn_fee.get(),
                txn_data.max_gas_amount().get(),
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
                *log_context,
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.min_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get(),
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_BELOW_MIN_BOUND));
        }

        // The submitted gas price is greater than the maximum gas unit price set by the VM.
        if txn_data.gas_unit_price().get() > gas_constants.max_price_per_gas_unit.get() {
            warn!(
                *log_context,
                "[VM] Gas unit error; min {}, submitted {}",
                gas_constants.max_price_per_gas_unit.get(),
                txn_data.gas_unit_price().get(),
            );
            return Err(VMStatus::Error(StatusCode::GAS_UNIT_PRICE_ABOVE_MAX_BOUND));
        }
        Ok(())
    }

    /// Run the prologue of a transaction by calling into either `SCRIPT_PROLOGUE_NAME` function
    /// or `MULTI_AGENT_SCRIPT_PROLOGUE_NAME` function stored in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_script_prologue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_timestamp_secs = txn_data.expiration_timestamp_secs();
        let chain_id = txn_data.chain_id();
        let mut gas_status = GasStatus::new_unmetered();
        let secondary_public_key_hashes: Vec<MoveValue> = txn_data
            .secondary_authentication_key_preimages
            .iter()
            .map(|preimage| MoveValue::vector_u8(HashValue::sha3_256_of(preimage).to_vec()))
            .collect();
        let args = if self.get_diem_version()? >= DIEM_VERSION_3 && txn_data.is_multi_agent() {
            vec![
                MoveValue::Signer(txn_data.sender),
                MoveValue::U64(txn_sequence_number),
                MoveValue::vector_u8(txn_public_key),
                MoveValue::vector_address(txn_data.secondary_signers()),
                MoveValue::Vector(secondary_public_key_hashes),
                MoveValue::U64(txn_gas_price),
                MoveValue::U64(txn_max_gas_units),
                MoveValue::U64(txn_expiration_timestamp_secs),
                MoveValue::U8(chain_id.id()),
            ]
        } else {
            vec![
                MoveValue::Signer(txn_data.sender),
                MoveValue::U64(txn_sequence_number),
                MoveValue::vector_u8(txn_public_key),
                MoveValue::U64(txn_gas_price),
                MoveValue::U64(txn_max_gas_units),
                MoveValue::U64(txn_expiration_timestamp_secs),
                MoveValue::U8(chain_id.id()),
                MoveValue::vector_u8(txn_data.script_hash.clone()),
            ]
        };
        let prologue_function_name =
            if self.get_diem_version()? >= DIEM_VERSION_3 && txn_data.is_multi_agent() {
                &MULTI_AGENT_SCRIPT_PROLOGUE_NAME
            } else {
                &SCRIPT_PROLOGUE_NAME
            };
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                prologue_function_name,
                vec![gas_currency_ty],
                serialize_values(&args),
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|err| convert_prologue_error(err, log_context))
    }

    /// Run the prologue of a transaction by calling into `MODULE_PROLOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_module_prologue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let txn_expiration_timestamp_secs = txn_data.expiration_timestamp_secs();
        let chain_id = txn_data.chain_id();
        let mut gas_status = GasStatus::new_unmetered();
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                MODULE_PROLOGUE_NAME,
                vec![gas_currency_ty],
                serialize_values(&vec![
                    MoveValue::Signer(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::vector_u8(txn_public_key),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_units),
                    MoveValue::U64(txn_expiration_timestamp_secs),
                    MoveValue::U8(chain_id.id()),
                ]),
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|err| convert_prologue_error(err, log_context))
    }

    /// Run the epilogue of a transaction by calling into `EPILOGUE_NAME` function stored
    /// in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_success_epilogue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        fail_point!("move_adapter::run_success_epilogue", |_| {
            Err(VMStatus::Error(
                StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR,
            ))
        });

        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = gas_status.remaining_gas().get();
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                USER_EPILOGUE_NAME,
                vec![gas_currency_ty],
                serialize_values(&vec![
                    MoveValue::Signer(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_units),
                    MoveValue::U64(gas_remaining),
                ]),
                gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|err| convert_epilogue_error(err, log_context))
    }

    /// Run the failure epilogue of a transaction by calling into `USER_EPILOGUE_NAME` function
    /// stored in the `ACCOUNT_MODULE` on chain.
    pub(crate) fn run_failure_epilogue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        gas_status: &mut GasStatus,
        txn_data: &TransactionMetadata,
        account_currency_symbol: &IdentStr,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let gas_currency_ty =
            account_config::type_tag_for_currency_code(account_currency_symbol.to_owned());
        let txn_sequence_number = txn_data.sequence_number();
        let txn_gas_price = txn_data.gas_unit_price().get();
        let txn_max_gas_units = txn_data.max_gas_amount().get();
        let gas_remaining = gas_status.remaining_gas().get();
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                USER_EPILOGUE_NAME,
                vec![gas_currency_ty],
                serialize_values(&vec![
                    MoveValue::Signer(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::U64(txn_gas_price),
                    MoveValue::U64(txn_max_gas_units),
                    MoveValue::U64(gas_remaining),
                ]),
                gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|e| {
                expect_only_successful_execution(e, USER_EPILOGUE_NAME.as_str(), log_context)
            })
    }

    /// Run the prologue of a transaction by calling into `PROLOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    pub(crate) fn run_writeset_prologue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        txn_data: &TransactionMetadata,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let txn_sequence_number = txn_data.sequence_number();
        let txn_public_key = txn_data.authentication_key_preimage().to_vec();
        let txn_expiration_timestamp_secs = txn_data.expiration_timestamp_secs();
        let chain_id = txn_data.chain_id();

        let mut gas_status = GasStatus::new_unmetered();
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                WRITESET_PROLOGUE_NAME,
                vec![],
                serialize_values(&vec![
                    MoveValue::Signer(txn_data.sender),
                    MoveValue::U64(txn_sequence_number),
                    MoveValue::vector_u8(txn_public_key),
                    MoveValue::U64(txn_expiration_timestamp_secs),
                    MoveValue::U8(chain_id.id()),
                ]),
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|err| convert_prologue_error(err, log_context))
    }

    /// Run the epilogue of a transaction by calling into `WRITESET_EPILOGUE_NAME` function stored
    /// in the `WRITESET_MODULE` on chain.
    pub(crate) fn run_writeset_epilogue<S: MoveResolver>(
        &self,
        session: &mut Session<S>,
        txn_data: &TransactionMetadata,
        should_trigger_reconfiguration: bool,
        log_context: &AdapterLogSchema,
    ) -> Result<(), VMStatus> {
        let mut gas_status = GasStatus::new_unmetered();
        session
            .execute_function(
                &account_config::ACCOUNT_MODULE,
                WRITESET_EPILOGUE_NAME,
                vec![],
                serialize_values(&vec![
                    MoveValue::Signer(txn_data.sender),
                    MoveValue::U64(txn_data.sequence_number),
                    MoveValue::Bool(should_trigger_reconfiguration),
                ]),
                &mut gas_status,
            )
            .map(|_return_vals| ())
            .map_err(expect_no_verification_errors)
            .or_else(|e| {
                expect_only_successful_execution(e, WRITESET_EPILOGUE_NAME.as_str(), log_context)
            })
    }

    pub fn new_session<'r, R: MoveResolver>(&self, r: &'r R) -> Session<'r, '_, R> {
        self.move_vm.new_session(r)
    }
}

/// Internal APIs for the Diem VM, primarily used for testing.
#[derive(Clone, Copy)]
pub struct DiemVMInternals<'a>(&'a DiemVMImpl);

impl<'a> DiemVMInternals<'a> {
    pub fn new(internal: &'a DiemVMImpl) -> Self {
        Self(internal)
    }

    /// Returns the internal Move VM instance.
    pub fn move_vm(self) -> &'a MoveVM {
        &self.0.move_vm
    }

    /// Returns the internal gas schedule if it has been loaded, or an error if it hasn't.
    pub fn gas_schedule(self, log_context: &AdapterLogSchema) -> Result<&'a CostTable, VMStatus> {
        self.0.get_gas_schedule(log_context)
    }

    /// Returns the version of Move Runtime.
    pub fn diem_version(self) -> Result<DiemVersion, VMStatus> {
        self.0.get_diem_version()
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

pub fn convert_changeset_and_events_cached<C: AccessPathCache>(
    ap_cache: &mut C,
    changeset: MoveChangeSet,
    events: Vec<MoveEvent>,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    // TODO: Cache access path computations if necessary.
    let mut ops = vec![];

    for (addr, account_changeset) in changeset.into_inner() {
        let (modules, resources) = account_changeset.into_inner();
        for (struct_tag, blob_opt) in resources {
            let ap = ap_cache.get_resource_path(addr, struct_tag);
            let op = match blob_opt {
                None => WriteOp::Deletion,
                Some(blob) => WriteOp::Value(blob),
            };
            ops.push((ap, op))
        }

        for (name, blob_opt) in modules {
            let ap = ap_cache.get_module_path(ModuleId::new(addr, name));
            let op = match blob_opt {
                None => WriteOp::Deletion,
                Some(blob) => WriteOp::Value(blob),
            };

            ops.push((ap, op))
        }
    }

    let ws = WriteSetMut::new(ops)
        .freeze()
        .map_err(|_| VMStatus::Error(StatusCode::DATA_FORMAT_ERROR))?;

    let events = events
        .into_iter()
        .map(|(guid, seq_num, ty_tag, blob)| {
            let key = EventKey::try_from(guid.as_slice())
                .map_err(|_| VMStatus::Error(StatusCode::EVENT_KEY_MISMATCH))?;
            Ok(ContractEvent::new(key, seq_num, ty_tag, blob))
        })
        .collect::<Result<Vec<_>, VMStatus>>()?;

    Ok((ws, events))
}

pub fn convert_changeset_and_events(
    changeset: MoveChangeSet,
    events: Vec<MoveEvent>,
) -> Result<(WriteSet, Vec<ContractEvent>), VMStatus> {
    convert_changeset_and_events_cached(&mut (), changeset, events)
}

pub(crate) fn charge_global_write_gas_usage<R: MoveResolver>(
    gas_status: &mut GasStatus,
    session: &Session<R>,
    sender: &AccountAddress,
) -> Result<(), VMStatus> {
    let total_cost = session.num_mutated_accounts(sender)
        * gas_status
            .cost_table()
            .gas_constants
            .global_memory_per_byte_write_cost
            .mul(gas_status.cost_table().gas_constants.default_account_size)
            .get();
    gas_status
        .deduct_gas(InternalGasUnits::new(total_cost))
        .map_err(|p_err| p_err.finish(Location::Undefined).into_vm_status())
}

pub(crate) fn get_transaction_output<A: AccessPathCache, S: MoveResolver>(
    ap_cache: &mut A,
    session: Session<S>,
    gas_left: GasUnits<GasCarrier>,
    txn_data: &TransactionMetadata,
    status: KeptVMStatus,
) -> Result<TransactionOutput, VMStatus> {
    let gas_used: u64 = txn_data.max_gas_amount().sub(gas_left).get();

    let (changeset, events) = session.finish().map_err(|e| e.into_vm_status())?;
    let (write_set, events) = convert_changeset_and_events_cached(ap_cache, changeset, events)?;

    Ok(TransactionOutput::new(
        write_set,
        events,
        gas_used,
        TransactionStatus::Keep(status),
    ))
}

#[test]
fn vm_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    use crate::{DiemVM, DiemVMValidator};

    assert_send::<DiemVM>();
    assert_sync::<DiemVM>();
    assert_send::<DiemVMValidator>();
    assert_sync::<DiemVMValidator>();
    assert_send::<MoveVM>();
    assert_sync::<MoveVM>();
}
