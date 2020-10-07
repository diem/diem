// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[cfg(test)]
mod executor_test;
#[cfg(any(test, feature = "fuzzing"))]
pub mod fuzzing;
mod logging;
mod metrics;
#[cfg(test)]
mod mock_vm;
mod speculation_cache;
mod types;

pub mod db_bootstrapper;

use crate::{
    logging::{LogEntry, LogSchema},
    metrics::{
        LIBRA_EXECUTOR_COMMIT_BLOCKS_SECONDS, LIBRA_EXECUTOR_ERRORS,
        LIBRA_EXECUTOR_EXECUTE_AND_COMMIT_CHUNK_SECONDS, LIBRA_EXECUTOR_EXECUTE_BLOCK_SECONDS,
        LIBRA_EXECUTOR_SAVE_TRANSACTIONS_SECONDS, LIBRA_EXECUTOR_TRANSACTIONS_SAVED,
        LIBRA_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS,
    },
    speculation_cache::SpeculationCache,
    types::{ProcessedVMOutput, TransactionData},
};
use anyhow::{bail, ensure, format_err, Result};
use executor_types::{
    BlockExecutor, ChunkExecutor, Error, ExecutedTrees, ProofReader, StateComputeResult,
    TransactionReplayer,
};
use fail::fail_point;
use libra_crypto::{
    hash::{CryptoHash, EventAccumulatorHasher, TransactionAccumulatorHasher},
    HashValue,
};
use libra_logger::prelude::*;
use libra_state_view::StateViewId;
use libra_trace::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config,
    proof::{accumulator::InMemoryAccumulator, SparseMerkleProof},
    transaction::{
        Transaction, TransactionInfo, TransactionListWithProof, TransactionOutput,
        TransactionPayload, TransactionStatus, TransactionToCommit, Version,
    },
    write_set::{WriteOp, WriteSet},
};
use libra_vm::VMExecutor;
use scratchpad::SparseMerkleTree;
use std::{
    collections::{hash_map, HashMap, HashSet},
    convert::TryFrom,
    marker::PhantomData,
    sync::Arc,
};
use storage_interface::{state_view::VerifiedStateView, DbReaderWriter, TreeState};

/// `Executor` implements all functionalities the execution module needs to provide.
pub struct Executor<V> {
    db: DbReaderWriter,
    cache: SpeculationCache,
    phantom: PhantomData<V>,
}

