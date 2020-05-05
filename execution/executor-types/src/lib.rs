// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;
pub use error::Error;

use anyhow::Result;
use libra_crypto::{
    hash::{EventAccumulatorHasher, TransactionAccumulatorHasher, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    epoch_info::EpochInfo,
    ledger_info::LedgerInfoWithSignatures,
    proof::{accumulator::InMemoryAccumulator, SparseMerkleProof},
    transaction::{Transaction, TransactionListWithProof, TransactionStatus, Version},
};
use scratchpad::{ProofRead, SparseMerkleTree};
use serde::{Deserialize, Serialize};
use std::{cmp::max, collections::HashMap, sync::Arc};
use storage_interface::TreeState;

pub trait ChunkExecutor: Send {
    /// Verifies the transactions based on the provided proofs and ledger info. If the transactions
    /// are valid, executes them and commits immediately if execution results match the proofs.
    /// Returns a vector of reconfiguration events in the chunk
    fn execute_and_commit_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        // Target LI that has been verified independently: the proofs are relative to this version.
        verified_target_li: LedgerInfoWithSignatures,
        // An optional end of epoch LedgerInfo. We do not allow chunks that end epoch without
        // carrying any epoch change LI.
        epoch_change_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>>;
}

pub trait BlockExecutor: Send {
    /// Get the latest committed block id
    fn committed_block_id(&mut self) -> Result<HashValue, Error>;

    /// Reset the internal state including cache with newly fetched latest committed block from storage.
    fn reset(&mut self) -> Result<(), Error>;

    /// Executes a block.
    fn execute_block(
        &mut self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error>;

    /// Saves eligible blocks to persistent storage.
    /// If we have multiple blocks and not all of them have signatures, we may send them to storage
    /// in a few batches. For example, if we have
    /// ```text
    /// A <- B <- C <- D <- E
    /// ```
    /// and only `C` and `E` have signatures, we will send `A`, `B` and `C` in the first batch,
    /// then `D` and `E` later in the another batch.
    /// Commits a block and all its ancestors in a batch manner.
    ///
    /// Returns `Ok(Result<Vec<Transaction>, Vec<ContractEvents>)` if successful,
    /// where Vec<Transaction> is a vector of transactions that were kept from the submitted blocks, and
    /// Vec<ContractEvents> is a vector of reconfiguration events in the submitted blocks
    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error>;
}

/// A structure that summarizes the result of the execution needed for consensus to agree on.
/// The execution is responsible for generating the ID of the new state, which is returned in the
/// result.
///
/// Not every transaction in the payload succeeds: the returned vector keeps the boolean status
/// of success / failure of the transactions.
/// Note that the specific details of compute_status are opaque to StateMachineReplication,
/// which is going to simply pass the results between StateComputer and TxnManager.
#[derive(Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct StateComputeResult {
    /// transaction accumulator root hash is identified as `state_id` in Consensus.
    root_hash: HashValue,
    /// Represents the roots of all the full subtrees from left to right in this accumulator
    /// after the execution. For details, please see [`InMemoryAccumulator`](accumulator::InMemoryAccumulator).
    frozen_subtree_roots: Vec<HashValue>,
    /// The number of leaves of the transaction accumulator after executing a proposed block.
    /// This state must be persisted to ensure that on restart that the version is calculated correctly.
    num_leaves: u64,
    /// If set, this is the new epoch info that should be changed to if this block is committed.
    epoch_info: Option<EpochInfo>,
    /// The compute status (success/failure) of the given payload. The specific details are opaque
    /// for StateMachineReplication, which is merely passing it between StateComputer and
    /// TxnManager.
    compute_status: Vec<TransactionStatus>,
    /// The transaction info hashes of all success txns.
    transaction_info_hashes: Vec<HashValue>,
}

impl StateComputeResult {
    pub fn new(
        root_hash: HashValue,
        frozen_subtree_roots: Vec<HashValue>,
        num_leaves: u64,
        epoch_info: Option<EpochInfo>,
        compute_status: Vec<TransactionStatus>,
        transaction_info_hashes: Vec<HashValue>,
    ) -> Self {
        Self {
            root_hash,
            frozen_subtree_roots,
            num_leaves,
            epoch_info,
            compute_status,
            transaction_info_hashes,
        }
    }
}

impl StateComputeResult {
    pub fn version(&self) -> Version {
        max(self.num_leaves, 1) - 1
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash
    }

    pub fn compute_status(&self) -> &Vec<TransactionStatus> {
        &self.compute_status
    }

    pub fn epoch_info(&self) -> &Option<EpochInfo> {
        &self.epoch_info
    }

    pub fn transaction_info_hashes(&self) -> &Vec<HashValue> {
        &self.transaction_info_hashes
    }

    pub fn num_leaves(&self) -> u64 {
        self.num_leaves
    }

    pub fn frozen_subtree_roots(&self) -> &Vec<HashValue> {
        &self.frozen_subtree_roots
    }

    pub fn has_reconfiguration(&self) -> bool {
        self.epoch_info.is_some()
    }
}

/// The entire set of data associated with a transaction. In addition to the output generated by VM
/// which includes the write set and events, this also has the in-memory trees.
#[derive(Clone, Debug)]
pub struct TransactionData {
    /// Each entry in this map represents the new blob value of an account touched by this
    /// transaction. The blob is obtained by deserializing the previous blob into a BTreeMap,
    /// applying relevant portion of write set on the map and serializing the updated map into a
    /// new blob.
    account_blobs: HashMap<AccountAddress, AccountStateBlob>,

    /// The list of events emitted during this transaction.
    events: Vec<ContractEvent>,

    /// The execution status set by the VM.
    status: TransactionStatus,

    /// The in-memory Sparse Merkle Tree after the write set is applied. This is `Rc` because the
    /// tree has uncommitted state and sometimes `StateVersionView` needs to have a pointer to the
    /// tree so VM can read it.
    state_tree: Arc<SparseMerkleTree>,

    /// The in-memory Merkle Accumulator that has all events emitted by this transaction.
    event_tree: Arc<InMemoryAccumulator<EventAccumulatorHasher>>,

    /// The amount of gas used.
    gas_used: u64,

    /// The transaction info hash if the VM status output was keep, None otherwise
    txn_info_hash: Option<HashValue>,
}

impl TransactionData {
    pub fn new(
        account_blobs: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>,
        status: TransactionStatus,
        state_tree: Arc<SparseMerkleTree>,
        event_tree: Arc<InMemoryAccumulator<EventAccumulatorHasher>>,
        gas_used: u64,
        txn_info_hash: Option<HashValue>,
    ) -> Self {
        TransactionData {
            account_blobs,
            events,
            status,
            state_tree,
            event_tree,
            gas_used,
            txn_info_hash,
        }
    }

    pub fn account_blobs(&self) -> &HashMap<AccountAddress, AccountStateBlob> {
        &self.account_blobs
    }

    pub fn events(&self) -> &[ContractEvent] {
        &self.events
    }

    pub fn status(&self) -> &TransactionStatus {
        &self.status
    }

    pub fn state_root_hash(&self) -> HashValue {
        self.state_tree.root_hash()
    }

    pub fn event_root_hash(&self) -> HashValue {
        self.event_tree.root_hash()
    }

    pub fn gas_used(&self) -> u64 {
        self.gas_used
    }

    pub fn prune_state_tree(&self) {
        self.state_tree.prune()
    }

    pub fn txn_info_hash(&self) -> Option<HashValue> {
        self.txn_info_hash
    }
}

/// The output of Processing the vm output of a series of transactions to the parent
/// in-memory state merkle tree and accumulator.
#[derive(Debug, Clone)]
pub struct ProcessedVMOutput {
    /// The entire set of data associated with each transaction.
    transaction_data: Vec<TransactionData>,

    /// The in-memory Merkle Accumulator and state Sparse Merkle Tree after appending all the
    /// transactions in this set.
    executed_trees: ExecutedTrees,

    /// If set, this is the new epoch info that should be changed to if this block is committed.
    epoch_info: Option<EpochInfo>,
}

impl ProcessedVMOutput {
    pub fn new(
        transaction_data: Vec<TransactionData>,
        executed_trees: ExecutedTrees,
        epoch_info: Option<EpochInfo>,
    ) -> Self {
        ProcessedVMOutput {
            transaction_data,
            executed_trees,
            epoch_info,
        }
    }

    pub fn transaction_data(&self) -> &[TransactionData] {
        &self.transaction_data
    }

    pub fn executed_trees(&self) -> &ExecutedTrees {
        &self.executed_trees
    }

    pub fn accu_root(&self) -> HashValue {
        self.executed_trees().state_id()
    }

    pub fn version(&self) -> Option<Version> {
        self.executed_trees().version()
    }

    pub fn epoch_info(&self) -> &Option<EpochInfo> {
        &self.epoch_info
    }

    pub fn state_compute_result(&self) -> StateComputeResult {
        let txn_accu = self.executed_trees().txn_accumulator();
        StateComputeResult {
            // Now that we have the root hash and execution status we can send the response to
            // consensus.
            // TODO: The VM will support a special transaction to set the validators for the
            // next epoch that is part of a block execution.
            root_hash: self.accu_root(),
            num_leaves: txn_accu.num_leaves(),
            epoch_info: self.epoch_info.clone(),
            frozen_subtree_roots: txn_accu.frozen_subtree_roots().clone(),
            compute_status: self
                .transaction_data()
                .iter()
                .map(|txn_data| txn_data.status())
                .cloned()
                .collect(),
            transaction_info_hashes: self
                .transaction_data()
                .iter()
                .filter_map(|x| x.txn_info_hash())
                .collect(),
        }
    }
}

/// A wrapper of the in-memory state sparse merkle tree and the transaction accumulator that
/// represent a specific state collectively. Usually it is a state after executing a block.
#[derive(Clone, Debug)]
pub struct ExecutedTrees {
    /// The in-memory Sparse Merkle Tree representing a specific state after execution. If this
    /// tree is presenting the latest commited state, it will have a single Subtree node (or
    /// Empty node) whose hash equals the root hash of the newest Sparse Merkle Tree in
    /// storage.
    state_tree: Arc<SparseMerkleTree>,

    /// The in-memory Merkle Accumulator representing a blockchain state consistent with the
    /// `state_tree`.
    transaction_accumulator: Arc<InMemoryAccumulator<TransactionAccumulatorHasher>>,
}

impl From<TreeState> for ExecutedTrees {
    fn from(tree_state: TreeState) -> Self {
        ExecutedTrees::new(
            tree_state.account_state_root_hash,
            tree_state.ledger_frozen_subtree_hashes,
            tree_state.num_transactions,
        )
    }
}

impl ExecutedTrees {
    pub fn new_copy(
        state_tree: Arc<SparseMerkleTree>,
        transaction_accumulator: Arc<InMemoryAccumulator<TransactionAccumulatorHasher>>,
    ) -> Self {
        Self {
            state_tree,
            transaction_accumulator,
        }
    }

    pub fn state_tree(&self) -> &Arc<SparseMerkleTree> {
        &self.state_tree
    }

    pub fn txn_accumulator(&self) -> &Arc<InMemoryAccumulator<TransactionAccumulatorHasher>> {
        &self.transaction_accumulator
    }

    pub fn version(&self) -> Option<Version> {
        let num_elements = self.txn_accumulator().num_leaves() as u64;
        if num_elements > 0 {
            Some(num_elements - 1)
        } else {
            None
        }
    }

    pub fn state_id(&self) -> HashValue {
        self.txn_accumulator().root_hash()
    }

    pub fn state_root(&self) -> HashValue {
        self.state_tree().root_hash()
    }

    pub fn new(
        state_root_hash: HashValue,
        frozen_subtrees_in_accumulator: Vec<HashValue>,
        num_leaves_in_accumulator: u64,
    ) -> ExecutedTrees {
        ExecutedTrees {
            state_tree: Arc::new(SparseMerkleTree::new(state_root_hash)),
            transaction_accumulator: Arc::new(
                InMemoryAccumulator::new(frozen_subtrees_in_accumulator, num_leaves_in_accumulator)
                    .expect("The startup info read from storage should be valid."),
            ),
        }
    }

    pub fn new_empty() -> ExecutedTrees {
        Self::new(*SPARSE_MERKLE_PLACEHOLDER_HASH, vec![], 0)
    }
}

pub struct ProofReader {
    account_to_proof: HashMap<HashValue, SparseMerkleProof>,
}

impl ProofReader {
    pub fn new(account_to_proof: HashMap<HashValue, SparseMerkleProof>) -> Self {
        ProofReader { account_to_proof }
    }
}

impl ProofRead for ProofReader {
    fn get_proof(&self, key: HashValue) -> Option<&SparseMerkleProof> {
        self.account_to_proof.get(&key)
    }
}
