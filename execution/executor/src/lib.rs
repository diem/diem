// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
#![allow(dead_code)]

#[cfg(test)]
mod executor_test;
#[cfg(test)]
mod mock_vm;

use anyhow::{bail, ensure, format_err, Result};
use debug_interface::prelude::*;
use executor_types::{ExecutedTrees, ProcessedVMOutput, ProofReader, TransactionData};
use futures::executor::block_on;
use libra_config::config::{NodeConfig, VMConfig};
use libra_crypto::hash::GENESIS_BLOCK_ID;
use libra_crypto::{
    hash::{CryptoHash, EventAccumulatorHasher},
    HashValue,
};
use libra_logger::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorSet},
    proof::{accumulator::InMemoryAccumulator, definition::LeafCount, SparseMerkleProof},
    transaction::{
        Transaction, TransactionInfo, TransactionListWithProof, TransactionOutput,
        TransactionPayload, TransactionStatus, TransactionToCommit, Version,
    },
    write_set::{WriteOp, WriteSet},
};
use libra_vm::VMExecutor;
use once_cell::sync::Lazy;
use scratchpad::SparseMerkleTree;
use std::{
    collections::{hash_map, HashMap, HashSet},
    convert::TryFrom,
    marker::PhantomData,
    sync::Arc,
};
use storage_client::{StorageRead, StorageWrite, VerifiedStateView};
use tokio::runtime::Runtime;

static OP_COUNTERS: Lazy<libra_metrics::OpMetrics> =
    Lazy::new(|| libra_metrics::OpMetrics::new_and_registered("executor"));

/// `Executor` implements all functionalities the execution module needs to provide.
pub struct Executor<V> {
    rt: Runtime,

    /// Client to storage service.
    storage_read_client: Arc<dyn StorageRead>,
    storage_write_client: Arc<dyn StorageWrite>,

    /// Configuration for the VM. The block processor currently creates a new VM for each block.
    vm_config: VMConfig,

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
        let rt = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .thread_name("tokio-executor")
            .build()
            .unwrap();

        let mut executor = Executor {
            rt,
            storage_read_client: storage_read_client.clone(),
            storage_write_client,
            vm_config: config.vm_config.clone(),
            phantom: PhantomData,
        };

        let startup_info = block_on(executor.rt.spawn(async move {
            storage_read_client
                .get_startup_info()
                .await
                .expect("Shouldn't fail")
        }))
        .unwrap();

