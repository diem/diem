// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    code_cache::{module_cache::ModuleCache, script_cache::ScriptCache},
    data_cache::RemoteCache,
    process_txn::{verify::VerifiedTransaction, ProcessTransaction},
    txn_executor::TransactionExecutor,
};
use libra_config::config::{VMMode, VMPublishingOption};
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::{
    transaction::{
        ChannelTransactionPayload, SignatureCheckedTransaction, TransactionPayload,
        MAX_TRANSACTION_SIZE_IN_BYTES,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::WriteSet,
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
pub struct ValidatedTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    txn: SignatureCheckedTransaction,
    txn_state: Option<ValidatedTransactionState<'alloc, 'txn, P>>,
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

impl<'alloc, 'txn, P> ValidatedTransaction<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    /// Creates a new instance by validating a `SignedTransaction`.
    ///
    /// This should be called through [`ProcessTransaction::validate`].
    pub(super) fn new(
        process_txn: ProcessTransaction<'alloc, 'txn, P>,
        mode: ValidationMode,
        publishing_option: &VMPublishingOption,
        vm_mode: VMMode,
    ) -> Result<Self, VMStatus> {
        let ProcessTransaction {
            txn,
            gas_schedule,
            module_cache,
            data_cache,
            ..
        } = process_txn;

        let txn_state = match txn.payload() {
            TransactionPayload::Program => return Err(VMStatus::new(StatusCode::MALFORMED)),
            TransactionPayload::Script(script) => {
                Some(ValidatedTransaction::validate(
                    &txn,
                    gas_schedule,
                    module_cache,
                    data_cache,
                    mode,
                    vm_mode,
                    || {
                        // Verify against whitelist if we are locked. Otherwise allow.
                        if !is_allowed_script(&publishing_option, &script.code()) {
                            warn!("[VM] Custom scripts not allowed: {:?}", &script.code());
                            return Err(VMStatus::new(StatusCode::UNKNOWN_SCRIPT));
                        }
                        Ok(())
                    },
                )?)
            }
            TransactionPayload::Module(module) => {
                debug!("validate module {:?}", module);
                Some(ValidatedTransaction::validate(
                    &txn,
                    gas_schedule,
                    module_cache,
                    data_cache,
                    mode,
                    vm_mode,
                    || {
                        if !publishing_option.is_open() {
                            warn!("[VM] Custom modules not allowed");
                            Err(VMStatus::new(StatusCode::UNKNOWN_MODULE))
                        } else {
                            Ok(())
                        }
                    },
                )?)
            }
            TransactionPayload::WriteSet(write_set) => {
                // The only acceptable write-set transaction for now is for the genesis
                // transaction.
                // XXX figure out a story for hard forks.
                if mode != ValidationMode::Genesis {
                    warn!("[VM] Attempt to process genesis after initialization");
                    return Err(VMStatus::new(StatusCode::REJECTED_WRITE_SET));
                }

                for (_access_path, write_op) in write_set {
                    // Genesis transactions only add entries, never delete them.
                    if write_op.is_deletion() {
                        error!("[VM] Bad genesis block");
                        // TODO: return more detailed error somehow?
                        return Err(VMStatus::new(StatusCode::INVALID_WRITE_SET));
                    }
                }

                None
            }
            TransactionPayload::Channel(channel_payload) => Some(ValidatedTransaction::validate(
                &txn,
                gas_schedule,
                module_cache,
                data_cache,
                mode,
                vm_mode,
                || {
                    Self::check_channel_payload(channel_payload)?;
                    Ok(())
                },
            )?),
        };

        Ok(Self { txn, txn_state })
    }

    fn check_channel_payload(channel_payload: &ChannelTransactionPayload) -> Result<(), VMStatus> {
        let channel_address = channel_payload.channel_address();
        let witness = channel_payload.witness();
        match channel_payload.verify() {
            Err(e) => {
                warn!("[VM] Verify channel payload signature fail: {:?}", e);
                return Err(
                    VMStatus::new(StatusCode::INVALID_SIGNATURE).with_message(format!("{:?}", e))
                );
            }
            Ok(_) => {}
        }
        let write_set = witness.write_set();
        for (ap, _op) in write_set {
            if &ap.address != &channel_address {
                warn!("[VM] Attempt to access a resource out of channel.");
                return Err(VMStatus::new(StatusCode::INVALID_WRITE_SET));
            }
        }
        Ok(())
    }

    /// Verifies the bytecode in this transaction.
    pub fn verify(
        self,
        script_cache: &'txn ScriptCache<'alloc>,
    ) -> Result<VerifiedTransaction<'alloc, 'txn, P>, VMStatus> {
        VerifiedTransaction::new(self, script_cache)
    }

    /// Returns a reference to the `SignatureCheckedTransaction` within.
    pub fn as_inner(&self) -> &SignatureCheckedTransaction {
        &self.txn
    }

    /// Consumes `self` and returns the `SignatureCheckedTransaction` within.
    #[allow(dead_code)]
    pub fn into_inner(self) -> SignatureCheckedTransaction {
        self.txn
    }

    /// Returns the `ValidatedTransactionState` within.
    pub(super) fn take_state(&mut self) -> Option<ValidatedTransactionState<'alloc, 'txn, P>> {
        self.txn_state.take()
    }

    fn validate(
        txn: &SignatureCheckedTransaction,
        gas_schedule: &'txn CostTable,
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
        mode: ValidationMode,
        vm_mode: VMMode,
        payload_check: impl Fn() -> Result<(), VMStatus>,
    ) -> Result<ValidatedTransactionState<'alloc, 'txn, P>, VMStatus> {
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
        // Only Onchain vm limit max gas.
        if vm_mode == VMMode::Onchain
            && txn.max_gas_amount() > gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS.get()
        {
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

        // Check channel write_set asset balance, offchain channel transaction should keep asset
        // balance, then cache the write_set to transaction cache for Move script to use.
        let pre_cache_write_set = match txn.payload() {
            TransactionPayload::Channel(channel_payload) => {
                Some(channel_payload.witness().write_set().clone())
            }
            _ => None,
        };

        let metadata = TransactionMetadata::new(&txn);
        let mut txn_state = ValidatedTransactionState::new(
            metadata,
            gas_schedule,
            module_cache,
            data_cache,
            pre_cache_write_set,
            vm_mode,
        );

        // Run the prologue to ensure that clients have enough gas and aren't tricking us by
        // sending us garbage.
        // TODO: write-set transactions (other than genesis??) should also run the prologue.
        match txn_state.txn_executor.run_prologue() {
            Ok(_) => {}
            Err(err) => {
                let vm_status = convert_prologue_runtime_error(&err, &txn.sender());

                // In validating mode, accept transactions with sequence number greater
                // or equal to the current sequence number.
                match (mode, vm_status.major_status) {
                    (ValidationMode::Validating, StatusCode::SEQUENCE_NUMBER_TOO_NEW) => {
                        trace!("[VM] Sequence number too new error ignored");
                    }
                    (_, _) => {
                        warn!("[VM] Error in prologue: {:?}", err);
                        return Err(vm_status);
                    }
                }
            }
        };

        Ok(txn_state)
    }
}

/// State for program-based [`ValidatedTransaction`] instances.
pub(super) struct ValidatedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    pub(super) txn_executor: TransactionExecutor<'alloc, 'txn, P>,
}

impl<'alloc, 'txn, P> ValidatedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    fn new(
        metadata: TransactionMetadata,
        gas_schedule: &'txn CostTable,
        module_cache: P,
        data_cache: &'txn dyn RemoteCache,
        pre_cache_write_set: Option<WriteSet>,
        vm_mode: VMMode,
    ) -> Self {
        let txn_executor = TransactionExecutor::new_with_vm_mode(
            module_cache,
            gas_schedule,
            data_cache,
            metadata,
            pre_cache_write_set,
            vm_mode,
        );
        Self { txn_executor }
    }
}
