// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{
    collections::{hash_map, HashMap, HashSet},
    convert::TryFrom,
    iter::repeat,
    ops::Deref,
    sync::Arc,
};

use anyhow::{anyhow, bail, ensure, Result};
use once_cell::sync::Lazy;
use rayon::prelude::*;

use diem_crypto::{
    hash::{CryptoHash, EventAccumulatorHasher},
    HashValue,
};
use diem_logger::error;
use diem_state_view::StateViewId;
use diem_types::{
    account_address::{AccountAddress, HashAccountAddress},
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    epoch_state::EpochState,
    event::EventKey,
    nibble::nibble_path::NibblePath,
    on_chain_config,
    proof::{accumulator::InMemoryAccumulator, definition::LeafCount},
    transaction::{
        Transaction, TransactionInfo, TransactionOutput, TransactionPayload, TransactionStatus,
        Version,
    },
    write_set::{WriteOp, WriteSet},
};
use executor_types::{ExecutedChunk, ExecutedTrees, ProofReader, TransactionData};
use scratchpad::{AccountStatus, FrozenSparseMerkleTree, SparseMerkleTree};
use storage_interface::{
    state_view::{InMemoryState, StateCache, VerifiedStateView},
    DbReader, TreeState,
};

use crate::{components::chunk_output::ChunkOutput, metrics::DIEM_EXECUTOR_ERRORS};

pub struct ApplyChunkOutput;

impl ApplyChunkOutput {
    pub fn apply(
        chunk_output: ChunkOutput,
        base_view: &ExecutedTrees,
    ) -> Result<(ExecutedChunk, Vec<Transaction>, Vec<Transaction>)> {
        let ChunkOutput {
            state_cache,
            transactions,
            transaction_outputs,
        } = chunk_output;

        // Separate transactions with different VM statuses.
        let (new_epoch, status, to_keep, to_discard, to_retry) =
            Self::sort_transactions(transactions, transaction_outputs)?;

        // Apply the write set, get the latest state.
        let (
            account_blobs,
            new_node_hashes_vec,
            state_checkpoint_hashes,
            result_state,
            next_epoch_state,
        ) = InMemoryStateCalculator::new(
            base_view.state(),
            state_cache,
            base_view.txn_accumulator().num_leaves(),
        )
        .run_for_transaction_chunk(&to_keep, new_epoch)?;

        // Calculate TransactionData and TransactionInfo, i.e. the ledger history diff.
        let (to_commit, transaction_info_hashes) = Self::assemble_ledger_diff(
            to_keep,
            account_blobs,
            new_node_hashes_vec,
            state_checkpoint_hashes,
        );
        let result_view = ExecutedTrees::new(
            result_state,
            Arc::new(base_view.txn_accumulator().append(&transaction_info_hashes)),
        );

        Ok((
            ExecutedChunk {
                status,
                to_commit,
                result_view,
                next_epoch_state,
                ledger_info: None,
            },
            to_discard,
            to_retry,
        ))
    }