impl<V> Executor<V>
where
    V: VMExecutor,
{
    pub fn committed_block_id(&self) -> HashValue {
        self.cache.committed_block_id()
    }

    /// Constructs an `Executor`.
    pub fn new(db: DbReaderWriter) -> Self {
        let startup_info = db
            .reader
            .get_startup_info()
            .expect("Shouldn't fail")
            .expect("DB not bootstrapped.");

        Self {
            db,
            cache: SpeculationCache::new_with_startup_info(startup_info),
            phantom: PhantomData,
        }
    }

    fn reset_cache(&mut self) -> Result<(), Error> {
        let startup_info = self
            .db
            .reader
            .get_startup_info()?
            .ok_or_else(|| format_err!("DB not bootstrapped."))?;
        self.cache = SpeculationCache::new_with_startup_info(startup_info);
        Ok(())
    }

    pub fn new_on_unbootstrapped_db(db: DbReaderWriter, tree_state: TreeState) -> Self {
        Self {
            db,
            cache: SpeculationCache::new_for_db_bootstrapping(tree_state),
            phantom: PhantomData,
        }
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
                epoch_change_li.ledger_info().ends_epoch(),
                "Epoch change LI does not carry validator set"
            );
            ensure!(
                epoch_change_li.ledger_info().next_epoch_state()
                    == new_output.epoch_state().as_ref(),
                "New validator set of a given epoch LI does not match local computation"
            );
            return Ok(Some(epoch_change_li));
        }
        ensure!(
            new_output.epoch_state().is_none(),
            "End of epoch chunk based on local computation but no EoE LedgerInfo provided."
        );
        Ok(None)
    }

    /// Verify input chunk and return transactions to be applied, skipping those already persisted.
    /// Specifically:
    ///  1. Verify that input transactions belongs to the ledger represented by the ledger info.
    ///  2. Verify that transactions to skip match what's already persisted (no fork).
    ///  3. Return Transactions to be applied.
    fn verify_chunk(
        &self,
        txn_list_with_proof: TransactionListWithProof,
        verified_target_li: &LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<TransactionInfo>)> {
        // 1. Verify that input transactions belongs to the ledger represented by the ledger info.
        txn_list_with_proof.verify(
            verified_target_li.ledger_info(),
            txn_list_with_proof.first_transaction_version,
        )?;

        // Return empty if there's no work to do.
        if txn_list_with_proof.transactions.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }
        let first_txn_version = match txn_list_with_proof.first_transaction_version {
            Some(tx) => tx as Version,
            None => {
                bail!(
                    "first_transaction_version doesn't exist in {:?}",
                    txn_list_with_proof
                );
            }
        };

        let num_committed_txns = self.cache.synced_trees().txn_accumulator().num_leaves();
        ensure!(
            first_txn_version <= num_committed_txns,
            "Transaction list too new. Expected version: {}. First transaction version: {}.",
            num_committed_txns,
            first_txn_version
        );
        let versions_between_first_and_committed = num_committed_txns - first_txn_version;
        if txn_list_with_proof.transactions.len() <= versions_between_first_and_committed as usize {
            // All already in DB, nothing to do.
            return Ok((Vec::new(), Vec::new()));
        }

        // 2. Verify that skipped transactions match what's already persisted (no fork):
        let num_txns_to_skip = num_committed_txns - first_txn_version;

        debug!(
            LogSchema::new(LogEntry::ChunkExecutor).num(num_txns_to_skip),
            "skipping_chunk_txns"
        );

        // If the proof is verified, then the length of txn_infos and txns must be the same.
        let skipped_transaction_infos =
            &txn_list_with_proof.proof.transaction_infos()[..num_txns_to_skip as usize];

        // Left side of the proof happens to be the frozen subtree roots of the accumulator
        // right before the list of txns are applied.
        let frozen_subtree_roots_from_proof = txn_list_with_proof
            .proof
            .left_siblings()
            .iter()
            .rev()
            .cloned()
            .collect::<Vec<_>>();
        let accu_from_proof = InMemoryAccumulator::<TransactionAccumulatorHasher>::new(
            frozen_subtree_roots_from_proof,
            first_txn_version,
        )?
        .append(
            &skipped_transaction_infos
                .iter()
                .map(CryptoHash::hash)
                .collect::<Vec<_>>()[..],
        );
        // The two accumulator root hashes should be identical.
        ensure!(
            self.cache.synced_trees().state_id() == accu_from_proof.root_hash(),
            "Fork happens because the current synced_trees doesn't match the txn list provided."
        );

        // 3. Return verified transactions to be applied.
        let mut txns: Vec<_> = txn_list_with_proof.transactions;
        txns.drain(0..num_txns_to_skip as usize);
        let (_, mut txn_infos) = txn_list_with_proof.proof.unpack();
        txn_infos.drain(0..num_txns_to_skip as usize);

        Ok((txns, txn_infos))
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
        let mut next_epoch_state = None;

        let proof_reader = ProofReader::new(account_to_proof);
        let new_epoch_event_key = on_chain_config::new_epoch_event_key();
        for (vm_output, txn) in itertools::zip_eq(vm_outputs.into_iter(), transactions.iter()) {
            if next_epoch_state.is_some() {
                txn_data.push(TransactionData::new(
                    HashMap::new(),
                    vec![],
                    TransactionStatus::Retry,
                    Arc::clone(&current_state_tree),
                    Arc::new(InMemoryAccumulator::<EventAccumulatorHasher>::default()),
                    0,
                    None,
                ));
                continue;
            }
            let (blobs, state_tree) = process_write_set(
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
                        status.clone(),
                    );

                    let real_txn_info_hash = txn_info.hash();
                    txn_info_hashes.push(real_txn_info_hash);
                    txn_info_hash = Some(real_txn_info_hash);
                }
                TransactionStatus::Discard(status) => {
                    if !vm_output.write_set().is_empty() || !vm_output.events().is_empty() {
                        error!(
                            "Discarded transaction has non-empty write set or events. \
                             Transaction: {:?}. Status: {:?}.",
                            txn, status,
                        );
                        LIBRA_EXECUTOR_ERRORS.inc();
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
                txn_info_hash,
            ));
            current_state_tree = state_tree;

            // check for change in validator set
            next_epoch_state = if vm_output
                .events()
                .iter()
                .any(|event| *event.key() == new_epoch_event_key)
            {
                let validator_set = account_to_state
                    .get(&on_chain_config::config_address())
                    .map(|state| {
                        state
                            .get_validator_set()?
                            .ok_or_else(|| format_err!("ValidatorSet does not exist"))
                    })
                    .ok_or_else(|| format_err!("ValidatorSet account does not exist"))??;
                let configuration = account_to_state
                    .get(&on_chain_config::config_address())
                    .map(|state| {
                        state
                            .get_configuration_resource()?
                            .ok_or_else(|| format_err!("Configuration does not exist"))
                    })
                    .ok_or_else(|| format_err!("Association account does not exist"))??;
                Some(EpochState {
                    epoch: configuration.epoch(),
                    verifier: (&validator_set).into(),
                })
            } else {
                None
            }
        }

        let current_transaction_accumulator =
            parent_trees.txn_accumulator().append(&txn_info_hashes);
        Ok(ProcessedVMOutput::new(
            txn_data,
            ExecutedTrees::new_copy(
                current_state_tree,
                Arc::new(current_transaction_accumulator),
            ),
            next_epoch_state,
        ))
    }

    fn extract_reconfig_events(events: Vec<ContractEvent>) -> Vec<ContractEvent> {
        let new_epoch_event_key = on_chain_config::new_epoch_event_key();
        events
            .into_iter()
            .filter(|event| *event.key() == new_epoch_event_key)
            .collect()
    }

    fn get_executed_trees(&self, block_id: HashValue) -> Result<ExecutedTrees, Error> {
        let executed_trees = if block_id == self.cache.committed_block_id() {
            self.cache.committed_trees().clone()
        } else {
            self.cache
                .get_block(&block_id)?
                .lock()
                .output()
                .executed_trees()
                .clone()
        };

        Ok(executed_trees)
    }

    fn get_executed_state_view<'a>(
        &self,
        id: StateViewId,
        executed_trees: &'a ExecutedTrees,
    ) -> VerifiedStateView<'a> {
        VerifiedStateView::new(
            id,
            Arc::clone(&self.db.reader),
            self.cache.committed_trees().version(),
            self.cache.committed_trees().state_root(),
            executed_trees.state_tree(),
        )
    }

    fn replay_transactions_impl(
        &mut self,
        first_version: u64,
        transactions: Vec<Transaction>,
        transaction_infos: Vec<TransactionInfo>,
    ) -> Result<(
        ProcessedVMOutput,
        Vec<TransactionToCommit>,
        Vec<ContractEvent>,
        Vec<Transaction>,
        Vec<TransactionInfo>,
    )> {
        // Construct a StateView and pass the transactions to VM.
        let state_view = VerifiedStateView::new(
            StateViewId::ChunkExecution { first_version },
            Arc::clone(&self.db.reader),
            self.cache.synced_trees().version(),
            self.cache.synced_trees().state_root(),
            self.cache.synced_trees().state_tree(),
        );

        fail_point!("executor::vm_execute_chunk", |_| {
            Err(anyhow::anyhow!("Injected error in execute_chunk"))
        });
        let vm_outputs = V::execute_block(transactions.clone(), &state_view)?;

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
            self.cache.synced_trees(),
        )?;

        // Since we have verified the proofs, we just need to verify that each TransactionInfo
        // object matches what we have computed locally.
        let mut txns_to_commit = vec![];
        let mut reconfig_events = vec![];
        let mut seen_retry = false;
        let mut txns_to_retry = vec![];
        let mut txn_infos_to_retry = vec![];
        for ((txn, txn_data), (i, txn_info)) in itertools::zip_eq(
            itertools::zip_eq(transactions, output.transaction_data()),
            transaction_infos.into_iter().enumerate(),
        ) {
            let recorded_status = match txn_data.status() {
                TransactionStatus::Keep(recorded_status) => recorded_status.clone(),
                status @ TransactionStatus::Discard(_) => bail!(
                    "The transaction at version {}, got the status of 'Discard': {:?}",
                    first_version + i as u64,
                    status
                ),
                TransactionStatus::Retry => {
                    seen_retry = true;
                    txns_to_retry.push(txn);
                    txn_infos_to_retry.push(txn_info);
                    continue;
                }
            };
            assert!(!seen_retry);
            let generated_txn_info = TransactionInfo::new(
                txn.hash(),
                txn_data.state_root_hash(),
                txn_data.event_root_hash(),
                txn_data.gas_used(),
                recorded_status.clone(),
            );
            ensure!(
                txn_info == generated_txn_info,
                "txn_info do not match for {}-th transaction in chunk.\nChunk txn_info: {}\nProof txn_info: {}",
                i, generated_txn_info, txn_info
            );
            txns_to_commit.push(TransactionToCommit::new(
                txn,
                txn_data.account_blobs().clone(),
                txn_data.events().to_vec(),
                txn_data.gas_used(),
                recorded_status,
            ));
            reconfig_events.append(&mut Self::extract_reconfig_events(
                txn_data.events().to_vec(),
            ));
        }

        Ok((
            output,
            txns_to_commit,
            reconfig_events,
            txns_to_retry,
            txn_infos_to_retry,
        ))
    }

    fn execute_chunk(
        &mut self,
        first_version: u64,
        transactions: Vec<Transaction>,
        transaction_infos: Vec<TransactionInfo>,
    ) -> Result<(
        ProcessedVMOutput,
        Vec<TransactionToCommit>,
        Vec<ContractEvent>,
    )> {
        let num_txns = transactions.len();

        let (processed_vm_output, txns_to_commit, events, txns_to_retry, _txn_infos_to_retry) =
            self.replay_transactions_impl(first_version, transactions, transaction_infos)?;

        ensure!(
            txns_to_retry.is_empty(),
            "The transaction at version {} got the status of 'Retry'",
            num_txns - txns_to_retry.len() + first_version as usize,
        );

        Ok((processed_vm_output, txns_to_commit, events))
    }
}

