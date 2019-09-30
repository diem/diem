// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::QuorumCert;
use failure::Result;
use futures::Future;
use libra_crypto::HashValue;
use libra_executor::StateComputeResult;
use libra_types::crypto_proxies::LedgerInfoWithSignatures;
use std::{pin::Pin, sync::Arc};

/// Retrieves and updates the status of transactions on demand (e.g., via talking with Mempool)
pub trait TxnManager: Send + Sync {
    type Payload;

    /// Brings new transactions to be applied.
    /// The `exclude_txns` list includes the transactions that are already pending in the
    /// branch of blocks consensus is trying to extend.
    fn pull_txns(
        &self,
        max_size: u64,
        exclude_txns: Vec<&Self::Payload>,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Payload>> + Send>>;

    /// Notifies TxnManager about the payload of the committed block including the state compute
    /// result, which includes the specifics of what transactions succeeded and failed.
    fn commit_txns<'a>(
        &'a self,
        txns: &Self::Payload,
        compute_result: &StateComputeResult,
        // Monotonic timestamp_usecs of committed blocks is used to GC expired transactions.
        timestamp_usecs: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

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
        // The id of a parent block, on top of which the given transactions should be executed.
        // We're going to use a special GENESIS_BLOCK_ID constant defined in libra_crypto::hash
        // module to refer to the block id of the Genesis block, which is executed in a
        // special way.
        parent_block_id: HashValue,
        // The id of a current block.
        block_id: HashValue,
        // Transactions to execute.
        transactions: &Self::Payload,
    ) -> Pin<Box<dyn Future<Output = Result<StateComputeResult>> + Send>>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    fn commit(
        &self,
        commit: LedgerInfoWithSignatures,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn sync_to(&self, commit: QuorumCert) -> Pin<Box<dyn Future<Output = Result<bool>> + Send>>;

    fn sync_to_or_bail(&self, commit: QuorumCert) {
        let status = futures::executor::block_on(self.sync_to(commit));
        match status {
            Ok(true) => (),
            Ok(false) => panic!(
                "state synchronizer failure, this validator will be killed as it can not \
                 recover from this error.  After the validator is restarted, synchronization will \
                 be retried.",
            ),
            Err(e) => panic!(
                "state synchronizer failure: {:?}, this validator will be killed as it can not \
                 recover from this error.  After the validator is restarted, synchronization will \
                 be retried.",
                e
            ),
        }
    }
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