        if startup_info.is_none() {
            let genesis_txn = config
                .execution
                .genesis
                .as_ref()
                .expect("failed to load genesis transaction!")
                .clone();
            executor.init_genesis(genesis_txn);
        }
        executor
    }

    /// This is used when we start for the first time and the DB is completely empty. It will write
    /// necessary information to DB by committing the genesis transaction.
    fn init_genesis(&mut self, genesis_txn: Transaction) {
        let genesis_txns = vec![genesis_txn];

        // Create a block with genesis_txn being the only transaction. Execute it then commit it
        // immediately.
        // We create `PRE_GENESIS_BLOCK_ID` as the parent of the genesis block.
        let pre_genesis_trees = ExecutedTrees::new_empty();
        let output = self
            .execute_block(
                *GENESIS_BLOCK_ID,
                genesis_txns.clone(),
                &pre_genesis_trees,
                &pre_genesis_trees,
            )
            .expect("Failed to execute genesis block.");

        let validator_set = output
            .validators()
            .clone()
            .expect("Genesis transaction must emit a validator set.");

        let root_hash = output.accu_root();
        let ledger_info_with_sigs = LedgerInfoWithSignatures::genesis(root_hash, validator_set);

        self.commit_blocks(
            vec![(genesis_txns, Arc::new(output))],
            ledger_info_with_sigs,
            &pre_genesis_trees,
        )
        .expect("Failed to commit genesis block.");
        info!("GENESIS transaction is committed.")
    }

    /// Executes a block.
    pub fn execute_block(
        &self,
        block_id: HashValue,
        transactions: Vec<Transaction>,
        parent_trees: &ExecutedTrees,
        committed_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput> {
        let _timer = OP_COUNTERS.timer("block_execute_time_s");
        // Construct a StateView and pass the transactions to VM.
        let state_view = VerifiedStateView::new(
            Arc::clone(&self.storage_read_client),
            self.rt.handle().clone(),
            committed_trees.version(),
            committed_trees.state_root(),
            parent_trees.state_tree(),
        );

        let vm_outputs = {
            trace_code_block!("executor::execute_block", {"block", block_id});
            let _timer = OP_COUNTERS.timer("vm_execute_block_time_s");
            V::execute_block(transactions.clone(), &self.vm_config, &state_view)?
        };

        trace_code_block!("executor::process_vm_outputs", {"block", block_id});
        let status: Vec<_> = vm_outputs
            .iter()
            .map(TransactionOutput::status)
            .cloned()
            .collect();
        if !status.is_empty() {
            trace!("Execution status: {:?}", status);
        }

        let (account_to_state, account_to_proof) = state_view.into();
        let output = Self::process_vm_outputs(
            account_to_state,
            account_to_proof,
            &transactions,
            vm_outputs,
            parent_trees,
        )
        .map_err(|err| format_err!("Failed to execute block: {}", err))?;

        Ok(output)
    }

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
    pub fn commit_blocks(
        &self,
        blocks: Vec<(Vec<Transaction>, Arc<ProcessedVMOutput>)>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
        synced_trees: &ExecutedTrees,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>)> {
        debug!(
            "Received request to commit block {:x}.",
            ledger_info_with_sigs.ledger_info().consensus_block_id()
        );
        let num_persistent_txns = synced_trees.txn_accumulator().num_leaves();

        // All transactions that need to go to storage. In the above example, this means all the
        // transactions in A, B and C whose status == TransactionStatus::Keep.
        // This must be done before calculate potential skipping of transactions in idempotent commit.
        let mut txns_to_keep = vec![];
        let mut blocks_txns = vec![];
        let mut reconfig_events = vec![];
        for (txn, txn_data) in blocks
            .iter()
            .flat_map(|block| itertools::zip_eq(&block.0, block.1.transaction_data()))
        {
            if let TransactionStatus::Keep(_) = txn_data.status() {
                txns_to_keep.push((
                    TransactionToCommit::new(
                        txn.clone(),
                        txn_data.account_blobs().clone(),
                        txn_data.events().to_vec(),
                        txn_data.gas_used(),
                        txn_data.status().vm_status().major_status,
                    ),
                    txn_data.num_account_created(),
                ));
                blocks_txns.push(txn.clone());
                reconfig_events.append(&mut Self::extract_reconfig_events(
                    txn_data.events().to_vec(),
                ));
            }
        }

        let last_block = blocks
            .last()
            .expect("CommittableBlockBatch has at least 1 block.");

        // Check that the version in ledger info (computed by consensus) matches the version
        // computed by us.
        let version = ledger_info_with_sigs.ledger_info().version();
        let num_txns_in_speculative_accumulator =
            last_block.1.executed_trees().txn_accumulator().num_leaves();
        assert_eq!(
            version + 1,
            num_txns_in_speculative_accumulator as Version,
            "Number of transactions in ledger info ({}) does not match number of transactions \
             in accumulator ({}).",
            version + 1,
            num_txns_in_speculative_accumulator,
        );

        let num_txns_to_keep = txns_to_keep.len() as u64;

        // Skip txns that are already committed to allow failures in state sync process.
        let first_version_to_keep = version + 1 - num_txns_to_keep;
        assert!(
            first_version_to_keep <= num_persistent_txns,
            "first_version {} in the blocks to commit cannot exceed # of committed txns: {}.",
            first_version_to_keep,
            num_persistent_txns
        );

        let num_txns_to_skip = num_persistent_txns - first_version_to_keep;
        let first_version_to_commit = first_version_to_keep + num_txns_to_skip;
        if num_txns_to_skip != 0 {
            info!(
                "The lastest committed/synced version: {}, the first version to keep in the batch: {}.\
                 Skipping the first {} transactions and start committing from version {}",
                num_persistent_txns - 1, /* latest persistent version */
                first_version_to_keep,
                num_txns_to_skip,
                first_version_to_commit
            );
        }

        // Skip duplicate txns that are already persistent.
        let (txns_to_commit, list_num_account_created): (Vec<_>, Vec<_>) = txns_to_keep
            .into_iter()
            .skip(num_txns_to_skip as usize)
            .unzip();

        let num_txns_to_commit = txns_to_commit.len() as u64;

        {
            let _timer = OP_COUNTERS.timer("storage_save_transactions_time_s");
            OP_COUNTERS.observe("storage_save_transactions.count", num_txns_to_commit as f64);
            assert_eq!(first_version_to_commit, version + 1 - num_txns_to_commit);
            let write_client = self.storage_write_client.clone();
            block_on(self.rt.spawn(async move {
                write_client
                    .save_transactions(
                        txns_to_commit,
                        first_version_to_commit,
                        Some(ledger_info_with_sigs),
                    )
                    .await
            }))
            .unwrap()?;
        }
        // Only bump the counter when the commit succeeds.
        OP_COUNTERS.inc_by("num_accounts", list_num_account_created.into_iter().sum());

        for block in blocks {
            for txn_data in block.1.transaction_data() {
                txn_data.prune_state_tree();
            }
        }
        // Now that the blocks are persisted successfully, we can reply to consensus
        Ok((blocks_txns, reconfig_events))
    }

    /// Verifies the transactions based on the provided proofs and ledger info. If the transactions
    /// are valid, executes them and commits immediately if execution results match the proofs.
    /// Returns a vector of reconfiguration events in the chunk
    pub fn execute_and_commit_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        // Target LI that has been verified independently: the proofs are relative to this version.
        verified_target_li: LedgerInfoWithSignatures,
        // An optional end of epoch LedgerInfo. We do not allow chunks that end epoch without
        // carrying any epoch change LI.
        epoch_change_li: Option<LedgerInfoWithSignatures>,
        synced_trees: &mut ExecutedTrees,
    ) -> Result<Vec<ContractEvent>> {
        info!(
            "Local synced version: {}. First transaction version in request: {:?}. \
             Number of transactions in request: {}.",
            synced_trees.txn_accumulator().num_leaves() - 1,
            txn_list_with_proof.first_transaction_version,
            txn_list_with_proof.transactions.len(),
        );

        let (num_txns_to_skip, first_version) = Self::verify_chunk(
            &txn_list_with_proof,
            &verified_target_li,
            synced_trees.txn_accumulator().num_leaves(),
        )?;

        info!("Skipping the first {} transactions.", num_txns_to_skip);
        let transactions: Vec<_> = txn_list_with_proof
            .transactions
            .into_iter()
            .skip(num_txns_to_skip as usize)
            .collect();

        // Construct a StateView and pass the transactions to VM.
        let state_view = VerifiedStateView::new(
            Arc::clone(&self.storage_read_client),
            self.rt.handle().clone(),
            synced_trees.version(),
            synced_trees.state_root(),
            synced_trees.state_tree(),
        );
        let vm_outputs = {
            let _timer = OP_COUNTERS.timer("vm_execute_chunk_time_s");
            V::execute_block(transactions.to_vec(), &self.vm_config, &state_view)?
        };

        // Since other validators have committed these transactions, their status should all be
        // TransactionStatus::Keep.
        for output in &vm_outputs {
            if let TransactionStatus::Discard(_) = output.status() {
                bail!("Syncing transactions that should be discarded.");
            }
        }

        let (account_to_state, account_to_proof) = state_view.into();

        let output = Self::process_vm_outputs(
            account_to_state,
            account_to_proof,
            &transactions,
            vm_outputs,
            synced_trees,
        )?;

        // Since we have verified the proofs, we just need to verify that each TransactionInfo
        // object matches what we have computed locally.
        let mut txns_to_commit = vec![];
        let mut reconfig_events = vec![];
        for (txn, txn_data) in itertools::zip_eq(transactions, output.transaction_data()) {
            txns_to_commit.push(TransactionToCommit::new(
                txn,
                txn_data.account_blobs().clone(),
                txn_data.events().to_vec(),
                txn_data.gas_used(),
                txn_data.status().vm_status().major_status,
            ));
            reconfig_events.append(&mut Self::extract_reconfig_events(
                txn_data.events().to_vec(),
            ));
        }

        let ledger_info_to_commit =
            Self::find_chunk_li(verified_target_li, epoch_change_li, &output)?;
        if ledger_info_to_commit.is_none() && txns_to_commit.is_empty() {
            return Ok(reconfig_events);
        }
        let write_client = self.storage_write_client.clone();
        let ledger_info = ledger_info_to_commit.clone();
        block_on(self.rt.spawn(async move {
            write_client
                .save_transactions(txns_to_commit, first_version, ledger_info)
                .await
        }))
        .unwrap()?;

        *synced_trees = output.executed_trees().clone();
        info!(
            "Synced to version {}, the corresponding LedgerInfo is {}.",
            synced_trees.version().expect("version must exist"),
            if ledger_info_to_commit.is_some() {
                "committed"
            } else {
                "not committed"
            },
        );
        Ok(reconfig_events)
    }

    /// In case there is a new LI to be added to a LedgerStore, verify and return it.
    fn find_chunk_li(
        verified_target_li: LedgerInfoWithSignatures,
        epoch_change_li: Option<LedgerInfoWithSignatures>,
        new_output: &ProcessedVMOutput,
    ) -> Result<Option<LedgerInfoWithSignatures>> {
        // If the chunk corresponds to the target LI, the target LI can be added to storage.
        if verified_target_li.ledger_info().version() == new_output.version().unwrap_or(0) {
            ensure!(
                verified_target_li
                    .ledger_info()
                    .transaction_accumulator_hash()
                    == new_output.accu_root(),
                "Root hash in target ledger info does not match local computation."
            );
            return Ok(Some(verified_target_li));
        }
        // If the epoch change LI is present, it must match the version of the chunk:
        // verify the version and the root hash.
        if let Some(epoch_change_li) = epoch_change_li {
            // Verify that the given ledger info corresponds to the new accumulator.
            ensure!(
                epoch_change_li.ledger_info().transaction_accumulator_hash()
                    == new_output.accu_root(),
                "Root hash of a given epoch LI does not match local computation."
            );
            ensure!(
                epoch_change_li.ledger_info().version() == new_output.version().unwrap_or(0),
                "Version of a given epoch LI does not match local computation."
            );
            ensure!(
                epoch_change_li.ledger_info().next_validator_set().is_some(),
                "Epoch change LI does not carry validator set"
            );
            ensure!(
                epoch_change_li.ledger_info().next_validator_set()
                    == new_output.validators().as_ref(),
                "New validator set of a given epoch LI does not match local computation"
            );
            return Ok(Some(epoch_change_li));
        }
        ensure!(
            new_output.validators().is_none(),
            "End of epoch chunk based on local computation but no EoE LedgerInfo provided."
        );
        Ok(None)
    }

    /// Verifies proofs using provided ledger info. Also verifies that the version of the first
    /// transaction matches the latest committed transaction. If the first few transaction happens
    /// to be older, returns how many need to be skipped and the first version to be committed.
    fn verify_chunk(
        txn_list_with_proof: &TransactionListWithProof,
        ledger_info_with_sigs: &LedgerInfoWithSignatures,
        num_committed_txns: u64,
    ) -> Result<(LeafCount, Version)> {
        txn_list_with_proof.verify(
            ledger_info_with_sigs.ledger_info(),
            txn_list_with_proof.first_transaction_version,
        )?;

        if txn_list_with_proof.transactions.is_empty() {
            return Ok((0, num_committed_txns as Version /* first_version */));
        }

        let first_txn_version = txn_list_with_proof
            .first_transaction_version
            .expect("first_transaction_version should exist.")
            as Version;

        ensure!(
            first_txn_version <= num_committed_txns,
            "Transaction list too new. Expected version: {}. First transaction version: {}.",
            num_committed_txns,
            first_txn_version
        );
        Ok((
            num_committed_txns - first_txn_version,
            num_committed_txns as Version,
        ))
    }

    /// Post-processing of what the VM outputs. Returns the entire block's output.
    fn process_vm_outputs(
        mut account_to_state: HashMap<AccountAddress, AccountState>,
        account_to_proof: HashMap<HashValue, SparseMerkleProof>,
        transactions: &[Transaction],
        vm_outputs: Vec<TransactionOutput>,
        parent_trees: &ExecutedTrees,
    ) -> Result<ProcessedVMOutput> {
        // The data of each individual transaction. For convenience purpose, even for the
        // transactions that will be discarded, we will compute its in-memory Sparse Merkle Tree
        // (it will be identical to the previous one).
        let mut txn_data = vec![];
        let mut current_state_tree = Arc::clone(parent_trees.state_tree());
        // The hash of each individual TransactionInfo object. This will not include the
        // transactions that will be discarded, since they do not go into the transaction
        // accumulator.
        let mut txn_info_hashes = vec![];
        let mut next_validator_set = None;

        let proof_reader = ProofReader::new(account_to_proof);
        let validator_set_change_event_key = ValidatorSet::change_event_key();
        for (vm_output, txn) in itertools::zip_eq(vm_outputs.into_iter(), transactions.iter()) {
            if next_validator_set.is_some() {
                txn_data.push(TransactionData::new(
                    HashMap::new(),
                    vec![],
                    TransactionStatus::Retry,
                    Arc::clone(&current_state_tree),
                    Arc::new(InMemoryAccumulator::<EventAccumulatorHasher>::default()),
                    0,
                    0,
                    None,
                ));
                continue;
            }
            let (blobs, state_tree, num_accounts_created) = Self::process_write_set(
                txn,
                &mut account_to_state,
                &proof_reader,
                vm_output.write_set().clone(),
                &current_state_tree,
            )?;

            let event_tree = {
                let event_hashes: Vec<_> =
                    vm_output.events().iter().map(CryptoHash::hash).collect();
                InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes)
            };

            let mut txn_info_hash = None;
            match vm_output.status() {
                TransactionStatus::Keep(status) => {
                    ensure!(
                        !vm_output.write_set().is_empty(),
                        "Transaction with empty write set should be discarded.",
                    );
                    // Compute hash for the TransactionInfo object. We need the hash of the
                    // transaction itself, the state root hash as well as the event root hash.
                    let txn_info = TransactionInfo::new(
                        txn.hash(),
                        state_tree.root_hash(),
                        event_tree.root_hash(),
                        vm_output.gas_used(),
                        status.major_status,
                    );

                    let real_txn_info_hash = txn_info.hash();
                    txn_info_hashes.push(real_txn_info_hash);
                    txn_info_hash = Some(real_txn_info_hash);
                }
                TransactionStatus::Discard(status) => {
                    if !vm_output.write_set().is_empty() || !vm_output.events().is_empty() {
                        crit!(
                            "Discarded transaction has non-empty write set or events. \
                             Transaction: {:?}. Status: {}.",
                            txn,
                            status,
                        );
                    }
                }
                TransactionStatus::Retry => (),
            }

            txn_data.push(TransactionData::new(
                blobs,
                vm_output.events().to_vec(),
                vm_output.status().clone(),
                Arc::clone(&state_tree),
                Arc::new(event_tree),
                vm_output.gas_used(),
                num_accounts_created,
                txn_info_hash,
            ));
            current_state_tree = state_tree;

            // check for change in validator set
            next_validator_set = vm_output
                .events()
                .iter()
                .find(|event| *event.key() == validator_set_change_event_key)
                .map(|event| ValidatorSet::from_bytes(event.event_data()))
                .transpose()?
        }

        let current_transaction_accumulator =
            parent_trees.txn_accumulator().append(&txn_info_hashes);
        Ok(ProcessedVMOutput::new(
            txn_data,
            ExecutedTrees::new_copy(
                current_state_tree,
                Arc::new(current_transaction_accumulator),
            ),
            next_validator_set,
        ))
    }

    /// For all accounts modified by this transaction, find the previous blob and update it based
    /// on the write set. Returns the blob value of all these accounts as well as the newly
    /// constructed state tree.
    fn process_write_set(
        transaction: &Transaction,
        account_to_state: &mut HashMap<AccountAddress, AccountState>,
        proof_reader: &ProofReader,
        write_set: WriteSet,
        previous_state_tree: &SparseMerkleTree,
    ) -> Result<(
        HashMap<AccountAddress, AccountStateBlob>,
        Arc<SparseMerkleTree>,
        usize, /* num_account_created */
    )> {
        let mut updated_blobs = HashMap::new();
        let mut num_accounts_created = 0;

        // Find all addresses this transaction touches while processing each write op.
        let mut addrs = HashSet::new();
        for (access_path, write_op) in write_set.into_iter() {
            let address = access_path.address;
            let path = access_path.path;
            match account_to_state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    let account_state = entry.get_mut();
                    // TODO(gzh): we check account creation here for now. Will remove it once we
                    // have a better way.
                    if account_state.is_empty() {
                        num_accounts_created += 1;
                    }
                    Self::update_account_state(account_state, path, write_op);
                }
                hash_map::Entry::Vacant(entry) => {
                    // Before writing to an account, VM should always read that account. So we
                    // should not reach this code path. The exception is genesis transaction (and
                    // maybe other FTVM transactions).
                    match transaction.as_signed_user_txn()?.payload() {
                        TransactionPayload::Program
                        | TransactionPayload::Module(_)
                        | TransactionPayload::Script(_) => {
                            bail!("Write set should be a subset of read set.")
                        }
                        TransactionPayload::WriteSet(_) => (),
                    }

                    let mut account_state = Default::default();
                    Self::update_account_state(&mut account_state, path, write_op);
                    entry.insert(account_state);
                }
            }
            addrs.insert(address);
        }

        for addr in addrs {
            let account_state = account_to_state.get(&addr).expect("Address should exist.");
            let account_blob = AccountStateBlob::try_from(account_state)?;
            updated_blobs.insert(addr, account_blob);
        }
        let state_tree = Arc::new(
            previous_state_tree
                .update(
                    updated_blobs
                        .iter()
                        .map(|(addr, value)| (addr.hash(), value.clone()))
                        .collect(),
                    proof_reader,
                )
                .expect("Failed to update state tree."),
        );

        Ok((updated_blobs, state_tree, num_accounts_created))
    }

    fn update_account_state(account_state: &mut AccountState, path: Vec<u8>, write_op: WriteOp) {
        match write_op {
            WriteOp::Value(new_value) => account_state.insert(path, new_value),
            WriteOp::Deletion => account_state.remove(&path),
        };
    }

    fn extract_reconfig_events(events: Vec<ContractEvent>) -> Vec<ContractEvent> {
        let reconfig_event_key = ValidatorSet::change_event_key();
        events
            .into_iter()
            .filter(|event| *event.key() == reconfig_event_key)
            .collect()
    }
}