impl<V: VMExecutor> ChunkExecutor for Executor<V> {
    fn execute_and_commit_chunk(
        &mut self,
        txn_list_with_proof: TransactionListWithProof,
        // Target LI that has been verified independently: the proofs are relative to this version.
        verified_target_li: LedgerInfoWithSignatures,
        // An optional end of epoch LedgerInfo. We do not allow chunks that end epoch without
        // carrying any epoch change LI.
        epoch_change_li: Option<LedgerInfoWithSignatures>,
    ) -> Result<Vec<ContractEvent>> {
        let _timer = LIBRA_EXECUTOR_EXECUTE_AND_COMMIT_CHUNK_SECONDS.start_timer();
        // 1. Update the cache in executor to be consistent with latest synced state.
        self.reset_cache()?;

        info!(
            LogSchema::new(LogEntry::ChunkExecutor)
                .local_synced_version(self.cache.synced_trees().txn_accumulator().num_leaves() - 1)
                .first_version_in_request(txn_list_with_proof.first_transaction_version)
                .num_txns_in_request(txn_list_with_proof.transactions.len()),
            "sync_request_received",
        );

        // 2. Verify input transaction list.
        let (transactions, transaction_infos) =
            self.verify_chunk(txn_list_with_proof, &verified_target_li)?;

        // 3. Execute transactions.
        let first_version = self.cache.synced_trees().txn_accumulator().num_leaves();
        let (output, txns_to_commit, reconfig_events) =
            self.execute_chunk(first_version, transactions, transaction_infos)?;

        // 4. Commit to DB.
        let ledger_info_to_commit =
            Self::find_chunk_li(verified_target_li, epoch_change_li, &output)?;
        if ledger_info_to_commit.is_none() && txns_to_commit.is_empty() {
            return Ok(reconfig_events);
        }
        fail_point!("executor::commit_chunk", |_| {
            Err(anyhow::anyhow!("Injected error in commit_chunk"))
        });
        self.db.writer.save_transactions(
            &txns_to_commit,
            first_version,
            ledger_info_to_commit.as_ref(),
        )?;

        // 5. Cache maintenance.
        let output_trees = output.executed_trees().clone();
        if let Some(ledger_info_with_sigs) = &ledger_info_to_commit {
            self.cache.update_block_tree_root(
                output_trees,
                ledger_info_with_sigs.ledger_info(),
                vec![],
                vec![],
            );
        } else {
            self.cache.update_synced_trees(output_trees);
        }
        self.cache.reset();

        info!(
            LogSchema::new(LogEntry::ChunkExecutor)
                .synced_to_version(
                    self.cache
                        .synced_trees()
                        .version()
                        .expect("version must exist")
                )
                .committed_with_ledger_info(ledger_info_to_commit.is_some()),
            "sync_finished",
        );

        Ok(reconfig_events)
    }
}