    fn sort_transactions(
        mut transactions: Vec<Transaction>,
        transaction_outputs: Vec<TransactionOutput>,
    ) -> Result<(
        bool,
        Vec<TransactionStatus>,
        Vec<(Transaction, ParsedTransactionOutput)>,
        Vec<Transaction>,
        Vec<Transaction>,
    )> {
        let num_txns = transactions.len();
        let mut transaction_outputs: Vec<ParsedTransactionOutput> =
            transaction_outputs.into_iter().map(Into::into).collect();
        // N.B. off-by-1 intentionally, for exclusive index
        let new_epoch_marker = transaction_outputs
            .iter()
            .position(|o| o.is_reconfig())
            .map(|idx| idx + 1);

        // Transactions after the epoch ending are all to be retried.
        let to_retry = if let Some(pos) = new_epoch_marker {
            transaction_outputs.drain(pos..);
            transactions.drain(pos..).collect()
        } else {
            vec![]
        };

        // N.B. Transaction status after the epoch marker are ignored and set to Retry forcibly.
        let status = transaction_outputs
            .iter()
            .map(|t| t.status())
            .cloned()
            .chain(repeat(TransactionStatus::Retry))
            .take(num_txns)
            .collect();

        // Separate transactions with the Keep status out.
        let (to_keep, to_discard) =
            itertools::zip_eq(transactions.into_iter(), transaction_outputs.into_iter())
                .partition::<Vec<(Transaction, ParsedTransactionOutput)>, _>(|(_, o)| {
                    matches!(o.status(), TransactionStatus::Keep(_))
                });

        // Sanity check transactions with the Discard status:
        let to_discard = to_discard
            .into_iter()
            .map(|(t, o)| {
                // In case a new status other than Retry, Keep and Discard is added:
                if !matches!(o.status(), TransactionStatus::Discard(_)) {
                    error!("Status other than Retry, Keep or Discard; Transaction discarded.");
                }
                // VM shouldn't have output anything for discarded transactions, log if it did.
                if !o.write_set().is_empty() || !o.events().is_empty() {
                    error!(
                        "Discarded transaction has non-empty write set or events. \
                     Transaction: {:?}. Status: {:?}.",
                        t,
                        o.status(),
                    );
                    DIEM_EXECUTOR_ERRORS.inc();
                }
                Ok(t)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((
            new_epoch_marker.is_some(),
            status,
            to_keep,
            to_discard,
            to_retry,
        ))
    }

    fn assemble_ledger_diff(
        to_keep: Vec<(Transaction, ParsedTransactionOutput)>,
        account_blobs: Vec<HashMap<AccountAddress, AccountStateBlob>>,
        new_node_hashes_vec: Vec<HashMap<NibblePath, HashValue>>,
        state_checkpoint_hashes: Vec<Option<HashValue>>,
    ) -> (Vec<(Transaction, TransactionData)>, Vec<HashValue>) {
        let mut to_commit = vec![];
        let mut txn_info_hashes = vec![];
        for (((txn, txn_output), state_checkpoint_hash), (new_node_hashes, blobs)) in
            itertools::zip_eq(
                itertools::zip_eq(to_keep, state_checkpoint_hashes),
                itertools::zip_eq(new_node_hashes_vec, account_blobs),
            )
        {
            let (write_set, events, reconfig_events, gas_used, status) = txn_output.unpack();
            let event_tree = {
                let event_hashes: Vec<_> = events.iter().map(CryptoHash::hash).collect();
                InMemoryAccumulator::<EventAccumulatorHasher>::from_leaves(&event_hashes)
            };

            let state_change_hash = CryptoHash::hash(&write_set);

            let txn_info = match &status {
                TransactionStatus::Keep(status) => TransactionInfo::new(
                    txn.hash(),
                    state_change_hash,
                    event_tree.root_hash(),
                    state_checkpoint_hash,
                    gas_used,
                    status.clone(),
                ),
                _ => unreachable!("Transaction sorted by status already."),
            };

            let txn_info_hash = txn_info.hash();
            txn_info_hashes.push(txn_info_hash);
            to_commit.push((
                txn,
                TransactionData::new(
                    blobs,
                    new_node_hashes,
                    write_set,
                    events,
                    reconfig_events,
                    status,
                    Arc::new(event_tree),
                    gas_used,
                    txn_info,
                    txn_info_hash,
                ),
            ))
        }
        (to_commit, txn_info_hashes)
    }
}

pub fn ensure_no_discard(to_discard: Vec<Transaction>) -> Result<()> {
    ensure!(to_discard.is_empty(), "Syncing discarded transactions");
    Ok(())
}

pub fn ensure_no_retry(to_retry: Vec<Transaction>) -> Result<()> {
    ensure!(to_retry.is_empty(), "Chunk crosses epoch boundary.",);
    Ok(())
}

/// For all accounts modified by this transaction, find the previous blob and update it based
/// on the write set. Returns the blob value of all these accounts.
pub fn process_write_set(
    transaction: Option<&Transaction>,
    account_to_state: &mut HashMap<AccountAddress, AccountState>,
    write_set: WriteSet,
) -> Result<HashSet<AccountAddress>> {
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
                if let Some(transaction) = transaction {
                    // Before writing to an account, VM should always read that account. So we
                    // should not reach this code path. The exception is genesis transaction (and
                    // maybe other writeset transactions).
                    match transaction {
                        Transaction::GenesisTransaction(_) => (),
                        Transaction::BlockMetadata(_) => {
                            bail!("Write set should be a subset of read set.")
                        }
                        Transaction::UserTransaction(txn) => match txn.payload() {
                            TransactionPayload::ModuleBundle(_)
                            | TransactionPayload::Script(_)
                            | TransactionPayload::ScriptFunction(_) => {
                                bail!("Write set should be a subset of read set.")
                            }
                            TransactionPayload::WriteSet(_) => (),
                        },
                        Transaction::StateCheckpoint => {}
                    }
                }

                let mut account_state = Default::default();
                update_account_state(&mut account_state, path, write_op);
                entry.insert(account_state);
            }
        }
        addrs.insert(address);
    }

    Ok(addrs)
}

