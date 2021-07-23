// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{MempoolError, StateSyncError};
use anyhow::Result;
use consensus_types::{block::Block, common::Payload, executed_block::ExecutedBlock};
use diem_crypto::HashValue;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use executor_types::{Error as ExecutionError, StateComputeResult};
use std::sync::Arc;

pub type StateComputerCommitCallBackType =
    Box<dyn FnOnce(&[Arc<ExecutedBlock>], LedgerInfoWithSignatures) + Send + Sync>;
#[cfg(test)]
pub fn empty_state_computer_call_back() -> StateComputerCommitCallBackType {
    Box::new(|_, _| {})
}

/// Retrieves and updates the status of transactions on demand (e.g., via talking with Mempool)
#[async_trait::async_trait]
pub trait TxnManager: Send + Sync {
    /// Brings new transactions to be applied.
    /// The `exclude_txns` list includes the transactions that are already pending in the
    /// branch of blocks consensus is trying to extend.
    async fn pull_txns(
        &self,
        max_size: u64,
        exclude: Vec<&Payload>,
    ) -> Result<Payload, MempoolError>;

    /// Notifies TxnManager about the executed result of the block,
    /// which includes the specifics of what transactions succeeded and failed.
    async fn notify(
        &self,
        block: &Block,
        compute_result: &StateComputeResult,
    ) -> Result<(), MempoolError>;

    /// Helper to trace transactions after block is generated
    fn trace_transactions(&self, _block: &Block) {}
}

/// While Consensus is managing proposed blocks, `StateComputer` is managing the results of the
/// (speculative) execution of their payload.
/// StateComputer is using proposed block ids for identifying the transactions.
#[async_trait::async_trait]
pub trait StateComputer: Send + Sync {
    /// How to execute a sequence of transactions and obtain the next state. While some of the
    /// transactions succeed, some of them can fail.
    /// In case all the transactions are failed, new_state_id is equal to the previous state id.
    fn compute(
        &self,
        // The block that will be computed.
        block: &Block,
        // The parent block root hash.
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, ExecutionError>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        blocks: &[Arc<ExecutedBlock>],
        finality_proof: LedgerInfoWithSignatures,
        callback: StateComputerCommitCallBackType,
    ) -> Result<(), ExecutionError>;

    /// Best effort state synchronization to the given target LedgerInfo.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<(), StateSyncError>;
}
