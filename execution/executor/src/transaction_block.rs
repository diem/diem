// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block_tree::Block, ExecutedTrees, StateComputeResult};
use crypto::{hash::EventAccumulatorHasher, HashValue};
use failure::{format_err, Result};
use futures::channel::oneshot;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    crypto_proxies::LedgerInfoWithSignatures,
    proof::accumulator::Accumulator,
    transaction::{SignedTransaction, TransactionStatus},
};
use logger::prelude::*;
use scratchpad::SparseMerkleTree;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// `TransactionBlock` holds everything about the block of transactions.
#[derive(Debug)]
pub struct TransactionBlock {
    /// Whether consensus has decided to commit this block.
    committed: bool,

    /// Id of this block.
    id: HashValue,

    /// Id of the parent block.
    parent_id: HashValue,

    /// The set of children.
    children: HashSet<HashValue>,

    /// The transactions themselves.
    transactions: Vec<SignedTransaction>,

    /// The result of processing VM's output.
    output: Option<ProcessedVMOutput>,

    /// The signatures on this block. Not all committed blocks will have signatures, if multiple
    /// blocks are committed at once.
    ledger_info_with_sigs: Option<LedgerInfoWithSignatures>,

    /// The result for `execute_block` request.
    execute_response: Option<StateComputeResult>,

    /// The senders associated with this block. These senders are like the promises associated with
    /// the futures returned by `execute_block` and `commit_block` APIs, which are fulfilled when
    /// the responses are ready.
    execute_response_senders: Vec<oneshot::Sender<Result<StateComputeResult>>>,
    commit_response_sender: Option<oneshot::Sender<Result<()>>>,
}

impl TransactionBlock {
    /// Constructs a new block. A `TransactionBlock` is constructed as soon as consensus gives us a
    /// new block. It has not been executed yet so output is `None`.
    pub fn new(
        transactions: Vec<SignedTransaction>,
        parent_id: HashValue,
        id: HashValue,
        execute_response_sender: oneshot::Sender<Result<StateComputeResult>>,
    ) -> Self {
        TransactionBlock {
            committed: false,
            id,
            parent_id,
            children: HashSet::new(),
            transactions,
            output: None,
            ledger_info_with_sigs: None,
            execute_response: None,
            execute_response_senders: vec![execute_response_sender],
            commit_response_sender: None,
        }
    }

    /// Returns the list of transactions.
    pub fn transactions(&self) -> &[SignedTransaction] {
        &self.transactions
    }

    /// Returns the output of the block.
    pub fn output(&self) -> &Option<ProcessedVMOutput> {
        &self.output
    }

    /// Returns the signatures on this block.
    pub fn ledger_info_with_sigs(&self) -> &Option<LedgerInfoWithSignatures> {
        &self.ledger_info_with_sigs
    }

    /// Saves the response in the block. If there are any queued senders, send the response.
    pub fn set_execute_block_response(&mut self, response: StateComputeResult) {
        assert!(self.execute_response.is_none(), "Response is already set.");
        self.execute_response = Some(response.clone());
        // Send the response since it's now available.
        self.send_execute_block_response(Ok(response));
    }

    /// Puts a sender in the queue. The response will be sent via the sender once available
    /// (possibly as soon as the function is called if the response if already available).
    pub fn queue_execute_block_response_sender(
        &mut self,
        sender: oneshot::Sender<Result<StateComputeResult>>,
    ) {
        // If the response is already available, just send it. Otherwise store the sender for later
        // use.
        match self.execute_response {
            Some(ref response) => {
                if let Err(_err) = sender.send(Ok(response.clone())) {
                    warn!("Failed to send execute block response.");
                }
            }
            None => self.execute_response_senders.push(sender),
        }
    }

    /// Sends finished `ExecuteBlockResponse` to consensus. This removes all the existing senders.
    pub fn send_execute_block_response(&mut self, response: Result<StateComputeResult>) {
        while let Some(sender) = self.execute_response_senders.pop() {
            // We need to send the result multiple times, but the error is not cloneable, thus the
            // result is not cloneable. This is a bit workaround.
            let resp = match &response {
                Ok(resp) => Ok(resp.clone()),
                Err(err) => Err(format_err!("{}", err)),
            };
            if let Err(_err) = sender.send(resp) {
                warn!("Failed to send execute block response.");
            }
        }
    }

