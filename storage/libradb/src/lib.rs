// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate provides [`LibraDB`] which represents physical storage of the core Libra data
//! structures.
//!
//! It relays read/write operations on the physical storage via [`schemadb`] to the underlying
//! Key-Value storage system, and implements libra data structures on top of it.

#[macro_use]
extern crate prometheus;
// Used in this and other crates for testing.
#[cfg(any(test, feature = "fuzzing"))]
pub mod test_helper;

pub mod backup;
pub mod errors;
pub mod schema;

mod change_set;
mod event_store;
mod ledger_counters;
mod ledger_store;
mod pruner;
mod state_store;
mod system_store;
mod transaction_store;

#[cfg(test)]
mod libradb_test;

use crate::{
    backup::BackupHandler,
    change_set::{ChangeSet, SealedChangeSet},
    errors::LibraDbError,
    event_store::EventStore,
    ledger_counters::LedgerCounters,
    ledger_store::LedgerStore,
    pruner::Pruner,
    schema::*,
    state_store::StateStore,
    system_store::SystemStore,
    transaction_store::TransactionStore,
};
use anyhow::{bail, ensure, format_err, Result};
use itertools::{izip, zip_eq};
use jellyfish_merkle::restore::JellyfishMerkleRestore;
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_logger::prelude::*;
use libra_metrics::OpMetrics;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::{ContractEvent, EventWithProof},
    crypto_proxies::ValidatorChangeProof,
    event::EventKey,
    get_with_proof::{RequestItem, ResponseItem},
    ledger_info::LedgerInfoWithSignatures,
    proof::{
        AccountStateProof, AccumulatorConsistencyProof, EventProof, SparseMerkleProof,
        SparseMerkleRangeProof, TransactionListProof, TransactionProof,
    },
    transaction::{
        TransactionInfo, TransactionListWithProof, TransactionToCommit, TransactionWithProof,
        Version,
    },
};
use once_cell::sync::Lazy;
use prometheus::{IntCounter, IntGauge, IntGaugeVec};
use schemadb::{ColumnFamilyOptions, ColumnFamilyOptionsMap, DB, DEFAULT_CF_NAME};
use std::{iter::Iterator, path::Path, sync::Arc, time::Instant};
use storage_proto::{StartupInfo, TreeState};

static OP_COUNTER: Lazy<OpMetrics> = Lazy::new(|| OpMetrics::new_and_registered("storage"));

pub static LIBRA_STORAGE_CF_SIZE_BYTES: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        // metric name
        "libra_storage_cf_size_bytes",
        // metric description
        "Libra storage Column Family size in bytes",
        // metric labels (dimensions)
        &["cf_name"]
    )
    .unwrap()
});

pub static LIBRA_STORAGE_COMMITTED_TXNS: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "libra_storage_committed_txns",
        "Libra storage committed transactions"
    )
    .unwrap()
});

pub static LIBRA_STORAGE_LATEST_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "libra_storage_latest_transaction_version",
        "Libra storage latest transaction version"
    )
    .unwrap()
});

const MAX_LIMIT: u64 = 1000;
const MAX_REQUEST_ITEMS: u64 = 100;

// TODO: Either implement an iteration API to allow a very old client to loop through a long history
// or guarantee that there is always a recent enough waypoint and client knows to boot from there.
const MAX_NUM_EPOCH_CHANGE_LEDGER_INFO: usize = 100;

fn error_if_too_many_requested(num_requested: u64, max_allowed: u64) -> Result<()> {
    if num_requested > max_allowed {
        Err(LibraDbError::TooManyRequested(num_requested, max_allowed).into())
    } else {
        Ok(())
    }
}

/// Trait that is implemented by a DB that supports certain public (to client) read APIs
/// expected of a Libra DB
pub trait LibraDBTrait: Send + Sync {
    /// Given an account address, returns the latest account state. `None` if the account does not
    /// exist.
    fn get_latest_account_state(&self, address: AccountAddress)
        -> Result<Option<AccountStateBlob>>;