fn update_account_state(account_state: &mut AccountState, path: Vec<u8>, write_op: WriteOp) {
    match write_op {
        WriteOp::Value(new_value) => account_state.insert(path, new_value),
        WriteOp::Deletion => account_state.remove(&path),
    };
}

pub trait IntoLedgerView {
    fn into_ledger_view(self, db: &Arc<dyn DbReader>) -> Result<ExecutedTrees>;
}

impl IntoLedgerView for TreeState {
    fn into_ledger_view(self, db: &Arc<dyn DbReader>) -> Result<ExecutedTrees> {
        let checkpoint_num_txns = self
            .num_transactions
            .checked_sub(self.write_sets_after_checkpoint.len() as LeafCount)
            .ok_or_else(|| anyhow!("State checkpoint pre-dates genesis."))?;
        let checkpoint =
            InMemoryState::new_at_checkpoint(self.state_checkpoint_hash, checkpoint_num_txns);

        let checkpoint_state_view = VerifiedStateView::new(
            StateViewId::Miscellaneous,
            db.clone(),
            checkpoint_num_txns.checked_sub(1),
            self.state_checkpoint_hash,
            checkpoint.checkpoint.clone(),
        );
        let write_sets: Vec<_> = self.write_sets_after_checkpoint.iter().collect();
        checkpoint_state_view.prime_cache_by_write_set(&write_sets)?;
        let state_cache = checkpoint_state_view.into_state_cache();
        let calculator =
            InMemoryStateCalculator::new(&checkpoint, state_cache, checkpoint_num_txns);
        let state = calculator.run_for_write_sets_after_checkpoint(&write_sets)?;
        let transaction_accumulator = Arc::new(InMemoryAccumulator::new(
            self.ledger_frozen_subtree_hashes,
            self.num_transactions,
        )?);

        Ok(ExecutedTrees::new(state, transaction_accumulator))
    }
}

// FIXME(aldenhu): doc
struct InMemoryStateCalculator {
    _frozen_base: FrozenSparseMerkleTree<AccountStateBlob>,
    accounts: HashMap<AccountAddress, AccountState>,
    proof_reader: ProofReader,

    checkpoint: SparseMerkleTree<AccountStateBlob>,
    checkpoint_num_transactions: LeafCount,
    latest: FrozenSparseMerkleTree<AccountStateBlob>,

    next_version: Version,
    updated_between_checkpoint_and_latest: HashSet<AccountAddress>,
    updated_after_latest: HashSet<AccountAddress>,
}

impl InMemoryStateCalculator {
    pub fn new(base: &InMemoryState, state_cache: StateCache, next_version: Version) -> Self {
        let StateCache {
            _frozen_base,
            accounts,
            proofs,
        } = state_cache;
        let InMemoryState {
            checkpoint,
            checkpoint_num_transactions,
            current,
            updated_since_checkpoint: updated_addresses_since_checkpoint,
        } = base.clone();

        Self {
            _frozen_base,
            accounts,
            proof_reader: ProofReader::new(proofs),
            checkpoint,
            checkpoint_num_transactions,
            latest: current.freeze(),
            next_version,
            updated_between_checkpoint_and_latest: updated_addresses_since_checkpoint,
            updated_after_latest: HashSet::new(),
        }
    }

