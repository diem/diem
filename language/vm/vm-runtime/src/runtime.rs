// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_processor::execute_user_transaction_block,
    code_cache::{
        module_adapter::ModuleFetcherImpl,
        module_cache::{BlockModuleCache, VMModuleCache},
        script_cache::ScriptCache,
    },
    counters::report_verification_status,
    data_cache::BlockDataCache,
    gas_meter::load_gas_schedule,
    loaded_data::loaded_module::LoadedModule,
    process_txn::{validate::ValidationMode, ProcessTransaction},
    system_txn::block_metadata_processor::process_block_metadata,
};
use libra_config::config::{VMConfig, VMMode, VMPublishingOption};
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    block_metadata::BlockMetadata,
    transaction::{SignedTransaction, Transaction, TransactionOutput},
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::WriteSet,
};
use vm::{errors::VMResult, gas_schedule::CostTable};
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
    vm_mode: VMMode,
}

impl<'alloc> VMRuntime<'alloc> {
    /// Create a new VM instance with an Arena allocator to store the modules and a `config` that
    /// contains the whitelist that this VM is allowed to execute.
    pub fn new(allocator: &'alloc Arena<LoadedModule>, config: &VMConfig) -> Self {
        VMRuntime {
            code_cache: VMModuleCache::new(allocator),
            script_cache: ScriptCache::new(allocator),
            publishing_option: config.publishing_options.clone(),
            vm_mode: config.mode.clone(),
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
        trace!("[VM] Verify transaction: {:?}", txn);
        // Treat a transaction as a single block.
        let module_cache =
            BlockModuleCache::new(&self.code_cache, ModuleFetcherImpl::new(data_view));
        let data_cache = BlockDataCache::new(data_view);

        // If we fail to load the gas schedule, then we fail to process the block.
        let gas_schedule = match load_gas_schedule(&module_cache, &data_cache) {
            // TODO/XXX: This is a hack to get around not having proper writesets yet. Once that gets
            // in remove this line.
            Err(_) if data_view.is_genesis() => CostTable::zero(),
            Ok(cost_table) => cost_table,
            Err(_error) => {
                return Some(
                    VMStatus::new(StatusCode::VM_STARTUP_FAILURE)
                        .with_sub_status(sub_status::VSF_GAS_SCHEDULE_NOT_FOUND),
                )
            }
        };

        let signature_verified_txn = match txn.check_signature() {
            Ok(t) => t,
            Err(_) => return Some(VMStatus::new(StatusCode::INVALID_SIGNATURE)),
        };

        let process_txn = ProcessTransaction::new(
            signature_verified_txn,
            &gas_schedule,
            module_cache,
            &data_cache,
        );
        let mode = if data_view.is_genesis() {
            ValidationMode::Genesis
        } else {
            ValidationMode::Validating
        };

        let validated_txn = match process_txn.validate(mode, &self.publishing_option, self.vm_mode)
        {
            Ok(validated_txn) => validated_txn,
            Err(vm_status) => {
                let res = Some(vm_status);
                report_verification_status(&res);
                return res;
            }
        };
        let res = match validated_txn.verify(&self.script_cache) {
            Ok(_) => None,
            Err(vm_status) => Some(vm_status),
        };
        report_verification_status(&res);
        res
    }

    /// Execute a block of transactions. The output vector will have the exact same length as the
    /// input vector. The discarded transactions will be marked as `TransactionStatus::Discard` and
    /// have an empty writeset. Also the data view is immutable, and also does not have interior
    /// mutability. writes to be applied to the data view are encoded in the write set part of a
    /// transaction output.
    pub fn execute_block_transactions(
        &self,
        txn_block: Vec<Transaction>,
        data_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let mut result = vec![];
        let blocks = chunk_block_transactions(txn_block);
        let mut data_cache = BlockDataCache::new(data_view);
        let code_cache = BlockModuleCache::new(&self.code_cache, ModuleFetcherImpl::new(data_view));

        for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    result.append(&mut execute_user_transaction_block(
                        txns,
                        &code_cache,
                        &self.script_cache,
                        &mut data_cache,
                        &self.publishing_option,
                        self.vm_mode,
                    )?)
                }
                TransactionBlock::BlockPrologue(block_metadata) => result.push(
                    process_block_metadata(block_metadata, &code_cache, &mut data_cache),
                ),
                // TODO: Implement the logic for processing writeset transactions.
                TransactionBlock::WriteSet(_) => unimplemented!(""),
            }
        }
        Ok(result)
    }
}

pub(crate) enum TransactionBlock {
    UserTransaction(Vec<SignedTransaction>),
    WriteSet(WriteSet),
    BlockPrologue(BlockMetadata),
}

pub(crate) fn chunk_block_transactions(txns: Vec<Transaction>) -> Vec<TransactionBlock> {
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
            Transaction::WriteSet(ws) => {
                if !buf.is_empty() {
                    blocks.push(TransactionBlock::UserTransaction(buf));
                    buf = vec![];
                }
                blocks.push(TransactionBlock::WriteSet(ws));
            }
            Transaction::UserTransaction(txn) => {
                buf.push(txn);
            }
        }
    }
    if !buf.is_empty() {
        blocks.push(TransactionBlock::UserTransaction(buf));
    }
    blocks
}