impl<V: VMExecutor> TransactionReplayer for Executor<V> {
    fn replay_chunk(
        &mut self,
        mut first_version: Version,
        mut txns: Vec<Transaction>,
        mut txn_infos: Vec<TransactionInfo>,
    ) -> Result<()> {
        ensure!(
            first_version == self.cache.synced_trees().txn_accumulator().num_leaves(),
            "Version not expected. Expected: {}, got: {}",
            self.cache.synced_trees().txn_accumulator().num_leaves(),
            first_version,
        );
        while !txns.is_empty() {
            let num_txns = txns.len();

            let (output, txns_to_commit, _, txns_to_retry, txn_infos_to_retry) =
                self.replay_transactions_impl(first_version, txns, txn_infos)?;
            assert!(txns_to_retry.len() < num_txns);

            self.db
                .writer
                .save_transactions(&txns_to_commit, first_version, None)?;

            self.cache
                .update_synced_trees(output.executed_trees().clone());

            txns = txns_to_retry;
            txn_infos = txn_infos_to_retry;
            first_version += txns_to_commit.len() as u64;
        }
        Ok(())
    }

    fn expecting_version(&self) -> Version {
        self.cache.synced_trees().version().map_or(0, |v| v + 1)
    }
}

impl<V: VMExecutor> BlockExecutor for Executor<V> {
    fn committed_block_id(&mut self) -> Result<HashValue, Error> {
        Ok(Self::committed_block_id(self))
    }