    pub fn run_for_transaction_chunk(
        mut self,
        to_keep: &[(Transaction, ParsedTransactionOutput)],
        new_epoch: bool,
    ) -> Result<(
        Vec<HashMap<AccountAddress, AccountStateBlob>>,
        Vec<HashMap<NibblePath, HashValue>>,
        Vec<Option<HashValue>>,
        InMemoryState,
        Option<EpochState>,
    )> {
        let mut account_blobs = Vec::new();
        let mut new_node_hashes_vec = Vec::new();
        let mut state_checkpoint_hashes = Vec::new();

        for (txn, txn_output) in to_keep {
            let (b, n, c) = self.add_transaction(txn, txn_output)?;
            account_blobs.push(b);
            new_node_hashes_vec.push(n);
            state_checkpoint_hashes.push(c);
        }
        let (result_state, accounts) = self.finish()?;

        // Get the updated validator set from updated account state.
        let next_epoch_state = if new_epoch {
            Some(Self::parse_validator_set(&accounts)?)
        } else {
            None
        };

        Ok((
            account_blobs,
            new_node_hashes_vec,
            state_checkpoint_hashes,
            result_state,
            next_epoch_state,
        ))
    }

    fn add_transaction(
        &mut self,
        txn: &Transaction,
        txn_output: &ParsedTransactionOutput,
    ) -> Result<(
        HashMap<AccountAddress, AccountStateBlob>,
        HashMap<NibblePath, HashValue>,
        Option<HashValue>,
    )> {
        let updated_accounts = process_write_set(
            Some(txn),
            &mut self.accounts,
            txn_output.write_set().clone(),
        )?;
        self.updated_after_latest
            .extend(updated_accounts.into_iter());
        self.next_version += 1;

        if txn_output.is_reconfig() {
            self.checkpoint()
        } else {
            match txn {
                Transaction::BlockMetadata(_) | Transaction::UserTransaction(_) => {
                    Ok((HashMap::new(), HashMap::new(), None))
                }
                Transaction::GenesisTransaction(_) | Transaction::StateCheckpoint => {
                    self.checkpoint()
                }
            }
        }
    }

    fn checkpoint(
        &mut self,
    ) -> Result<(
        HashMap<AccountAddress, AccountStateBlob>,
        HashMap<NibblePath, HashValue>,
        Option<HashValue>,
    )> {
        // Update SMT.
        let updated_after_current = self.account_blobs_after_current()?;
        let smt_updates: Vec<_> = updated_after_current
            .iter()
            .map(|(addr, blob)| (HashAccountAddress::hash(addr), blob))
            .collect();
        let new_checkpoint = self.latest.batch_update(smt_updates, &self.proof_reader)?;
        let new_node_hashes =
            new_checkpoint.new_node_hashes_since(&self.checkpoint.clone().freeze());
        let root_hash = new_checkpoint.root_hash();

        // Calculate the set of accounts that got changed since last checkpoint.
        let updated_between_checkpoint_and_current: Vec<_> = self
            .updated_between_checkpoint_and_latest
            .difference(&self.updated_after_latest)
            .map(|addr| match self.latest.get(addr.hash()) {
                AccountStatus::ExistsInScratchPad(blob) => Ok((*addr, blob)),
                _ => Err(anyhow!(
                    "Pending state after checkpoint missing. addr: {:x}",
                    addr
                )),
            })
            .collect::<Result<_>>()?;

        let mut account_blobs = updated_after_current;
        account_blobs.extend(updated_between_checkpoint_and_current.into_iter());

        // Move self to the new checkpoint.
        self.latest = new_checkpoint.clone();
        self.checkpoint = new_checkpoint.unfreeze();
        self.checkpoint_num_transactions = self.next_version;
        self.updated_between_checkpoint_and_latest = HashSet::new();
        self.updated_after_latest = HashSet::new();

        Ok((account_blobs, new_node_hashes, Some(root_hash)))
    }

