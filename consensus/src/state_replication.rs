// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus_types::{block::Block, common::Payload};
use executor_types::{Error, StateComputeResult};
use libra_crypto::HashValue;
use libra_types::ledger_info::LedgerInfoWithSignatures;

/// Retrieves and updates the status of transactions on demand (e.g., via talking with Mempool)
#[async_trait::async_trait]
pub trait TxnManager: Send + Sync {
    /// Brings new transactions to be applied.
    /// The `exclude_txns` list includes the transactions that are already pending in the
    /// branch of blocks consensus is trying to extend.
    async fn pull_txns(&self, max_size: u64, exclude: Vec<&Payload>) -> Result<Payload>;

    /// Notifies TxnManager about the executed result of the block,
    /// which includes the specifics of what transactions succeeded and failed.
    async fn notify(&self, block: &Block, compute_result: &StateComputeResult) -> Result<()>;

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
    ) -> Result<StateComputeResult, Error>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        block_ids: Vec<HashValue>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Result<()>;

    /// Best effort state synchronization to the given target LedgerInfo.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<()>;
}
