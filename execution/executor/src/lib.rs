// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod block_processor;
mod block_tree;
mod transaction_block;

#[cfg(test)]
mod executor_test;
#[cfg(test)]
mod mock_vm;

use crate::block_processor::BlockProcessor;
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer};
use config::config::NodeConfig;
use crypto::{
    hash::{
        TransactionAccumulatorHasher, ACCUMULATOR_PLACEHOLDER_HASH, GENESIS_BLOCK_ID,
        PRE_GENESIS_BLOCK_ID, SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
use failure::{format_err, Result};
use futures::{channel::oneshot, executor::block_on};
use lazy_static::lazy_static;
use libra_types::{
    crypto_proxies::LedgerInfoWithSignatures,
    ledger_info::LedgerInfo,
    proof::accumulator::Accumulator,
    transaction::{Transaction, TransactionListWithProof, TransactionStatus, Version},
    validator_set::ValidatorSet,
};
use logger::prelude::*;
use scratchpad::SparseMerkleTree;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
};
use storage_client::{StorageRead, StorageWrite};
use vm_runtime::VMExecutor;

lazy_static! {
    static ref OP_COUNTERS: metrics::OpMetrics = metrics::OpMetrics::new_and_registered("executor");
}

/// A structure that specifies the result of the execution.
/// The execution is responsible for generating the ID of the new state, which is returned in the
/// result.
///
/// Not every transaction in the payload succeeds: the returned vector keeps the boolean status
/// of success / failure of the transactions.
/// Note that the specific details of compute_status are opaque to StateMachineReplication,
/// which is going to simply pass the results between StateComputer and TxnManager.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct StateComputeResult {
    pub executed_state: ExecutedState,
    /// The compute status (success/failure) of the given payload. The specific details are opaque
    /// for StateMachineReplication, which is merely passing it between StateComputer and
    /// TxnManager.
    pub compute_status: Vec<TransactionStatus>,
}

impl StateComputeResult {
    pub fn version(&self) -> Version {
        self.executed_state.version
    }

    pub fn root_hash(&self) -> HashValue {
        self.executed_state.state_id
    }

    pub fn status(&self) -> &Vec<TransactionStatus> {
        &self.compute_status
    }
}

/// Executed state derived from StateComputeResult that is maintained with every proposed block.
/// `state_id`(transaction accumulator root hash) summarized both the information of the version and
/// the validators.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutedState {
    /// Tracks the execution state of a proposed block
    pub state_id: HashValue,
    /// Version of after executing a proposed block.  This state must be persisted to ensure
    /// that on restart that the version is calculated correctly
    pub version: Version,
    /// If set, this is the validator set that should be changed to if this block is committed.
    /// TODO [Reconfiguration] the validators are currently ignored, no reconfiguration yet.
    pub validators: Option<ValidatorSet>,
}

impl ExecutedState {
    pub fn state_for_genesis() -> Self {
        ExecutedState {
            state_id: *ACCUMULATOR_PLACEHOLDER_HASH,
            version: 0,
            validators: None,
        }
    }
}

impl CanonicalSerialize for ExecutedState {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer.encode_bytes(self.state_id.as_ref())?;
        serializer.encode_u64(self.version)?;
        if let Some(validators) = &self.validators {
            serializer.encode_struct(validators)?;
        }
        Ok(())
    }
}

/// `Executor` implements all functionalities the execution module needs to provide.
pub struct Executor<V> {
    /// A thread that keeps processing blocks.
    block_processor_thread: Option<std::thread::JoinHandle<()>>,

    /// Where we can send command to the block processor. The block processor sits at the other end
    /// of the channel and processes the commands.
    command_sender: Mutex<Option<mpsc::Sender<Command>>>,

    phantom: PhantomData<V>,
}

