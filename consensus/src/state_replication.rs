// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus_types::{block::Block, executed_block::ExecutedBlock};
use executor_types::{ExecutedTrees, ProcessedVMOutput, StateComputeResult};
use libra_crypto::HashValue;
use libra_types::{crypto_proxies::ValidatorChangeProof, ledger_info::LedgerInfoWithSignatures};

/// Retrieves and updates the status of transactions on demand (e.g., via talking with Mempool)
#[async_trait::async_trait]
pub trait TxnManager: Send + Sync {
    type Payload;

    /// Brings new transactions to be applied.
    /// The `exclude_txns` list includes the transactions that are already pending in the
    /// branch of blocks consensus is trying to extend.
    async fn pull_txns(
        &mut self,
        max_size: u64,
        exclude_txns: Vec<&Self::Payload>,
    ) -> Result<Self::Payload>;

    /// Notifies TxnManager about the payload of the committed block including the state compute
    /// result, which includes the specifics of what transactions succeeded and failed.
    async fn commit_txns(
        &mut self,
        txns: &Self::Payload,
        compute_result: &StateComputeResult,
    ) -> Result<()>;

    /// Bypass the trait object non-clonable limit.
    fn _clone_box(&self) -> Box<dyn TxnManager<Payload = Self::Payload>>;

    // Helper to trace transactions after block is generated
    fn trace_transactions(&self, _txns: &Self::Payload, _block_id: HashValue) {}
}

impl<T> Clone for Box<dyn TxnManager<Payload = T>> {
    fn clone(&self) -> Box<dyn TxnManager<Payload = T>> {
        self._clone_box()
    }
}

/// While Consensus is managing proposed blocks, `StateComputer` is managing the results of the
/// (speculative) execution of their payload.
/// StateComputer is using proposed block ids for identifying the transactions.
#[async_trait::async_trait]
pub trait StateComputer: Send + Sync {
    type Payload;

    /// How to execute a sequence of transactions and obtain the next state. While some of the
    /// transactions succeed, some of them can fail.
    /// In case all the transactions are failed, new_state_id is equal to the previous state id.
    fn compute(
        &self,
        // The block that will be computed.
        block: &Block<Self::Payload>,
        // The executed trees of parent block.
        parent_executed_trees: &ExecutedTrees,
        // The last committed trees.
        committed_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    async fn commit(
        &self,
        blocks: Vec<&ExecutedBlock<Self::Payload>>,
        finality_proof: LedgerInfoWithSignatures,
        synced_trees: &ExecutedTrees,
    ) -> Result<()>;

    /// Best effort state synchronization to the given target LedgerInfo.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    async fn sync_to(&self, target: LedgerInfoWithSignatures) -> Result<()>;

    /// Generate the epoch change proof from start_epoch to the latest epoch.
    async fn get_epoch_proof(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<ValidatorChangeProof>;
}