    fn reset(&mut self) -> Result<(), Error> {
        self.reset_cache()
    }

    fn execute_block(
        &mut self,
        block: (HashValue, Vec<Transaction>),
        parent_block_id: HashValue,
    ) -> Result<StateComputeResult, Error> {
        let (block_id, mut transactions) = block;

        // Reconfiguration rule - if a block is a child of pending reconfiguration, it needs to be empty
        // So we roll over the executed state until it's committed and we start new epoch.
        let (output, state_compute_result) = if parent_block_id != self.committed_block_id()?
            && self
                .cache
                .get_block(&parent_block_id)?
                .lock()
                .output()
                .has_reconfiguration()
        {
            let parent = self.cache.get_block(&parent_block_id)?;
            let parent_block = parent.lock();
            let parent_output = parent_block.output();

            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "reconfig_descendant_block_received"
            );

            let output = ProcessedVMOutput::new(
                vec![],
                parent_output.executed_trees().clone(),
                parent_output.epoch_state().clone(),
            );

            let parent_accu = parent_output.executed_trees().txn_accumulator();
            let state_compute_result = output.compute_result(
                parent_accu.frozen_subtree_roots().clone(),
                parent_accu.num_leaves(),
            );

            // Reset the reconfiguration suffix transactions to empty list.
            transactions = vec![];

            (output, state_compute_result)
        } else {
            info!(
                LogSchema::new(LogEntry::BlockExecutor).block_id(block_id),
                "execute_block"
            );

            let _timer = LIBRA_EXECUTOR_EXECUTE_BLOCK_SECONDS.start_timer();

            let parent_block_executed_trees = self.get_executed_trees(parent_block_id)?;

            let state_view = self.get_executed_state_view(
                StateViewId::BlockExecution { block_id },
                &parent_block_executed_trees,
            );

            let vm_outputs = {
                trace_code_block!("executor::execute_block", {"block", block_id});
                let _timer = LIBRA_EXECUTOR_VM_EXECUTE_BLOCK_SECONDS.start_timer();
                fail_point!("executor::vm_execute_block", |_| {
                    Err(Error::from(anyhow::anyhow!(
                        "Injected error in vm_execute_block"
                    )))
                });
                V::execute_block(transactions.clone(), &state_view).map_err(anyhow::Error::from)?
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
                &parent_block_executed_trees,
            )
            .map_err(|err| format_err!("Failed to execute block: {}", err))?;

            let parent_accu = parent_block_executed_trees.txn_accumulator();

            let state_compute_result = output.compute_result(
                parent_accu.frozen_subtree_roots().clone(),
                parent_accu.num_leaves(),
            );
            (output, state_compute_result)
        };

