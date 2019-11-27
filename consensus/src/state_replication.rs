// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::txn_manager::TxnManager;
use consensus_types::block::Block;
use consensus_types::executed_block::ExecutedBlock;
use executor::{ExecutedTrees, ProcessedVMOutput};
use failure::Result;
use futures::Future;
use libra_types::crypto_proxies::{LedgerInfoWithSignatures, ValidatorChangeEventWithProof};
use std::{pin::Pin, sync::Arc};

/// While Consensus is managing proposed blocks, `StateComputer` is managing the results of the
/// (speculative) execution of their payload.
/// StateComputer is using proposed block ids for identifying the transactions.
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
        executed_trees: ExecutedTrees,
    ) -> Pin<Box<dyn Future<Output = Result<ProcessedVMOutput>> + Send>>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    fn commit(
        &self,
        blocks: Vec<&ExecutedBlock<Self::Payload>>,
        finality_proof: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Best effort state synchronization to the given target LedgerInfo.
    /// In case of success (`Result::Ok`) the LI of storage is at the given target.
    /// In case of failure (`Result::Error`) the LI of storage remains unchanged, and the validator
    /// can assume there were no modifications to the storage made.
    fn sync_to(
        &self,
        target: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn committed_trees(&self) -> ExecutedTrees;

    fn sync_to_or_bail(&self, commit: LedgerInfoWithSignatures) {
        let status = futures::executor::block_on(self.sync_to(commit));
        // TODO: this is going to change after https://github.com/libra/libra/issues/1590
        status.expect(
            "state synchronizer failure, this validator will be killed as it can not \
             recover from this error.  After the validator is restarted, synchronization will \
             be retried. failure:",
        )
    }

    /// Generate the epoch change proof from start_epoch to the latest epoch.
    fn get_epoch_proof(
        &self,
        start_epoch: u64,
    ) -> Pin<Box<dyn Future<Output = Result<ValidatorChangeEventWithProof>> + Send>>;
}

pub trait StateMachineReplication {
    type Payload;
    /// The function is synchronous: it returns when the state is initialized / recovered from
    /// persisted storage and all the threads have been started.
    fn start(
        &mut self,
        txn_manager: Arc<dyn TxnManager<Payload = Self::Payload>>,
        state_computer: Arc<dyn StateComputer<Payload = Self::Payload>>,
    ) -> Result<()>;

    /// Stop is synchronous: returns when all the threads are shutdown and the state is persisted.
    fn stop(&mut self);
}
