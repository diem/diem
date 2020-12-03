// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use consensus_types::block::Block;
use diem_crypto::HashValue;
use diem_types::transaction::Transaction;

mod execution_correctness;
mod execution_correctness_manager;
mod local;
mod process;
mod remote_service;
mod serializer;
mod thread;

pub use crate::{
    execution_correctness::ExecutionCorrectness,
    execution_correctness_manager::ExecutionCorrectnessManager, process::Process,
};

#[cfg(test)]
mod tests;

fn id_and_transactions_from_block(block: &Block) -> (HashValue, Vec<Transaction>) {
    let id = block.id();
    let mut transactions = vec![Transaction::BlockMetadata(block.into())];
    transactions.extend(
        block
            .payload()
            .unwrap_or(&vec![])
            .iter()
            .map(|txn| Transaction::UserTransaction(txn.clone())),
    );
    (id, transactions)
}