        // Add the output to the speculation_output_tree
        self.cache
            .add_block(parent_block_id, (block_id, transactions, output))?;

        Ok(state_compute_result)
    }

    fn commit_blocks(
        &mut self,
        block_ids: Vec<HashValue>,
        ledger_info_with_sigs: LedgerInfoWithSignatures,
    ) -> Result<(Vec<Transaction>, Vec<ContractEvent>), Error> {
        let _timer = LIBRA_EXECUTOR_COMMIT_BLOCKS_SECONDS.start_timer();
        let block_id_to_commit = ledger_info_with_sigs.ledger_info().consensus_block_id();

        info!(
            LogSchema::new(LogEntry::BlockExecutor).block_id(block_id_to_commit),
            "commit_block"
        );

        let version = ledger_info_with_sigs.ledger_info().version();

        let num_txns_in_li = version
            .checked_add(1)
            .ok_or_else(|| format_err!("version + 1 overflows"))?;
        let num_persistent_txns = self.cache.synced_trees().txn_accumulator().num_leaves();

        if num_txns_in_li < num_persistent_txns {
            return Err(Error::InternalError {
                error: format!(
                    "Try to commit stale transactions with the last version as {}",
                    version
                ),
            });
        }

        if num_txns_in_li == num_persistent_txns {
            return Ok(self.cache.committed_txns_and_events());
        }

        // All transactions that need to go to storage. In the above example, this means all the
        // transactions in A, B and C whose status == TransactionStatus::Keep.
        // This must be done before calculate potential skipping of transactions in idempotent commit.
        let mut txns_to_keep = vec![];
        let arc_blocks = block_ids
            .iter()
            .map(|id| self.cache.get_block(id))
            .collect::<Result<Vec<_>, Error>>()?;
        let blocks = arc_blocks.iter().map(|b| b.lock()).collect::<Vec<_>>();
        for (txn, txn_data) in blocks.iter().flat_map(|block| {
            itertools::zip_eq(block.transactions(), block.output().transaction_data())
        }) {
            if let TransactionStatus::Keep(recorded_status) = txn_data.status() {
                txns_to_keep.push(TransactionToCommit::new(
                    txn.clone(),
                    txn_data.account_blobs().clone(),
                    txn_data.events().to_vec(),
                    txn_data.gas_used(),
                    recorded_status.clone(),
                ));
            }
        }

        let last_block = blocks
            .last()
            .ok_or_else(|| format_err!("CommittableBlockBatch is empty"))?;

        // Check that the version in ledger info (computed by consensus) matches the version
        // computed by us.
        let num_txns_in_speculative_accumulator = last_block
            .output()
            .executed_trees()
            .txn_accumulator()
            .num_leaves();
        assert_eq!(
            num_txns_in_li, num_txns_in_speculative_accumulator as Version,
            "Number of transactions in ledger info ({}) does not match number of transactions \
             in accumulator ({}).",
            num_txns_in_li, num_txns_in_speculative_accumulator,
        );

        let num_txns_to_keep = txns_to_keep.len() as u64;

        // Skip txns that are already committed to allow failures in state sync process.
        let first_version_to_keep = num_txns_in_li - num_txns_to_keep;
        assert!(
            first_version_to_keep <= num_persistent_txns,
            "first_version {} in the blocks to commit cannot exceed # of committed txns: {}.",
            first_version_to_keep,
            num_persistent_txns
        );

        let num_txns_to_skip = num_persistent_txns - first_version_to_keep;
        let first_version_to_commit = first_version_to_keep + num_txns_to_skip;

        if num_txns_to_skip != 0 {
            debug!(
                LogSchema::new(LogEntry::BlockExecutor)
                    .latest_synced_version(num_persistent_txns - 1)
                    .first_version_to_keep(first_version_to_keep)
                    .num_txns_to_keep(num_txns_to_keep)
                    .first_version_to_commit(first_version_to_commit),
                "skip_transactions_when_committing"
            );
        }

        // Skip duplicate txns that are already persistent.
        let txns_to_commit = &txns_to_keep[num_txns_to_skip as usize..];

        let num_txns_to_commit = txns_to_commit.len() as u64;
        {
            let _timer = LIBRA_EXECUTOR_SAVE_TRANSACTIONS_SECONDS.start_timer();
            LIBRA_EXECUTOR_TRANSACTIONS_SAVED.observe(num_txns_to_commit as f64);

            assert_eq!(first_version_to_commit, num_txns_in_li - num_txns_to_commit);
            fail_point!("executor::commit_blocks", |_| {
                Err(Error::from(anyhow::anyhow!(
                    "Injected error in commit_blocks"
                )))
            });
            self.db.writer.save_transactions(
                txns_to_commit,
                first_version_to_commit,
                Some(&ledger_info_with_sigs),
            )?;
        }

        // Prune the tree.
        for block in blocks {
            for txn_data in block.output().transaction_data() {
                txn_data.prune_state_tree();
            }
        }
        // Calculate committed transactions and reconfig events now that commit has succeeded
        let mut committed_txns = vec![];
        let mut reconfig_events = vec![];
        for txn in txns_to_commit.iter() {
            committed_txns.push(txn.transaction().clone());
            reconfig_events.append(&mut Self::extract_reconfig_events(txn.events().to_vec()));
        }

        self.cache.prune(
            ledger_info_with_sigs.ledger_info(),
            committed_txns.clone(),
            reconfig_events.clone(),
        )?;

        // Now that the blocks are persisted successfully, we can reply to consensus
        Ok((committed_txns, reconfig_events))
    }
}