    /// When the block is created, no one has called `commit_block` on this block yet, so we do not
    /// have the sender and `self.commit_response_sender` is initialized to `None`. When consensus
    /// calls `commit_block` on a block, we will put the sender in the block. So when this block is
    /// persisted in storage later, we will call `send_commit_response` and consensus will receive
    /// the response.
    pub fn set_commit_response_sender(
        &mut self,
        commit_response_sender: oneshot::Sender<Result<()>>,
    ) {
        assert!(
            self.commit_response_sender.is_none(),
            "CommitBlockResponse sender should not exist."
        );
        self.commit_response_sender = Some(commit_response_sender);
    }

    /// Sends finished `CommitBlockResponse` to consensus.
    pub fn send_commit_block_response(&mut self) {
        let sender = self
            .commit_response_sender
            .take()
            .expect("CommitBlockResponse sender should exist.");
        if let Err(_err) = sender.send(Ok(())) {
            warn!("Failed to send commit block response:.");
        }
    }

    /// Returns a pointer to the executed trees representing the state at the end of the block.
    /// Should only be called when the block has finished execution and `set_output` has been
    /// called.
    pub fn executed_trees(&self) -> &ExecutedTrees {
        self.output
            .as_ref()
            .expect("The block has no output yet.")
            .executed_trees()
    }
}

impl Block for TransactionBlock {
    type Output = ProcessedVMOutput;
    type Signature = LedgerInfoWithSignatures;

    fn is_committed(&self) -> bool {
        self.committed
    }

    fn set_committed(&mut self) {
        assert!(!self.committed);
        self.committed = true;
    }

    fn is_executed(&self) -> bool {
        self.output.is_some()
    }

    fn set_output(&mut self, output: Self::Output) {
        assert!(self.output.is_none(), "Output is already set.");
        self.output = Some(output);
    }

    fn set_signature(&mut self, signature: Self::Signature) {
        assert!(
            self.ledger_info_with_sigs.is_none(),
            "Signature is already set."
        );
        self.ledger_info_with_sigs = Some(signature);
    }

    fn id(&self) -> HashValue {
        self.id
    }

    fn parent_id(&self) -> HashValue {
        self.parent_id
    }

    fn add_child(&mut self, child_id: HashValue) {
        assert!(self.children.insert(child_id));
    }

    fn children(&self) -> &HashSet<HashValue> {
        &self.children
    }
}

impl Drop for TransactionBlock {
    fn drop(&mut self) {
        // It is possible for a block to be discarded before it gets executed, for example, due to
        // a parallel block getting committed. In this case we still want to send a response back.
        if !self.execute_response_senders.is_empty() {
            assert!(self.execute_response.is_none());
            self.send_execute_block_response(Err(format_err!("Block {} is discarded.", self.id)));
        }
    }
}

/// The entire set of data associated with a transaction. In addition to the output generated by VM
/// which includes the write set and events, this also has the in-memory trees.
#[derive(Debug)]
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
    state_tree: Rc<SparseMerkleTree>,

    /// The in-memory Merkle Accumulator that has all events emitted by this transaction.
    event_tree: Rc<Accumulator<EventAccumulatorHasher>>,

    /// The amount of gas used.
    gas_used: u64,

    /// The number of newly created accounts.
    num_account_created: usize,
}

impl TransactionData {
    pub fn new(
        account_blobs: HashMap<AccountAddress, AccountStateBlob>,
        events: Vec<ContractEvent>,
        status: TransactionStatus,
        state_tree: Rc<SparseMerkleTree>,
        event_tree: Rc<Accumulator<EventAccumulatorHasher>>,
        gas_used: u64,
        num_account_created: usize,
    ) -> Self {
        TransactionData {
            account_blobs,
            events,
            status,
            state_tree,
            event_tree,
            gas_used,
            num_account_created,
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

    pub fn num_account_created(&self) -> usize {
        self.num_account_created
    }

    pub fn prune_state_tree(&self) {
        self.state_tree.prune()
    }
}

/// Generated by processing VM's output.
#[derive(Debug)]
pub struct ProcessedVMOutput {
    /// The entire set of data associated with each transaction.
    transaction_data: Vec<TransactionData>,

    /// The in-memory Merkle Accumulator and state Sparse Merkle Tree after appending all the
    /// transactions in this set.
    executed_trees: ExecutedTrees,
}

impl ProcessedVMOutput {
    pub fn new(transaction_data: Vec<TransactionData>, executed_trees: ExecutedTrees) -> Self {
        ProcessedVMOutput {
            transaction_data,
            executed_trees,
        }
    }

    pub fn transaction_data(&self) -> &[TransactionData] {
        &self.transaction_data
    }

    pub fn executed_trees(&self) -> &ExecutedTrees {
        &self.executed_trees
    }
}