    /// Returns the latest version and committed block timestamp
    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)>;

    fn get_txn_by_account(
        &self,
        address: AccountAddress,
        seq_num: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>>;

    fn get_latest_version(&self) -> Result<Version>;

    fn get_transactions(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// Returns events by given event key
    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>>;

    /// Returns proof of new state relative to version known to client
    fn get_state_proof(
        &self,
        known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )>;

    /// Returns the account state corresponding to the given version and account address with proof
    /// based on `ledger_version`
    fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        version: Version,
        ledger_version: Version,
    ) -> Result<AccountStateWithProof>;
}

/// This holds a handle to the underlying DB responsible for physical storage and provides APIs for
/// access to the core Libra data structures.
pub struct LibraDB {
    db: Arc<DB>,
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
    event_store: EventStore,
    system_store: SystemStore,
    pruner: Pruner,
}

impl LibraDB {
    /// Config parameter for the pruner.
    const NUM_HISTORICAL_VERSIONS_TO_KEEP: u64 = 1_000_000;

    pub fn open<P: AsRef<Path> + Clone>(
        db_root_path: P,
        readonly: bool,
        log_dir: Option<P>,
    ) -> Result<Self> {
        let cf_opts_map: ColumnFamilyOptionsMap = [
            (
                /* LedgerInfo CF = */ DEFAULT_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (EPOCH_BY_VERSION_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_ACCUMULATOR_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_BY_KEY_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_CF_NAME, ColumnFamilyOptions::default()),
            (
                JELLYFISH_MERKLE_NODE_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (LEDGER_COUNTERS_CF_NAME, ColumnFamilyOptions::default()),
            (STALE_NODE_INDEX_CF_NAME, ColumnFamilyOptions::default()),
            (TRANSACTION_CF_NAME, ColumnFamilyOptions::default()),
            (
                TRANSACTION_ACCUMULATOR_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (
                TRANSACTION_BY_ACCOUNT_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (TRANSACTION_INFO_CF_NAME, ColumnFamilyOptions::default()),
        ]
        .iter()
        .cloned()
        .collect();

        let path = db_root_path.as_ref().join("libradb");
        let instant = Instant::now();

        let db = Arc::new(if readonly {
            let db_log_dir = log_dir
                .ok_or_else(|| format_err!("Must provide log_dir if opening in readonly mode."))?;
            if !db_log_dir.as_ref().is_dir() {
                bail!("Invalid log directory: {:?}", db_log_dir.as_ref());
            }
            info!("log stored at {:?}", db_log_dir.as_ref());
            DB::open_readonly(path.clone(), cf_opts_map, db_log_dir.as_ref().to_path_buf())?
        } else {
            DB::open(path.clone(), cf_opts_map)?
        });

        info!(
            "Opened LibraDB at {:?} in {} ms",
            path,
            instant.elapsed().as_millis()
        );

        Ok(LibraDB {
            db: Arc::clone(&db),
            event_store: EventStore::new(Arc::clone(&db)),
            ledger_store: Arc::new(LedgerStore::new(Arc::clone(&db))),
            state_store: Arc::new(StateStore::new(Arc::clone(&db))),
            transaction_store: Arc::new(TransactionStore::new(Arc::clone(&db))),
            system_store: SystemStore::new(Arc::clone(&db)),
            pruner: Pruner::new(Arc::clone(&db), Self::NUM_HISTORICAL_VERSIONS_TO_KEEP),
        })
    }

    /// This creates an empty LibraDB instance on disk or opens one if it already exists.
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Self {
        Self::open(db_root_path, false, None).expect("Unable to open LibraDB")
    }

    // ================================== Public API ==================================
    /// Returns events specified by `query_path` with sequence number in range designated by
    /// `start_seq_num`, `ascending` and `limit`. If ascending is true this query will return up to
    /// `limit` events that were emitted after `start_event_seq_num`. Otherwise, it will return up
    /// to `limit` events in the reverse order. Both cases are inclusive.
    fn get_events_by_query_path(
        &self,
        query_path: &AccessPath,
        start_seq_num: u64,
        ascending: bool,
        limit: u64,
        ledger_version: Version,
    ) -> Result<(Vec<EventWithProof>, AccountStateWithProof)> {
        let account_state_with_proof =
            self.get_account_state_with_proof(query_path.address, ledger_version, ledger_version)?;

        let event_key = {
            let (event_key_opt, _count) =
                account_state_with_proof.get_event_key_and_count_by_query_path(&query_path.path)?;
            if let Some(event_key) = event_key_opt {
                event_key
            } else {
                return Ok((Vec::new(), account_state_with_proof));
            }
        };

        let events_with_proof = self.get_events_by_event_key(
            &event_key,
            start_seq_num,
            ascending,
            limit,
            ledger_version,
        )?;

        // We always need to return the account blob to prove that this is indeed the event that was
        // being queried.
        Ok((events_with_proof, account_state_with_proof))
    }

    fn get_events_by_event_key(
        &self,
        event_key: &EventKey,
        start_seq_num: u64,
        ascending: bool,
        limit: u64,
        ledger_version: Version,
    ) -> Result<Vec<EventWithProof>> {
        error_if_too_many_requested(limit, MAX_LIMIT)?;
        let get_latest = !ascending && start_seq_num == u64::max_value();

        let cursor = if get_latest {
            // Caller wants the latest, figure out the latest seq_num.
            // In the case of no events on that path, use 0 and expect empty result below.
            self.event_store
                .get_latest_sequence_number(ledger_version, &event_key)?
                .unwrap_or(0)
        } else {
            start_seq_num
        };

        // Convert requested range and order to a range in ascending order.
        let (first_seq, real_limit) = get_first_seq_num_and_limit(ascending, cursor, limit)?;

        // Query the index.
        let mut event_keys = self.event_store.lookup_events_by_key(
            &event_key,
            first_seq,
            real_limit,
            ledger_version,
        )?;

        // When descending, it's possible that user is asking for something beyond the latest
        // sequence number, in which case we will consider it a bad request and return an empty
        // list.
        // For example, if the latest sequence number is 100, and the caller is asking for 110 to
        // 90, we will get 90 to 100 from the index lookup above. Seeing that the last item
        // is 100 instead of 110 tells us 110 is out of bound.
        if !ascending {
            if let Some((seq_num, _, _)) = event_keys.last() {
                if *seq_num < cursor {
                    event_keys = Vec::new();
                }
            }
        }

        let mut events_with_proof = event_keys
            .into_iter()
            .map(|(seq, ver, idx)| {
                let (event, event_proof) = self
                    .event_store
                    .get_event_with_proof_by_version_and_index(ver, idx)?;
                ensure!(
                    seq == event.sequence_number(),
                    "Index broken, expected seq:{}, actual:{}",
                    seq,
                    event.sequence_number()
                );
                let (txn_info, txn_info_proof) = self
                    .ledger_store
                    .get_transaction_info_with_proof(ver, ledger_version)?;
                let proof = EventProof::new(txn_info_proof, txn_info, event_proof);
                Ok(EventWithProof::new(ver, idx, event, proof))
            })
            .collect::<Result<Vec<_>>>()?;
        if !ascending {
            events_with_proof.reverse();
        }

        Ok(events_with_proof)
    }

    /// Returns ledger infos reflecting epoch bumps starting with the given epoch. If there are no
    /// more than `MAX_NUM_EPOCH_CHANGE_LEDGER_INFO` results, this function returns all of them,
    /// otherwise the first `MAX_NUM_EPOCH_CHANGE_LEDGER_INFO` results are returned and a flag
    /// (when true) will be used to indicate the fact that there is more.
    pub fn get_epoch_change_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<(Vec<LedgerInfoWithSignatures>, bool)> {
        self.ledger_store.get_first_n_epoch_change_ledger_infos(
            start_epoch,
            end_epoch,
            MAX_NUM_EPOCH_CHANGE_LEDGER_INFO,
        )
    }

    /// Persist transactions. Called by the executor module when either syncing nodes or committing
    /// blocks during normal operation.
    ///
    /// `first_version` is the version of the first transaction in `txns_to_commit`.
    /// When `ledger_info_with_sigs` is provided, verify that the transaction accumulator root hash
    /// it carries is generated after the `txns_to_commit` are applied.
    /// Note that even if `txns_to_commit` is empty, `frist_version` is checked to be
    /// `ledger_info_with_sigs.ledger_info.version + 1` if `ledger_info_with_sigs` is not `None`.
    pub fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: Version,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let num_txns = txns_to_commit.len() as u64;
        // ledger_info_with_sigs could be None if we are doing state synchronization. In this case
        // txns_to_commit should not be empty. Otherwise it is okay to commit empty blocks.
        ensure!(
            ledger_info_with_sigs.is_some() || num_txns > 0,
            "txns_to_commit is empty while ledger_info_with_sigs is None.",
        );

        if let Some(x) = ledger_info_with_sigs {
            let claimed_last_version = x.ledger_info().version();
            ensure!(
                claimed_last_version + 1 == first_version + num_txns,
                "Transaction batch not applicable: first_version {}, num_txns {}, last_version {}",
                first_version,
                num_txns,
                claimed_last_version,
            );
        }

        // Gather db mutations to `batch`.
        let mut cs = ChangeSet::new();

        let new_root_hash = self.save_transactions_impl(txns_to_commit, first_version, &mut cs)?;

        // If expected ledger info is provided, verify result root hash and save the ledger info.
        if let Some(x) = ledger_info_with_sigs {
            let expected_root_hash = x.ledger_info().transaction_accumulator_hash();
            ensure!(
                new_root_hash == expected_root_hash,
                "Root hash calculated doesn't match expected. {:?} vs {:?}",
                new_root_hash,
                expected_root_hash,
            );

            self.ledger_store.put_ledger_info(x, &mut cs)?;
        }

        // Persist.
        let (sealed_cs, counters) = self.seal_change_set(first_version, num_txns, cs)?;
        self.commit(sealed_cs)?;
        // Once everything is successfully persisted, update the latest in-memory ledger info.
        if let Some(x) = ledger_info_with_sigs {
            self.ledger_store.set_latest_ledger_info(x.clone());
        }

        // Only increment counter if commit succeeds and there are at least one transaction written
        // to the storage. That's also when we'd inform the pruner thread to work.
        if num_txns > 0 {
            let last_version = first_version + num_txns - 1;
            OP_COUNTER.inc_by("committed_txns", num_txns as usize);
            LIBRA_STORAGE_COMMITTED_TXNS.inc_by(num_txns as i64);
            OP_COUNTER.set("latest_transaction_version", last_version as usize);
            LIBRA_STORAGE_LATEST_TXN_VERSION.set(last_version as i64);
            counters
                .expect("Counters should be bumped with transactions being saved.")
                .bump_op_counters();

            self.pruner.wake(last_version);
        }

        Ok(())
    }

    fn save_transactions_impl(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: u64,
        mut cs: &mut ChangeSet,
    ) -> Result<HashValue> {
        let last_version = first_version + txns_to_commit.len() as u64 - 1;

        // Account state updates. Gather account state root hashes
        let account_state_sets = txns_to_commit
            .iter()
            .map(|txn_to_commit| txn_to_commit.account_states().clone())
            .collect::<Vec<_>>();
        let state_root_hashes =
            self.state_store
                .put_account_state_sets(account_state_sets, first_version, &mut cs)?;

        // Event updates. Gather event accumulator root hashes.
        let event_root_hashes = zip_eq(first_version..=last_version, txns_to_commit)
            .map(|(ver, txn_to_commit)| {
                self.event_store
                    .put_events(ver, txn_to_commit.events(), &mut cs)
            })
            .collect::<Result<Vec<_>>>()?;

        // Transaction updates. Gather transaction hashes.
        zip_eq(first_version..=last_version, txns_to_commit)
            .map(|(ver, txn_to_commit)| {
                self.transaction_store
                    .put_transaction(ver, txn_to_commit.transaction(), &mut cs)
            })
            .collect::<Result<()>>()?;

        // Transaction accumulator updates. Get result root hash.
        let txn_infos = izip!(txns_to_commit, state_root_hashes, event_root_hashes)
            .map(|(t, s, e)| {
                Ok(TransactionInfo::new(
                    t.transaction().hash(),
                    s,
                    e,
                    t.gas_used(),
                    t.major_status(),
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(txn_infos.len(), txns_to_commit.len());

        let new_root_hash =
            self.ledger_store
                .put_transaction_infos(first_version, &txn_infos, &mut cs)?;

        Ok(new_root_hash)
    }

    /// This backs the `UpdateToLatestLedger` public read API which returns the latest
    /// [`LedgerInfoWithSignatures`] together with items requested and proofs relative to the same
    /// ledger info.
    pub fn update_to_latest_ledger(
        &self,
        client_known_version: Version,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        error_if_too_many_requested(request_items.len() as u64, MAX_REQUEST_ITEMS)?;

        // Get the latest ledger info and signatures
        let ledger_info_with_sigs = self.ledger_store.get_latest_ledger_info()?;
        let ledger_info = ledger_info_with_sigs.ledger_info();
        let ledger_version = ledger_info.version();

        // TODO: cache last epoch change version to avoid a DB access in most cases.
        let client_epoch = self.ledger_store.get_epoch(client_known_version)?;
        let validator_change_proof = if client_epoch < ledger_info.epoch() {
            let (ledger_infos_with_sigs, more) = self.get_epoch_change_ledger_infos(
                client_epoch,
                self.ledger_store.get_epoch(ledger_info.version())?,
            )?;
            ValidatorChangeProof::new(ledger_infos_with_sigs, more)
        } else {
            ValidatorChangeProof::new(vec![], /* more = */ false)
        };

        let client_new_version = if !validator_change_proof.more {
            ledger_version
        } else {
            validator_change_proof
                .ledger_info_with_sigs
                .last()
                .expect("Must have at least one LedgerInfo.")
                .ledger_info()
                .version()
        };
        let ledger_consistency_proof = self
            .ledger_store
            .get_consistency_proof(client_known_version, client_new_version)?;

        // If the validator change proof in the response is enough for the client to update to
        // latest LedgerInfo, fulfill all request items. Otherwise the client will not be able to
        // verify the latest LedgerInfo, so do not send response items back.
        let response_items = if !validator_change_proof.more {
            self.get_response_items(request_items, ledger_version)?
        } else {
            vec![]
        };

        Ok((
            response_items,
            ledger_info_with_sigs,
            validator_change_proof,
            ledger_consistency_proof,
        ))
    }

    fn get_response_items(
        &self,
        request_items: Vec<RequestItem>,
        ledger_version: Version,
    ) -> Result<Vec<ResponseItem>> {
        request_items
            .into_iter()
            .map(|request_item| match request_item {
                RequestItem::GetAccountState { address } => Ok(ResponseItem::GetAccountState {
                    account_state_with_proof: self.get_account_state_with_proof(
                        address,
                        ledger_version,
                        ledger_version,
                    )?,
                }),
                RequestItem::GetAccountTransactionBySequenceNumber {
                    account,
                    sequence_number,
                    fetch_events,
                } => {
                    let transaction_with_proof = self.get_txn_by_account(
                        account,
                        sequence_number,
                        ledger_version,
                        fetch_events,
                    )?;

                    let proof_of_current_sequence_number = match transaction_with_proof {
                        Some(_) => None,
                        None => Some(self.get_account_state_with_proof(
                            account,
                            ledger_version,
                            ledger_version,
                        )?),
                    };

                    Ok(ResponseItem::GetAccountTransactionBySequenceNumber {
                        transaction_with_proof,
                        proof_of_current_sequence_number,
                    })
                }

                RequestItem::GetEventsByEventAccessPath {
                    access_path,
                    start_event_seq_num,
                    ascending,
                    limit,
                } => {
                    let (events_with_proof, proof_of_latest_event) = self
                        .get_events_by_query_path(
                            &access_path,
                            start_event_seq_num,
                            ascending,
                            limit,
                            ledger_version,
                        )?;
                    Ok(ResponseItem::GetEventsByEventAccessPath {
                        events_with_proof,
                        proof_of_latest_event,
                    })
                }
                RequestItem::GetTransactions {
                    start_version,
                    limit,
                    fetch_events,
                } => {
                    let txn_list_with_proof =
                        self.get_transactions(start_version, limit, ledger_version, fetch_events)?;

                    Ok(ResponseItem::GetTransactions {
                        txn_list_with_proof,
                    })
                }
            })
            .collect::<Result<Vec<_>>>()
    }

    // =========================== Libra Core Internal APIs ========================================

    /// Gets the latest state root hash together with its version.
    pub fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        let (version, txn_info) = self.ledger_store.get_latest_transaction_info()?;
        Ok((version, txn_info.state_root_hash()))
    }

    /// Gets an account state by account address, out of the ledger state indicated by the state
    /// Merkle tree root hash.
    ///
    /// This is used by libra core (executor) internally.
    pub fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        self.state_store
            .get_account_state_with_proof_by_version(address, version)
    }

    /// Gets information needed from storage during the startup of the executor or state
    /// synchronizer module.
    ///
    /// This is used by the libra core (executor, state synchronizer) internally.
    pub fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        // Get the latest ledger info. Return None if not bootstrapped.
        let (latest_ledger_info, latest_validator_set) =
            match self.ledger_store.get_startup_info()? {
                Some(x) => x,
                None => return Ok(None),
            };

        let latest_tree_state = {
            let (latest_version, txn_info) = self.ledger_store.get_latest_transaction_info()?;
            let account_state_root_hash = txn_info.state_root_hash();
            let ledger_frozen_subtree_hashes = self
                .ledger_store
                .get_ledger_frozen_subtree_hashes(latest_version)?;
            TreeState::new(
                latest_version,
                ledger_frozen_subtree_hashes,
                account_state_root_hash,
            )
        };

        let li_version = latest_ledger_info.ledger_info().version();
        assert!(latest_tree_state.version >= li_version);
        let startup_info = if latest_tree_state.version != li_version {
            // We synced to some version ahead of the version of the latest ledger info. Thus, we are still in sync mode.
            let committed_version = li_version;
            let committed_txn_info = self.ledger_store.get_transaction_info(committed_version)?;
            let committed_account_state_root_hash = committed_txn_info.state_root_hash();
            let committed_ledger_frozen_subtree_hashes = self
                .ledger_store
                .get_ledger_frozen_subtree_hashes(committed_version)?;
            StartupInfo::new(
                latest_ledger_info,
                latest_validator_set,
                TreeState::new(
                    committed_version,
                    committed_ledger_frozen_subtree_hashes,
                    committed_account_state_root_hash,
                ),
                Some(latest_tree_state),
            )
        } else {
            // The version of the latest ledger info matches other data. So the storage is not in sync mode.
            StartupInfo::new(
                latest_ledger_info,
                latest_validator_set,
                latest_tree_state,
                None,
            )
        };

        Ok(Some(startup_info))
    }

    // ================================== Backup APIs ===================================

    /// Gets an instance of `BackupHandler` for data backup purpose.
    pub fn get_backup_handler(&self) -> BackupHandler {
        BackupHandler::new(
            Arc::clone(&self.ledger_store),
            Arc::clone(&self.transaction_store),
            Arc::clone(&self.state_store),
        )
    }

    pub fn restore_account_state(
        &self,
        iter: impl Iterator<Item = (Vec<(HashValue, AccountStateBlob)>, SparseMerkleRangeProof)>,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<()> {
        let mut restore =
            JellyfishMerkleRestore::new(&*self.state_store, version, expected_root_hash)?;
        for (chunk, proof) in iter {
            restore.add_chunk(chunk, proof)?;
        }
        restore.finish()?;
        Ok(())
    }

    // ================================== Private APIs ==================================
    /// Convert a `ChangeSet` to `SealedChangeSet`.
    ///
    /// Specifically, counter increases are added to current counter values and converted to DB
    /// alternations.
    fn seal_change_set(
        &self,
        first_version: Version,
        num_txns: Version,
        mut cs: ChangeSet,
    ) -> Result<(SealedChangeSet, Option<LedgerCounters>)> {
        // Avoid reading base counter values when not necessary.
        let counters = if num_txns > 0 {
            Some(self.system_store.bump_ledger_counters(
                first_version,
                first_version + num_txns - 1,
                cs.counter_bumps,
                &mut cs.batch,
            )?)
        } else {
            None
        };

        Ok((SealedChangeSet { batch: cs.batch }, counters))
    }

    /// Write the whole schema batch including all data necessary to mutate the ledger
    /// state of some transaction by leveraging rocksdb atomicity support. Also committed are the
    /// LedgerCounters.
    fn commit(&self, sealed_cs: SealedChangeSet) -> Result<()> {
        self.db.write_schemas(sealed_cs.batch)?;

        match self.db.get_approximate_sizes_cf() {
            Ok(cf_sizes) => {
                for (cf_name, size) in cf_sizes {
                    OP_COUNTER.set(&format!("cf_size_bytes_{}", cf_name), size as usize);
                    LIBRA_STORAGE_CF_SIZE_BYTES
                        .with_label_values(&[&cf_name])
                        .set(size as i64);
                }
            }
            Err(err) => warn!(
                "Failed to get approximate size of column families: {}.",
                err
            ),
        }

        Ok(())
    }

    pub fn get_transaction_with_proof(
        &self,
        version: Version,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionWithProof> {
        let proof = {
            let (txn_info, txn_info_accumulator_proof) = self
                .ledger_store
                .get_transaction_info_with_proof(version, ledger_version)?;
            TransactionProof::new(txn_info_accumulator_proof, txn_info)
        };
        let transaction = self.transaction_store.get_transaction(version)?;

        // If events were requested, also fetch those.
        let events = if fetch_events {
            Some(self.event_store.get_events_by_version(version)?)
        } else {
            None
        };

        Ok(TransactionWithProof {
            version,
            transaction,
            events,
            proof,
        })
    }
}

impl LibraDBTrait for LibraDB {
    fn get_latest_account_state(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        let ledger_info_with_sigs = self.ledger_store.get_latest_ledger_info()?;
        let version = ledger_info_with_sigs.ledger_info().version();
        let (blob, _proof) = self
            .state_store
            .get_account_state_with_proof_by_version(address, version)?;
        Ok(blob)
    }

    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)> {
        let latest_ledger_info = self.ledger_store.get_latest_ledger_info()?;
        let ledger_info = latest_ledger_info.ledger_info();
        Ok((ledger_info.version(), ledger_info.timestamp_usecs()))
    }

    /// Returns a transaction that is the `seq_num`-th one associated with the given account. If
    /// the transaction with given `seq_num` doesn't exist, returns `None`.
    fn get_txn_by_account(
        &self,
        address: AccountAddress,
        seq_num: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>> {
        self.transaction_store
            .lookup_transaction_by_account(address, seq_num, ledger_version)?
            .map(|version| self.get_transaction_with_proof(version, ledger_version, fetch_events))
            .transpose()
    }

    /// Gets the latest version number available in the ledger.
    fn get_latest_version(&self) -> Result<Version> {
        Ok(self
            .ledger_store
            .get_latest_ledger_info()?
            .ledger_info()
            .version())
    }

    // ======================= State Synchronizer Internal APIs ===================================
    /// Gets a batch of transactions for the purpose of synchronizing state to another node.
    ///
    /// This is used by the State Synchronizer module internally.
    fn get_transactions(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        error_if_too_many_requested(limit, MAX_LIMIT)?;

        if start_version > ledger_version || limit == 0 {
            return Ok(TransactionListWithProof::new_empty());
        }

        let limit = std::cmp::min(limit, ledger_version - start_version + 1);

        let txns = (start_version..start_version + limit)
            .map(|version| Ok(self.transaction_store.get_transaction(version)?))
            .collect::<Result<Vec<_>>>()?;
        let txn_infos = (start_version..start_version + limit)
            .map(|version| Ok(self.ledger_store.get_transaction_info(version)?))
            .collect::<Result<Vec<_>>>()?;
        let events = if fetch_events {
            Some(
                (start_version..start_version + limit)
                    .map(|version| Ok(self.event_store.get_events_by_version(version)?))
                    .collect::<Result<Vec<_>>>()?,
            )
        } else {
            None
        };
        let proof = TransactionListProof::new(
            self.ledger_store.get_transaction_range_proof(
                Some(start_version),
                limit,
                ledger_version,
            )?,
            txn_infos,
        );

        Ok(TransactionListWithProof::new(
            txns,
            events,
            Some(start_version),
            proof,
        ))
    }

    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        let version = self
            .ledger_store
            .get_latest_ledger_info()?
            .ledger_info()
            .version();
        let events = self
            .get_events_by_event_key(event_key, start, false, limit, version)?
            .into_iter()
            .map(|e| (e.transaction_version, e.event))
            .collect();
        Ok(events)
    }

    fn get_state_proof(
        &self,
        known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        let ledger_info_with_sigs = self.ledger_store.get_latest_ledger_info()?;
        let ledger_info = ledger_info_with_sigs.ledger_info();
        let known_epoch = self.ledger_store.get_epoch(known_version)?;
        let validator_change_proof = if known_epoch < ledger_info.epoch() {
            let (ledger_infos_with_sigs, more) = self.get_epoch_change_ledger_infos(
                known_epoch,
                self.ledger_store.get_epoch(ledger_info.version())?,
            )?;
            ValidatorChangeProof::new(ledger_infos_with_sigs, more)
        } else {
            ValidatorChangeProof::new(vec![], /* more = */ false)
        };

        let ledger_consistency_proof = self
            .ledger_store
            .get_consistency_proof(known_version, ledger_info.version())?;
        Ok((
            ledger_info_with_sigs,
            validator_change_proof,
            ledger_consistency_proof,
        ))
    }

    fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        version: Version,
        ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        ensure!(
            version <= ledger_version,
            "The queried version {} should be equal to or older than ledger version {}.",
            version,
            ledger_version
        );
        let latest_version = self.get_latest_version()?;
        ensure!(
            ledger_version <= latest_version,
            "The ledger version {} is greater than the latest version currently in ledger: {}",
            ledger_version,
            latest_version
        );

        let (txn_info, txn_info_accumulator_proof) = self
            .ledger_store
            .get_transaction_info_with_proof(version, ledger_version)?;
        let (account_state_blob, sparse_merkle_proof) = self
            .state_store
            .get_account_state_with_proof_by_version(address, version)?;
        Ok(AccountStateWithProof::new(
            version,
            account_state_blob,
            AccountStateProof::new(txn_info_accumulator_proof, txn_info, sparse_merkle_proof),
        ))
    }
}

// Convert requested range and order to a range in ascending order.
fn get_first_seq_num_and_limit(ascending: bool, cursor: u64, limit: u64) -> Result<(u64, u64)> {
    ensure!(limit > 0, "limit should > 0, got {}", limit);

    Ok(if ascending {
        (cursor, limit)
    } else if limit <= cursor {
        (cursor - limit + 1, limit)
    } else {
        (0, cursor + 1)
    })
}