/// For all accounts modified by this transaction, find the previous blob and update it based
/// on the write set. Returns the blob value of all these accounts as well as the newly
/// constructed state tree.
pub fn process_write_set(
    transaction: &Transaction,
    account_to_state: &mut HashMap<AccountAddress, AccountState>,
    proof_reader: &ProofReader,
    write_set: WriteSet,
    previous_state_tree: &SparseMerkleTree,
) -> Result<(
    HashMap<AccountAddress, AccountStateBlob>,
    Arc<SparseMerkleTree>,
)> {
    let mut updated_blobs = HashMap::new();

    // Find all addresses this transaction touches while processing each write op.
    let mut addrs = HashSet::new();
    for (access_path, write_op) in write_set.into_iter() {
        let address = access_path.address;
        let path = access_path.path;
        match account_to_state.entry(address) {
            hash_map::Entry::Occupied(mut entry) => {
                update_account_state(entry.get_mut(), path, write_op);
            }
            hash_map::Entry::Vacant(entry) => {
                // Before writing to an account, VM should always read that account. So we
                // should not reach this code path. The exception is genesis transaction (and
                // maybe other writeset transactions).
                match transaction {
                    Transaction::GenesisTransaction(_) => (),
                    Transaction::BlockMetadata(_) => {
                        bail!("Write set should be a subset of read set.")
                    }
                    Transaction::UserTransaction(txn) => match txn.payload() {
                        TransactionPayload::Module(_) | TransactionPayload::Script(_) => {
                            bail!("Write set should be a subset of read set.")
                        }
                        TransactionPayload::WriteSet(_) => (),
                    },
                }

                let mut account_state = Default::default();
                update_account_state(&mut account_state, path, write_op);
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

    Ok((updated_blobs, state_tree))
}

fn update_account_state(account_state: &mut AccountState, path: Vec<u8>, write_op: WriteOp) {
    match write_op {
        WriteOp::Value(new_value) => account_state.insert(path, new_value),
        WriteOp::Deletion => account_state.remove(&path),
    };
}
