use crate::{
    code_cache::module_cache::{ModuleCache, TransactionModuleCache},
    data_cache::RemoteCache,
    loaded_data::loaded_module::LoadedModule,
    process_txn::{verify::VerifiedTransaction, ProcessTransaction},
    txn_executor::TransactionExecutor,
};
use config::config::VMPublishingOption;
use logger::prelude::*;
use tiny_keccak::Keccak;
use types::{
    transaction::{
        SignatureCheckedTransaction, TransactionPayload, MAX_TRANSACTION_SIZE_IN_BYTES,
        SCRIPT_HASH_LENGTH,
    },
    vm_error::{VMStatus, VMValidationStatus},
};
use vm::{
    errors::convert_prologue_runtime_error, gas_schedule, transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;

pub fn is_allowed_script(publishing_option: &VMPublishingOption, program: &[u8]) -> bool {
    match publishing_option {
        VMPublishingOption::Open | VMPublishingOption::CustomScripts => true,
        VMPublishingOption::Locked(whitelist) => {
            let mut hash = [0u8; SCRIPT_HASH_LENGTH];
            let mut keccak = Keccak::new_sha3_256();
            keccak.update(program);
            keccak.finalize(&mut hash);
            whitelist.contains(&hash)
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
    ) -> Result<Self, VMStatus> {
        let ProcessTransaction {
            txn,
            module_cache,
            data_cache,
            allocator,
            ..
        } = process_txn;
        let txn = match txn.check_signature() {
            Ok(txn) => txn,
            Err(_) => {
                error!("[VM] Invalid signature");
                return Err(VMStatus::Validation(VMValidationStatus::InvalidSignature));
            }
        };

        let txn_state = match txn.payload() {
            TransactionPayload::Program(program) => {
                // The transaction is too large.
                if txn.raw_txn_bytes_len() > MAX_TRANSACTION_SIZE_IN_BYTES {
                    let error_str = format!(
                        "max size: {}, txn size: {}",
                        MAX_TRANSACTION_SIZE_IN_BYTES,
                        txn.raw_txn_bytes_len()
                    );
                    warn!(
                        "[VM] Transaction size too big {} (max {})",
                        txn.raw_txn_bytes_len(),
                        MAX_TRANSACTION_SIZE_IN_BYTES
                    );
                    return Err(VMStatus::Validation(
                        VMValidationStatus::ExceededMaxTransactionSize(error_str),
                    ));
                }

                // The submitted max gas units that the transaction can consume is greater than the
                // maximum number of gas units bound that we have set for any
                // transaction.
                if txn.max_gas_amount() > gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS {
                    let error_str = format!(
                        "max gas units: {}, gas units submitted: {}",
                        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS,
                        txn.max_gas_amount()
                    );
                    warn!(
                        "[VM] Gas unit error; max {}, submitted {}",
                        gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS,
                        txn.max_gas_amount()
                    );
                    return Err(VMStatus::Validation(
                        VMValidationStatus::MaxGasUnitsExceedsMaxGasUnitsBound(error_str),
                    ));
                }

                // The submitted transactions max gas units needs to be at least enough to cover the
                // intrinsic cost of the transaction as calculated against the size of the
                // underlying `RawTransaction`
                let min_txn_fee =
                    gas_schedule::calculate_intrinsic_gas(txn.raw_txn_bytes_len() as u64);
                if txn.max_gas_amount() < min_txn_fee {
                    let error_str = format!(
                        "min gas required for txn: {}, gas submitted: {}",
                        min_txn_fee,
                        txn.max_gas_amount()
                    );
                    warn!(
                        "[VM] Gas unit error; min {}, submitted {}",
                        min_txn_fee,
                        txn.max_gas_amount()
                    );
                    return Err(VMStatus::Validation(
                        VMValidationStatus::MaxGasUnitsBelowMinTransactionGasUnits(error_str),
                    ));
                }

                // The submitted gas price is less than the minimum gas unit price set by the VM.
                // NB: MIN_PRICE_PER_GAS_UNIT may equal zero, but need not in the future. Hence why
                // we turn off the clippy warning.
                #[allow(clippy::absurd_extreme_comparisons)]
                let below_min_bound = txn.gas_unit_price() < gas_schedule::MIN_PRICE_PER_GAS_UNIT;
                if below_min_bound {
                    let error_str = format!(
                        "gas unit min price: {}, submitted price: {}",
                        gas_schedule::MIN_PRICE_PER_GAS_UNIT,
                        txn.gas_unit_price()
                    );
                    warn!(
                        "[VM] Gas unit error; min {}, submitted {}",
                        gas_schedule::MIN_PRICE_PER_GAS_UNIT,
                        txn.gas_unit_price()
                    );
                    return Err(VMStatus::Validation(
                        VMValidationStatus::GasUnitPriceBelowMinBound(error_str),
                    ));
                }

                // The submitted gas price is greater than the maximum gas unit price set by the VM.
                if txn.gas_unit_price() > gas_schedule::MAX_PRICE_PER_GAS_UNIT {
                    let error_str = format!(
                        "gas unit max price: {}, submitted price: {}",
                        gas_schedule::MAX_PRICE_PER_GAS_UNIT,
                        txn.gas_unit_price()
                    );
                    warn!(
                        "[VM] Gas unit error; min {}, submitted {}",
                        gas_schedule::MAX_PRICE_PER_GAS_UNIT,
                        txn.gas_unit_price()
                    );
                    return Err(VMStatus::Validation(
                        VMValidationStatus::GasUnitPriceAboveMaxBound(error_str),
                    ));
                }

                // Verify against whitelist if we are locked. Otherwise allow.
                if !is_allowed_script(&publishing_option, &program.code()) {
                    warn!("[VM] Custom scripts not allowed: {:?}", &program.code());
                    return Err(VMStatus::Validation(VMValidationStatus::UnknownScript));
                }

                if !publishing_option.is_open() {
                    // Not allowing module publishing for now.
                    if !program.modules().is_empty() {
                        warn!("[VM] Custom modules not allowed");
                        return Err(VMStatus::Validation(VMValidationStatus::UnknownModule));
                    }
                }

                let metadata = TransactionMetadata::new(&txn);
                let mut txn_state =
                    ValidatedTransactionState::new(metadata, module_cache, data_cache, allocator);

                // Run the prologue to ensure that clients have enough gas and aren't tricking us by
                // sending us garbage.
                // TODO: write-set transactions (other than genesis??) should also run the prologue.
                match txn_state.txn_executor.run_prologue() {
                    Ok(Ok(_)) => {}
                    Ok(Err(ref err)) => {
                        let vm_status = convert_prologue_runtime_error(&err, &txn.sender());

                        // In validating mode, accept transactions with sequence number greater
                        // or equal to the current sequence number.
                        match (mode, vm_status) {
                            (
                                ValidationMode::Validating,
                                VMStatus::Validation(VMValidationStatus::SequenceNumberTooNew),
                            ) => {
                                trace!("[VM] Sequence number too new error ignored");
                            }
                            (_, vm_status) => {
                                warn!("[VM] Error in prologue: {:?}", err);
                                return Err(vm_status);
                            }
                        }
                    }
                    Err(ref err) => {
                        error!("[VM] VM internal error in prologue: {:?}", err);
                        return Err(err.into());
                    }
                };

                Some(txn_state)
            }
            TransactionPayload::WriteSet(write_set) => {
                // The only acceptable write-set transaction for now is for the genesis
                // transaction.
                // XXX figure out a story for hard forks.
                if mode != ValidationMode::Genesis {
                    warn!("[VM] Attempt to process genesis after initialization");
                    return Err(VMStatus::Validation(VMValidationStatus::RejectedWriteSet));
                }

                for (_access_path, write_op) in write_set {
                    // Genesis transactions only add entries, never delete them.
                    if write_op.is_deletion() {
                        error!("[VM] Bad genesis block");
                        // TODO: return more detailed error somehow?
                        return Err(VMStatus::Validation(VMValidationStatus::InvalidWriteSet));
                    }
                }

                None
            }
        };

        Ok(Self { txn, txn_state })
    }

    /// Verifies the bytecode in this transaction.
    pub fn verify(self) -> Result<VerifiedTransaction<'alloc, 'txn, P>, VMStatus> {
        VerifiedTransaction::new(self)
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
}

/// State for program-based [`ValidatedTransaction`] instances.
pub(super) struct ValidatedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    // <'txn, 'txn> looks weird, but it just means that the module cache passed in (the
    // TransactionModuleCache) allocates for that long.
    pub(super) txn_executor:
        TransactionExecutor<'txn, 'txn, TransactionModuleCache<'alloc, 'txn, P>>,
}

impl<'alloc, 'txn, P> ValidatedTransactionState<'alloc, 'txn, P>
where
    'alloc: 'txn,
    P: ModuleCache<'alloc>,
{
    fn new(
        metadata: TransactionMetadata,
        module_cache: P,
        data_cache: &'txn RemoteCache,
        allocator: &'txn Arena<LoadedModule>,
    ) -> Self {
        // This temporary cache is used for modules published by a single transaction.
        let txn_module_cache = TransactionModuleCache::new(module_cache, allocator);
        let txn_executor = TransactionExecutor::new(txn_module_cache, data_cache, metadata);
        Self { txn_executor }
    }
}
