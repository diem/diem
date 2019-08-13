// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::QuorumCert;
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer};
use crypto::{hash::ACCUMULATOR_PLACEHOLDER_HASH, HashValue};
use failure::Result;
use futures::Future;
use nextgen_crypto::ed25519::*;
use serde::{Deserialize, Serialize};
use state_synchronizer::SyncStatus;
use std::{pin::Pin, sync::Arc};
use types::{
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, Version},
    validator_set::ValidatorSet,
};

/// A structure that specifies the result of the execution.
/// The execution is responsible for generating the ID of the new state, which is returned in the
/// result.
///
/// Not every transaction in the payload succeeds: the returned vector keeps the boolean status
/// of success / failure of the transactions.
/// Note that the specific details of compute_status are opaque to StateMachineReplication,
/// which is going to simply pass the results between StateComputer and TxnManager.
pub struct StateComputeResult {
    /// The new state generated after the execution.
    pub new_state_id: HashValue,
    /// The compute status (success/failure) of the given payload. The specific details are opaque
    /// for StateMachineReplication, which is merely passing it between StateComputer and
    /// TxnManager.
    pub compute_status: Vec<bool>,
    /// Counts the number of `true` values in the `compute_status` field.
    pub num_successful_txns: u64,
    /// If set, these are the validator public keys that will be used to start the next epoch
    /// immediately after this state is committed
    /// TODO [Reconfiguration] the validators are currently ignored, no reconfiguration yet.
    pub validators: Option<ValidatorSet>,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutedState {
    pub state_id: HashValue,
    pub version: Version,
}

impl ExecutedState {
    pub fn state_for_genesis() -> Self {
        ExecutedState {
            state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
            version: 0,
        }
    }
}

impl CanonicalSerialize for ExecutedState {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_raw_bytes(self.state_id.as_ref())?;
        serializer.encode_u64(self.version)?;
        Ok(())
    }
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
        // We're going to use a special GENESIS_BLOCK_ID constant defined in crypto::hash module to
        // refer to the block id of the Genesis block, which is executed in a special way.
        parent_block_id: HashValue,
        // The id of a current block.
        block_id: HashValue,
        // Transactions to execute.
        transactions: &Self::Payload,
    ) -> Pin<Box<dyn Future<Output = Result<StateComputeResult>> + Send>>;

    /// Send a successful commit. A future is fulfilled when the state is finalized.
    fn commit(
        &self,
        commit: LedgerInfoWithSignatures<Ed25519Signature>,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    /// Synchronize to a commit that not present locally.
    fn sync_to(
        &self,
        commit: QuorumCert,
    ) -> Pin<Box<dyn Future<Output = Result<SyncStatus>> + Send>>;

    /// Get a chunk of transactions as a batch
    fn get_chunk(
        &self,
        start_version: u64,
        target_version: u64,
        batch_size: u64,
    ) -> Pin<Box<dyn Future<Output = Result<TransactionListWithProof>> + Send>>;
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
