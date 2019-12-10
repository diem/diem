// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_processor::execute_user_transaction_block,
    counters::*,
    data_cache::BlockDataCache,
    move_vm::MoveVM,
    process_txn::validate::{ValidatedTransaction, ValidationMode},
    runtime::VMRuntime,
    system_txn::block_metadata_processor::process_block_metadata,
    VMExecutor, VMVerifier,
};
use libra_config::config::VMConfig;
use libra_logger::prelude::*;
use libra_state_view::StateView;
use libra_types::{
    block_metadata::BlockMetadata,
    transaction::{SignedTransaction, Transaction, TransactionOutput},
    vm_error::{sub_status, StatusCode, VMStatus},
    write_set::WriteSet,
};
use std::sync::Arc;
use vm::{errors::VMResult, gas_schedule::CostTable, transaction_metadata::TransactionMetadata};

/// A wrapper to make VMRuntime standalone and thread safe.
#[derive(Clone)]
pub struct LibraVM {
    move_vm: Arc<MoveVM>,
}

impl LibraVM {
    pub fn new(config: &VMConfig) -> Self {
        let inner = MoveVM::new(config);
        Self {
            move_vm: Arc::new(inner),
        }
    }
}

// Validators external API
impl VMVerifier for LibraVM {
    fn validate_transaction(
        &self,
        transaction: SignedTransaction,
        state_view: &dyn StateView,
    ) -> Option<VMStatus> {
        // TODO: This should be implemented as an async function.
        record_stats! {time_hist | TXN_VALIDATION_TIME_TAKEN | {
            // XXX: This is different from invoking bytecode verifier. Will clean the code up after
            // refactor.
            self.move_vm
                .execute_runtime(move |runtime| verify_transaction(runtime, transaction, state_view))
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
        config: &VMConfig,
        state_view: &dyn StateView,
    ) -> VMResult<Vec<TransactionOutput>> {
        let vm = MoveVM::new(config);

        let mut result = vec![];
        let blocks = chunk_block_transactions(transactions);
        let mut data_cache = BlockDataCache::new(state_view);
        for block in blocks {
            match block {
                TransactionBlock::UserTransaction(txns) => {
                    let outs = vm.execute_runtime(|runtime| {
                        execute_user_transaction_block(txns, runtime, &mut data_cache, state_view)
                    });
                    result.append(&mut outs?);
                }
                TransactionBlock::BlockPrologue(block_metadata) => {
                    result.push(vm.execute_runtime(|runtime| {
                        process_block_metadata(block_metadata, runtime, state_view, &mut data_cache)
                    })?)
                }
                // TODO: Implement the logic for processing writeset transactions.
                TransactionBlock::WriteSet(_) => unimplemented!(""),
            }
        }

        Ok(result)
    }
}

/// Transactions divided by transaction flow.
/// Transaction flows are different across different types of transactions.
pub enum TransactionBlock {
    UserTransaction(Vec<SignedTransaction>),
    WriteSet(WriteSet),
    BlockPrologue(BlockMetadata),
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

/// Determine if a transaction is valid. Will return `None` if the transaction is accepted,
/// `Some(Err)` if the VM rejects it, with `Err` as an error code. We verify the following
/// items:
/// 1. The signature on the `SignedTransaction` matches the public key included in the
///    transaction
/// 2. The script to be executed is in the whitelist.
/// 3. Invokes `LibraAccount.prologue`, which checks properties such as the transaction has the
/// right sequence number and the sender has enough balance to pay for the gas.
/// 4. Transaction arguments matches the main function's type signature.
pub fn verify_transaction(
    runtime: &VMRuntime,
    txn: SignedTransaction,
    state_view: &dyn StateView,
) -> Option<VMStatus> {
    trace!("[VM] Verify transaction: {:?}", txn);
    let mut data_cache = BlockDataCache::new(state_view);

    let gas_schedule = match runtime.load_gas_schedule(&mut data_cache, state_view) {
        // TODO/XXX: This is a hack to get around not having proper writesets yet. Once that gets
        // in remove this line.
        Err(_) if state_view.is_genesis() => CostTable::zero(),
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

    let mode = if state_view.is_genesis() {
        ValidationMode::Genesis
    } else {
        ValidationMode::Validating
    };

    let validated_txn = match ValidatedTransaction::new(
        signature_verified_txn,
        runtime,
        state_view,
        &gas_schedule,
        &data_cache,
        mode,
    ) {
        Ok(validated_txn) => validated_txn,
        Err(vm_status) => {
            let res = Some(vm_status);
            report_verification_status(&res);
            return res;
        }
    };
    let txn_data = TransactionMetadata::new(validated_txn.as_inner());
    let res = match validated_txn.verify(&txn_data) {
        Ok(_) => None,
        Err(vm_status) => Some(vm_status),
    };
    report_verification_status(&res);
    res
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
