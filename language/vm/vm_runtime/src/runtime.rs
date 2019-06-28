// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_processor::execute_block,
    code_cache::{
        module_adapter::ModuleFetcherImpl,
        module_cache::{BlockModuleCache, VMModuleCache},
        script_cache::ScriptCache,
    },
    counters,
    data_cache::BlockDataCache,
    loaded_data::loaded_module::LoadedModule,
    process_txn::{validate::ValidationMode, ProcessTransaction},
};
use config::config::{VMConfig, VMPublishingOption};
use logger::prelude::*;
use state_view::StateView;
use types::{
    transaction::{SignedTransaction, TransactionOutput},
    vm_error::{VMStatus, VMValidationStatus},
};
use vm_cache_map::Arena;

/// An instantiation of the MoveVM.
/// `code_cache` is the top level module cache that holds loaded published modules.
/// `script_cache` is the cache that stores all the scripts that have previously been invoked.
/// `publishing_option` is the publishing option that is set. This can be one of either:
/// * Locked, with a whitelist of scripts that the VM is allowed to execute. For scripts that aren't
///   in the whitelist, the VM will just reject it in `verify_transaction`.
/// * Custom scripts, which will allow arbitrary valid scripts, but no module publishing
/// * Open script and module publishing
pub struct VMRuntime<'alloc> {
    code_cache: VMModuleCache<'alloc>,
    script_cache: ScriptCache<'alloc>,
    publishing_option: VMPublishingOption,
}

impl<'alloc> VMRuntime<'alloc> {
    /// Create a new VM instance with an Arena allocator to store the modules and a `config` that
    /// contains the whitelist that this VM is allowed to execute.
    pub fn new(allocator: &'alloc Arena<LoadedModule>, config: &VMConfig) -> Self {
        VMRuntime {
            code_cache: VMModuleCache::new(allocator),
            script_cache: ScriptCache::new(allocator),
            publishing_option: config.publishing_options.clone(),
        }
    }

    /// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
    /// `Some(Err)` if the VM rejects it, with `Err` as an error code. We verify the following
    /// items:
    /// 1. The signature on the `SignedTransaction` matches the public key included in the
    ///    transaction
    /// 2. The script to be executed is in the whitelist.
    /// 3. Invokes `LibraAccount.prologue`, which checks properties such as the transaction has the
    /// right sequence number and the sender has enough balance to pay for the gas. 4.
    /// Transaction arguments matches the main function's type signature. 5. Script and modules
    /// in the transaction pass the bytecode static verifier.
    ///
    /// Note: In the future. we may defer these checks to a later pass, as all the scripts we will
    /// execute are pre-verified scripts. And bytecode verification is expensive. Thus whether we
    /// want to perform this check here remains unknown.
    pub fn verify_transaction(
        &self,
        txn: SignedTransaction,
        data_view: &dyn StateView,
    ) -> Option<VMStatus> {
        debug!("[VM] Verify transaction: {:?}", txn);
        // Treat a transaction as a single block.
        let module_cache =
            BlockModuleCache::new(&self.code_cache, ModuleFetcherImpl::new(data_view));
        let data_cache = BlockDataCache::new(data_view);

        let arena = Arena::new();
        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::Validation(VMValidationStatus::InvalidSignature)),
        };

        let process_txn =
            ProcessTransaction::new(signature_verified_txn, module_cache, &data_cache, &arena);
        let mode = if data_view.is_genesis() {
            ValidationMode::Genesis
        } else {
            ValidationMode::Validating
        };

        let validated_txn = match process_txn.validate(mode, &self.publishing_option) {
            Ok(validated_txn) => validated_txn,
            Err(vm_status) => {
                counters::UNVERIFIED_TRANSACTION.inc();
                return Some(vm_status);
            }
        };
        match validated_txn.verify() {
            Ok(_) => {
                counters::VERIFIED_TRANSACTION.inc();
                None
            }
            Err(vm_status) => {
                counters::UNVERIFIED_TRANSACTION.inc();
                Some(vm_status)
            }
        }
    }

    /// Execute a block of transactions. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty writeset. Also the data view is immutable, and also does not have interior
    /// mutability. writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    pub fn execute_block_transactions(
        &self,
        txn_block: Vec<SignedTransaction>,
        data_view: &dyn StateView,
    ) -> Vec<TransactionOutput> {
        execute_block(
            txn_block,
            &self.code_cache,
            &self.script_cache,
            data_view,
            &self.publishing_option,
        )
    }
}