    fn parse_validator_set(accounts: &HashMap<AccountAddress, AccountState>) -> Result<EpochState> {
        let validator_set = accounts
            .get(&on_chain_config::config_address())
            .map(|state| {
                state
                    .get_validator_set()?
                    .ok_or_else(|| anyhow!("ValidatorSet does not exist"))
            })
            .ok_or_else(|| anyhow!("ValidatorSet account does not exist"))??;
        let configuration = accounts
            .get(&on_chain_config::config_address())
            .map(|state| {
                state
                    .get_configuration_resource()?
                    .ok_or_else(|| anyhow!("Configuration does not exist"))
            })
            .ok_or_else(|| anyhow!("Association account does not exist"))??;

        Ok(EpochState {
            epoch: configuration.epoch(),
            verifier: (&validator_set).into(),
        })
    }

    fn account_blobs_after_current(&self) -> Result<HashMap<AccountAddress, AccountStateBlob>> {
        self.updated_after_latest
            .iter()
            .collect::<Vec<_>>()
            .par_iter()
            .with_min_len(100)
            .map(|addr| {
                Ok((
                    **addr,
                    AccountStateBlob::try_from(
                        self.accounts
                            .get(addr)
                            .ok_or_else(|| anyhow!("Address should exist."))?,
                    )?,
                ))
            })
            .collect::<Result<_>>()
    }

    fn finish(self) -> Result<(InMemoryState, HashMap<AccountAddress, AccountState>)> {
        let updates_after_current = self.account_blobs_after_current()?;
        let smt_updates: Vec<_> = updates_after_current
            .iter()
            .map(|(addr, blob)| (HashAccountAddress::hash(addr), blob))
            .collect();
        let latest = self.latest.batch_update(smt_updates, &self.proof_reader)?;

        let mut updated_since_checkpoint = self.updated_between_checkpoint_and_latest;
        updated_since_checkpoint.extend(updates_after_current.keys().cloned());

        let result_state = InMemoryState::new(
            self.checkpoint,
            self.checkpoint_num_transactions,
            latest.unfreeze(),
            updated_since_checkpoint,
        );

        // FIXME(aldenhu): do we need to prime "accounts"?
        Ok((result_state, self.accounts))
    }

    pub fn run_for_write_sets_after_checkpoint(
        mut self,
        write_sets: &[&WriteSet],
    ) -> Result<InMemoryState> {
        for write_set in write_sets {
            let updated_accounts =
                process_write_set(None, &mut self.accounts, (*write_set).clone())?;
            self.updated_after_latest
                .extend(updated_accounts.into_iter());
            self.next_version += 1;
        }
        let (result_state, _) = self.finish()?;
        Ok(result_state)
    }
}

static NEW_EPOCH_EVENT_KEY: Lazy<EventKey> = Lazy::new(on_chain_config::new_epoch_event_key);

struct ParsedTransactionOutput {
    output: TransactionOutput,
    reconfig_events: Vec<ContractEvent>,
}

impl From<TransactionOutput> for ParsedTransactionOutput {
    fn from(output: TransactionOutput) -> Self {
        let reconfig_events = output
            .events()
            .iter()
            .filter(|e| *e.key() == *NEW_EPOCH_EVENT_KEY)
            .cloned()
            .collect();
        Self {
            output,
            reconfig_events,
        }
    }
}

impl Deref for ParsedTransactionOutput {
    type Target = TransactionOutput;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}

impl ParsedTransactionOutput {
    fn is_reconfig(&self) -> bool {
        !self.reconfig_events.is_empty()
    }

    pub fn unpack(
        self,
    ) -> (
        WriteSet,
        Vec<ContractEvent>,
        Vec<ContractEvent>,
        u64,
        TransactionStatus,
    ) {
        let Self {
            output,
            reconfig_events,
        } = self;
        let (write_set, events, gas_used, status) = output.unpack();

        (write_set, events, reconfig_events, gas_used, status)
    }
}
