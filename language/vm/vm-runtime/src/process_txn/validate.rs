// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::VMRuntime;
use crate::txn_executor::TransactionExecutor;
use crate::{data_cache::RemoteCache, process_txn::verify::VerifiedTransaction};
use libra_config::config::VMPublishingOption;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    transaction::{SignatureCheckedTransaction, TransactionPayload, MAX_TRANSACTION_SIZE_IN_BYTES},
    vm_error::{StatusCode, VMStatus},
};
use vm::{
    errors::convert_prologue_runtime_error,
    gas_schedule::{self, AbstractMemorySize, CostTable, GasAlgebra, GasCarrier},
    transaction_metadata::TransactionMetadata,
};

pub fn is_allowed_script(publishing_option: &VMPublishingOption, program: &[u8]) -> bool {
    match publishing_option {
        VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
        VMPublishingOption::Locked(whitelist) => {
            let hash_value = HashValue::from_sha3_256(program);
            whitelist.contains(hash_value.as_ref())
        }
    }
}

/// Represents a [`SignedTransaction`] that has been *validated*. This includes all the steps
/// required to ensure that a transaction is valid, other than verifying the submitted program.
pub struct ValidatedTransaction {
    txn: SignatureCheckedTransaction,
}

/// The mode to validate transactions in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// This is the genesis transaction. At the moment it is the only mode that allows for
    /// write-set transactions.
    Genesis,
    /// We're only validating a transaction, not executing it. This tolerates the sequence number
    /// being too new.
    Validating,
    /// We're executing a transaction. This runs the full suite of checks.
    #[allow(dead_code)]
    Executing,
}

impl ValidatedTransaction {
    /// Creates a new instance by validating a `SignedTransaction`.
    ///
    /// This should be called through [`ProcessTransaction::validate`].
    pub(crate) fn new(
        txn: SignatureCheckedTransaction,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        gas_schedule: &CostTable,
        data_cache: &dyn RemoteCache,
        mode: ValidationMode,
    ) -> Result<Self, VMStatus> {
        match txn.payload() {
            TransactionPayload::Program => return Err(VMStatus::new(StatusCode::MALFORMED)),
            TransactionPayload::Script(script) => {
                ValidatedTransaction::validate(
                    &txn,
                    runtime,
                    state_view,
                    gas_schedule,
                    data_cache,
                    mode,
                    || {
                        // Verify against whitelist if we are locked. Otherwise allow.
                        if !is_allowed_script(&runtime.publishing_option(), &script.code()) {
                            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
                            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
                        }
                        Ok(())
                    },
                )?
            }
            TransactionPayload::Module(module) => {
                debug!("validate module {:?}", module);
                ValidatedTransaction::validate(
                    &txn,
                    runtime,
                    state_view,
                    gas_schedule,
                    data_cache,
                    mode,
                    || {
                        if !runtime.publishing_option().is_open() {
                            warn!("[VM] Custom modules not allowed");
                            Err(VMStatus::new(StatusCode::UNKNOWN_MODULE))
                        } else {
                            Ok(())
                        }
                    },
                )?
            }
            TransactionPayload::WriteSet(write_set_payload) => {
                // The only acceptable write-set transaction for now is for the genesis
                // transaction.
                // XXX figure out a story for hard forks.
                if mode != ValidationMode::Genesis {
                    warn!("[VM] Attempt to process genesis after initialization");
                    return Err(VMStatus::new(StatusCode::REJECTED_WRITE_SET));
                }

                for (_access_path, write_op) in write_set_payload.write_set() {
                    // Genesis transactions only add entries, never delete them.
                    if write_op.is_deletion() {
                        error!("[VM] Bad genesis block");
                        // TODO: return more detailed error somehow?
                        return Err(VMStatus::new(StatusCode::INVALID_WRITE_SET));
                    }
                }
            }
        }

        Ok(Self { txn })
    }

    /// Verifies the bytecode in this transaction.
    pub fn verify(self, txn_data: &TransactionMetadata) -> Result<VerifiedTransaction, VMStatus> {
        VerifiedTransaction::new(self, txn_data)
    }

    /// Consumes `self` and returns the `SignatureCheckedTransaction` within.
    pub fn into_inner(self) -> SignatureCheckedTransaction {
        self.txn
    }

    pub fn as_inner(&self) -> &SignatureCheckedTransaction {
        &self.txn
    }

    fn validate(
        txn: &SignatureCheckedTransaction,
        runtime: &VMRuntime,
        state_view: &dyn StateView,
        gas_schedule: &CostTable,
        data_cache: &dyn RemoteCache,
        mode: ValidationMode,
        payload_check: impl Fn() -> Result<(), VMStatus>,
    ) -> Result<(), VMStatus> {
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

        payload_check()?;

        let txn_data = TransactionMetadata::new(&txn);

        // Run the prologue to ensure that clients have enough gas and aren't tricking us by
        // sending us garbage.
        // TODO: write-set transactions (other than genesis??) should also run the prologue.
        let mut txn_executor = TransactionExecutor::new(gas_schedule, data_cache, txn_data);
        txn_executor
            .run_prologue(runtime, state_view)
            .or_else(|err| {
                let vm_status = convert_prologue_runtime_error(&err, &txn.sender());

                // In validating mode, accept transactions with sequence number greater
                // or equal to the current sequence number.
                match (mode, vm_status.major_status) {
                    (ValidationMode::Validating, StatusCode::SEQUENCE_NUMBER_TOO_NEW) => {
                        trace!("[VM] Sequence number too new error ignored");
                        Ok(())
                    }
                    (_, _) => {
                        warn!("[VM] Error in prologue: {:?}", err);
                        Err(vm_status)
                    }
                }
            })
    }
}