impl<V> Executor<V>
    where
        V: VMExecutor,
{
    /// Constructs an `Executor`.
    pub fn new(
        storage_read_client: Arc<dyn StorageRead>,
        storage_write_client: Arc<dyn StorageWrite>,
        config: &NodeConfig,
    ) -> Self {
        let startup_info = storage_read_client
            .get_startup_info()
            .expect("Failed to read startup info from storage.");

        let (
            state_root_hash,
            frozen_subtrees_in_accumulator,
            num_leaves_in_accumulator,
            committed_timestamp_usecs,
            committed_block_id,
        ) = match startup_info {
            Some(info) => {
                info!("Startup info read from DB: {:?}.", info);
                let ledger_info = info.ledger_info;
                (
                    info.account_state_root_hash,
                    info.ledger_frozen_subtree_hashes,
                    info.latest_version + 1,
                    ledger_info.timestamp_usecs(),
                    ledger_info.consensus_block_id(),
                )
            }
            None => {
                info!("Startup info is empty. Will start from GENESIS.");
                (
                    *SPARSE_MERKLE_PLACEHOLDER_HASH,
                    vec![],
                    0,
                    0,
                    *PRE_GENESIS_BLOCK_ID,
                )
            }
        };

        let (command_sender, command_receiver) = mpsc::channel();

        let vm_config = config.vm_config.clone();
        let executor = Executor {
            block_processor_thread: Some(
                std::thread::Builder::new()
                    .name("block_processor".into())
                    .spawn(move || {
                        let mut block_processor = BlockProcessor::<V>::new(
                            command_receiver,
                            committed_timestamp_usecs,
                            state_root_hash,
                            frozen_subtrees_in_accumulator,
                            num_leaves_in_accumulator,
                            committed_block_id,
                            storage_read_client,
                            storage_write_client,
                            vm_config,
                        );
                        block_processor.run();
                    })
                    .expect("Failed to create block processor thread."),
            ),
            command_sender: Mutex::new(Some(command_sender)),
            phantom: PhantomData,
        };

        if committed_block_id == *PRE_GENESIS_BLOCK_ID {
            let genesis_transaction = config
                .get_genesis_transaction()
                .expect("failed to load genesis transaction!");
            executor.init_genesis(genesis_transaction);
        }

        executor
    }

    /// This is used when we start for the first time and the DB is completely empty. It will write
    /// necessary information to DB by committing the genesis transaction.
    fn init_genesis(&self, genesis_txn: Transaction) {
        // Create a block with genesis_txn being the only transaction. Execute it then commit it
        // immediately.
        // We create `PRE_GENESIS_BLOCK_ID` as the parent of the genesis block.
        let state_compute_result = block_on(self.execute_block(
            vec![genesis_txn],
            *PRE_GENESIS_BLOCK_ID,
            *GENESIS_BLOCK_ID,
        ))
            .expect("Response sender was unexpectedly dropped.")
            .expect("Failed to execute genesis block.");

        let root_hash = state_compute_result.executed_state.state_id;
        let ledger_info = LedgerInfo::new(
            /* version = */ 0,
            root_hash,
            /* consensus_data_hash = */ HashValue::zero(),
            *GENESIS_BLOCK_ID,
            /* epoch_num = */ 0,
            /* timestamp_usecs = */ 0,
            None,
        );
        let ledger_info_with_sigs = LedgerInfoWithSignatures::new(ledger_info, BTreeMap::new());
        block_on(self.commit_block(ledger_info_with_sigs))
            .expect("Response sender was unexpectedly dropped.")
            .expect("Failed to commit genesis block.");
        info!("GENESIS transaction is committed.")
    }

    /// Executes a block.
    pub fn execute_block(
        &self,
        transactions: Vec<Transaction>,
        parent_id: HashValue,
        id: HashValue,
    ) -> oneshot::Receiver<Result<StateComputeResult>> {
        debug!(
            "Received request to execute block. Parent id: {:x}. Id: {:x}.",
            parent_id, id
        );

        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::ExecuteBlock {
                        transactions,
                        parent_id,
                        id,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }

    /// Executes a block.
    pub fn pre_execute_block(
        &self,
        transactions: Vec<Transaction>,
        parent_state_id: HashValue,
    ) -> oneshot::Receiver<Result<StateComputeResult>> {
        debug!(
            "Received request to pre execute block. Parent state id: {:x}.",
            parent_state_id
        );

        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::PreExecuteBlock {
                        transactions,
                        parent_state_id,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }

    pub fn commit_block(
        &self,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> oneshot::Receiver<Result<()>> {
        debug!(
            "Received request to commit block {:x}.",
            ledger_info_with_sigs.ledger_info().consensus_block_id()
        );

        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::CommitBlock {
                        ledger_info_with_sigs,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }

    /// Commits a block and all its ancestors. Returns `Ok(())` if successful.
    pub fn commit_block_with_id(
        &self,
        block_id:HashValue,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> oneshot::Receiver<Result<()>> {
        debug!(
            "Received request to commit block {:x}.",
            ledger_info_with_sigs.ledger_info().consensus_block_id()
        );

        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::CommitBlockWithId {
                        block_id,
                        ledger_info_with_sigs,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }

    /// Executes and commits a chunk of transactions that are already committed by majority of the
    /// validators.
    pub fn execute_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> oneshot::Receiver<Result<()>> {
        debug!(
            "Received request to execute chunk. Chunk size: {}. Target version: {}.",
            txn_list_with_proof.transaction_and_infos.len(),
            ledger_info_with_sigs.ledger_info().version(),
        );

        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::ExecuteChunk {
                        txn_list_with_proof,
                        ledger_info_with_sigs,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }

    /// Rollback
    pub fn rollback_by_block_id(
        &self,
        block_id: HashValue,
    ) -> oneshot::Receiver<Result<()>> {
        let (resp_sender, resp_receiver) = oneshot::channel();
        match self
            .command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .as_ref()
            {
                Some(sender) => sender
                    .send(Command::RollbackBlock {
                        block_id,
                        resp_sender,
                    })
                    .expect("Did block processor thread panic?"),
                None => resp_sender
                    .send(Err(format_err!("Executor is shutting down.")))
                    .expect("Failed to send error message."),
            }
        resp_receiver
    }
}

impl<V> Drop for Executor<V> {
    fn drop(&mut self) {
        // Drop the sender so the block processor thread will exit.
        self.command_sender
            .lock()
            .expect("Failed to lock mutex.")
            .take()
            .expect("Command sender should exist.");
        self.block_processor_thread
            .take()
            .expect("Block processor thread should exist.")
            .join()
            .expect("Did block processor thread panic?");
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum Command {
    ExecuteBlock {
        transactions: Vec<Transaction>,
        parent_id: HashValue,
        id: HashValue,
        resp_sender: oneshot::Sender<Result<StateComputeResult>>,
    },
    PreExecuteBlock {
        transactions: Vec<Transaction>,
        parent_state_id: HashValue,
        resp_sender: oneshot::Sender<Result<StateComputeResult>>,
    },
    CommitBlock {
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        resp_sender: oneshot::Sender<Result<()>>,
    },
    CommitBlockWithId {
        block_id: HashValue,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        resp_sender: oneshot::Sender<Result<()>>,
    },
    ExecuteChunk {
        txn_list_with_proof: TransactionListWithProof,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        resp_sender: oneshot::Sender<Result<()>>,
    },
    RollbackBlock {
        block_id: HashValue,
        resp_sender: oneshot::Sender<Result<()>>,
    },
}

#[derive(Clone, Debug)]
pub struct ExecutedTrees {
    /// The in-memory Sparse Merkle Tree representing a specific state after execution. If this
    /// tree is presenting the latest commited state, it will have a single Subtree node (or
    /// Empty node) whose hash equals the root hash of the newest Sparse Merkle Tree in
    /// storage.
    state_tree: Rc<SparseMerkleTree>,

    /// The in-memory Merkle Accumulator representing a blockchain state consistent with the
    /// `state_tree`.
    transaction_accumulator: Rc<Accumulator<TransactionAccumulatorHasher>>,
}

impl ExecutedTrees {
    pub fn state_tree(&self) -> &Rc<SparseMerkleTree> {
        &self.state_tree
    }

    pub fn txn_accumulator(&self) -> &Rc<Accumulator<TransactionAccumulatorHasher>> {
        &self.transaction_accumulator
    }

    pub fn version_and_state_root(&self) -> (Option<Version>, HashValue) {
        let num_elements = self.txn_accumulator().num_leaves() as u64;
        let version = if num_elements > 0 {
            Some(num_elements - 1)
        } else {
            None
        };
        (version, self.state_tree().root_hash())
    }

    /// Reset ExecutedTrees
    pub fn reset(&mut self, state_tree: Rc<SparseMerkleTree>, transaction_accumulator: Rc<Accumulator<TransactionAccumulatorHasher>>) {
        let mut executed_trees = ExecutedTrees { state_tree, transaction_accumulator };
        std::mem::swap(self, &mut executed_trees);
    }
}
