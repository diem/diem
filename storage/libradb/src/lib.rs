// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate provides [`LibraDB`] which represents physical storage of the core Libra data
//! structures.
//!
//! It relays read/write operations on the physical storage via [`schemadb`] to the underlying
//! Key-Value storage system, and implements libra data structures on top of it.

// Used in other crates for testing.
pub mod mock_genesis;
// Used in this and other crates for testing.
pub mod test_helper;

pub mod errors;

mod event_store;
mod ledger_store;
pub mod schema;
mod state_store;
mod transaction_store;

#[cfg(test)]
mod libradb_test;

use crate::{
    errors::LibraDbError, event_store::EventStore, ledger_store::LedgerStore, schema::*,
    state_store::StateStore, transaction_store::TransactionStore,
};
use crypto::{
    hash::{CryptoHash, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use failure::prelude::*;
use itertools::{izip, zip_eq};
use lazy_static::lazy_static;
use logger::prelude::*;
use metrics::OpMetrics;
use schemadb::{ColumnFamilyOptions, ColumnFamilyOptionsMap, SchemaBatch, DB, DEFAULT_CF_NAME};
use std::{iter::Iterator, path::Path, sync::Arc, time::Instant};
use storage_proto::ExecutorStartupInfo;
use types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::EventWithProof,
    get_with_proof::{RequestItem, ResponseItem},
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccountStateProof, EventProof, SignedTransactionProof, SparseMerkleProof},
    transaction::{
        SignedTransactionWithProof, TransactionInfo, TransactionListWithProof, TransactionToCommit,
        Version,
    },
    validator_change::ValidatorChangeEventWithProof,
};

lazy_static! {
    static ref OP_COUNTER: OpMetrics = OpMetrics::new_and_registered("storage");
}

const MAX_LIMIT: u64 = 1000;
const MAX_REQUEST_ITEMS: u64 = 100;

fn error_if_too_many_requested(num_requested: u64, max_allowed: u64) -> Result<()> {
    if num_requested > max_allowed {
        Err(LibraDbError::TooManyRequested(num_requested, max_allowed).into())
    } else {
        Ok(())
    }
}

/// This holds a handle to the underlying DB responsible for physical storage and provides APIs for
/// access to the core Libra data structures.
pub struct LibraDB {
    db: Arc<DB>,
    ledger_store: LedgerStore,
    transaction_store: TransactionStore,
    state_store: StateStore,
    event_store: EventStore,
}

impl LibraDB {
    /// This creates an empty LibraDB instance on disk or opens one if it already exists.
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Self {
        let cf_opts_map: ColumnFamilyOptionsMap = [
            (
                /* LedgerInfo CF = */ DEFAULT_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (ACCOUNT_STATE_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_ACCUMULATOR_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_BY_ACCESS_PATH_CF_NAME, ColumnFamilyOptions::default()),
            (EVENT_CF_NAME, ColumnFamilyOptions::default()),
            (SIGNATURE_CF_NAME, ColumnFamilyOptions::default()),
            (SIGNED_TRANSACTION_CF_NAME, ColumnFamilyOptions::default()),
            (STATE_MERKLE_NODE_CF_NAME, ColumnFamilyOptions::default()),
            (
                TRANSACTION_ACCUMULATOR_CF_NAME,
                ColumnFamilyOptions::default(),
            ),
            (TRANSACTION_INFO_CF_NAME, ColumnFamilyOptions::default()),
            (VALIDATOR_CF_NAME, ColumnFamilyOptions::default()),
        ]
        .iter()
        .cloned()
        .collect();

        let path = db_root_path.as_ref().join("libradb");
        let instant = Instant::now();
        let db = Arc::new(
            DB::open(path.clone(), cf_opts_map)
                .unwrap_or_else(|e| panic!("LibraDB open failed: {:?}", e)),
        );

        info!(
            "Opened LibraDB at {:?} in {} ms",
            path,
            instant.elapsed().as_millis()
        );

        LibraDB {
            db: Arc::clone(&db),
            event_store: EventStore::new(Arc::clone(&db)),
            ledger_store: LedgerStore::new(Arc::clone(&db)),
            state_store: StateStore::new(Arc::clone(&db)),
            transaction_store: TransactionStore::new(Arc::clone(&db)),
        }
    }

    // ================================== Public API ==================================
    /// Returns the account state corresponding to the given version and account address with proof
    /// based on `ledger_version`
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
            .get_account_state_with_proof_by_state_root(address, txn_info.state_root_hash())?;
        Ok(AccountStateWithProof::new(
            version,
            account_state_blob,
            AccountStateProof::new(txn_info_accumulator_proof, txn_info, sparse_merkle_proof),
        ))
    }

    /// Returns events specified by `access_path` with sequence number in range designated by
    /// `start_seq_num`, `ascending` and `limit`. If ascending is true this query will return up to
    /// `limit` events that were emitted after `start_event_seq_num`. Otherwise it will return up to
    /// `limit` events in the reverse order. Both cases are inclusive.
    fn get_events_by_event_access_path(
        &self,
        access_path: &AccessPath,
        start_seq_num: u64,
        ascending: bool,
        limit: u64,
        ledger_version: Version,
    ) -> Result<(Vec<EventWithProof>, Option<AccountStateWithProof>)> {
        error_if_too_many_requested(limit, MAX_LIMIT)?;

        let get_latest = !ascending && start_seq_num == u64::max_value();
        let cursor = if get_latest {
            // Caller wants the latest, figure out the latest seq_num.
            // In the case of no events on that path, use 0 and expect empty result below.
            self.event_store
                .get_latest_sequence_number(ledger_version, access_path)?
                .unwrap_or(0)
        } else {
            start_seq_num
        };

        // Convert requested range and order to a range in ascending order.
        let (first_seq, real_limit) = get_first_seq_num_and_limit(ascending, cursor, limit)?;

        // Query the index.
        let mut event_keys = self.event_store.lookup_events_by_access_path(
            access_path,
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

        // There are two cases where we need to return proof_of_latest_event to let the caller know
        // the latest sequence number:
        //   1. The user asks for the latest event by using u64::max() as the cursor, apparently
        // he doesn't know the latest sequence number.
        //   2. We are going to return less than `real_limit` items. (Two cases can lead to that:
        // a. the cursor is beyond the latest sequence number; b. in ascending order we don't have
        // enough items to return because the latest sequence number is hit). In this case we
        // need to return the proof to convince the caller we didn't hide any item from him. Note
        // that we use `real_limit` instead of `limit` here because it takes into account the case
        // of hitting 0 in descending order, which is valid and doesn't require the proof.
        let proof_of_latest_event = if get_latest || events_with_proof.len() < real_limit as usize {
            Some(self.get_account_state_with_proof(
                access_path.address,
                ledger_version,
                ledger_version,
            )?)
        } else {
            None
        };

        Ok((events_with_proof, proof_of_latest_event))
    }

    /// Returns a signed transaction that is the `seq_num`-th one associated with the given account.
    /// If the signed transaction with given `seq_num` doesn't exist, returns `None`.
    // TODO(gzh): Use binary search for now. We may create seq_num index in the future.
    fn get_txn_by_account_and_seq(
        &self,
        address: AccountAddress,
        seq_num: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<SignedTransactionWithProof>> {
        // If txn with seq_num n is at some version, the corresponding account state at the
        // same version will be the first account state that has seq_num n + 1.
        let seq_num = seq_num + 1;
        let (mut start_version, mut end_version) = (0, ledger_version);
        while start_version < end_version {
            let mid_version = start_version + (end_version - start_version) / 2;
            let account_seq_num = self.get_account_seq_num_by_version(address, mid_version)?;
            if account_seq_num >= seq_num {
                end_version = mid_version;
            } else {
                start_version = mid_version + 1;
            }
        }
        assert_eq!(start_version, end_version);

        let seq_num_found = self.get_account_seq_num_by_version(address, start_version)?;
        if seq_num_found < seq_num {
            return Ok(None);
        } else if seq_num_found > seq_num {
            // log error
            bail!("internal error: seq_num is not continuous.")
        }
        // start_version cannot be 0 (genesis version).
        assert_eq!(
            self.get_account_seq_num_by_version(address, start_version - 1)?,
            seq_num_found - 1
        );
        self.get_transaction_with_proof(start_version, ledger_version, fetch_events)
            .map(Some)
    }

    /// Gets the latest version number available in the ledger.
    fn get_latest_version(&self) -> Result<Version> {
        Ok(self
            .ledger_store
            .get_latest_ledger_info()?
            .ledger_info()
            .version())
    }

    /// Persist transactions. Called by the executor module when either syncing nodes or committing
    /// blocks during normal operation.
    ///
    /// When `ledger_info_with_sigs` is provided, verify that the transaction accumulator root hash
    /// it carries is generated after the `txns_to_commit` are applied.
    pub fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: Version,
        ledger_info_with_sigs: &Option<LedgerInfoWithSignatures>,
    ) -> Result<()> {
        let num_txns = txns_to_commit.len() as u64;
        // ledger_info_with_sigs could be None if we are doing state synchronization. In this case
        // txns_to_commit should not be empty. Otherwise it is okay to commit empty blocks.
        ensure!(
            ledger_info_with_sigs.is_some() || num_txns > 0,
            "txns_to_commit is empty while ledger_info_with_sigs is None.",
        );

        let cur_state_root_hash = if first_version == 0 {
            *SPARSE_MERKLE_PLACEHOLDER_HASH
        } else {
            self.ledger_store
                .get_transaction_info(first_version - 1)?
                .state_root_hash()
        };

        if let Some(x) = ledger_info_with_sigs {
            let last_version = x.ledger_info().version();
            ensure!(
                first_version + num_txns - 1 == last_version,
                "Transaction batch not applicable: first_version {}, num_txns {}, last_version {}",
                first_version,
                num_txns,
                last_version
            );
        }

        // Gather db mutations to `batch`.
        let mut batch = SchemaBatch::new();

        let new_root_hash = self.save_transactions_impl(
            txns_to_commit,
            first_version,
            cur_state_root_hash,
            &mut batch,
        )?;

        // If expected ledger info is provided, verify result root hash and save the ledger info.
        if let Some(x) = ledger_info_with_sigs {
            let expected_root_hash = x.ledger_info().transaction_accumulator_hash();
            ensure!(
                new_root_hash == expected_root_hash,
                "Root hash calculated doesn't match expected. {:?} vs {:?}",
                new_root_hash,
                expected_root_hash,
            );

            self.ledger_store.put_ledger_info(x, &mut batch)?;
        }

        // Persist.
        self.commit(batch)?;
        // Only increment counter if commit(batch) succeeds.
        OP_COUNTER.inc_by("committed_txns", txns_to_commit.len());
        Ok(())
    }

    fn save_transactions_impl(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: u64,
        cur_state_root_hash: HashValue,
        mut batch: &mut SchemaBatch,
    ) -> Result<HashValue> {
        let last_version = first_version + txns_to_commit.len() as u64 - 1;

        // Account state updates. Gather account state root hashes
        let account_state_sets = txns_to_commit
            .iter()
            .map(|txn_to_commit| txn_to_commit.account_states().clone())
            .collect::<Vec<_>>();
        let state_root_hashes = self.state_store.put_account_state_sets(
            account_state_sets,
            first_version,
            cur_state_root_hash,
            &mut batch,
        )?;

        // Event updates. Gather event accumulator root hashes.
        let event_root_hashes = zip_eq(first_version..=last_version, txns_to_commit)
            .map(|(ver, txn_to_commit)| {
                self.event_store
                    .put_events(ver, txn_to_commit.events(), &mut batch)
            })
            .collect::<Result<Vec<_>>>()?;

        // Transaction updates. Gather transaction hashes.
        zip_eq(first_version..=last_version, txns_to_commit)
            .map(|(ver, txn_to_commit)| {
                self.transaction_store
                    .put_transaction(ver, txn_to_commit.signed_txn(), &mut batch)
            })
            .collect::<Result<()>>()?;
        let txn_hashes = txns_to_commit
            .iter()
            .map(|txn_to_commit| txn_to_commit.signed_txn().hash())
            .collect::<Vec<_>>();
        let gas_amounts = txns_to_commit
            .iter()
            .map(TransactionToCommit::gas_used)
            .collect::<Vec<_>>();

        // Transaction accumulator updates. Get result root hash.
        let txn_infos = izip!(
            txn_hashes,
            state_root_hashes,
            event_root_hashes,
            gas_amounts
        )
        .map(|(t, s, e, g)| TransactionInfo::new(t, s, e, g))
        .collect::<Vec<_>>();
        assert_eq!(txn_infos.len(), txns_to_commit.len());

        let new_root_hash =
            self.ledger_store
                .put_transaction_infos(first_version, &txn_infos, &mut batch)?;

        Ok(new_root_hash)
    }

    /// This backs the `UpdateToLatestLedger` public read API which returns the latest
    /// [`LedgerInfoWithSignatures`] together with items requested and proofs relative to the same
    /// ledger info.
    pub fn update_to_latest_ledger(
        &self,
        _client_known_version: u64,
        request_items: Vec<RequestItem>,
    ) -> Result<(
        Vec<ResponseItem>,
        LedgerInfoWithSignatures,
        Vec<ValidatorChangeEventWithProof>,
    )> {
        error_if_too_many_requested(request_items.len() as u64, MAX_REQUEST_ITEMS)?;

        // Get the latest ledger info and signatures
        let ledger_info_with_sigs = self.ledger_store.get_latest_ledger_info()?;
        let ledger_version = ledger_info_with_sigs.ledger_info().version();

        // Fulfill all request items
        let response_items = request_items
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
                    let signed_transaction_with_proof = self.get_txn_by_account_and_seq(
                        account,
                        sequence_number,
                        ledger_version,
                        fetch_events,
                    )?;

                    let proof_of_current_sequence_number = match signed_transaction_with_proof {
                        Some(_) => None,
                        None => Some(self.get_account_state_with_proof(
                            account,
                            ledger_version,
                            ledger_version,
                        )?),
                    };

                    Ok(ResponseItem::GetAccountTransactionBySequenceNumber {
                        signed_transaction_with_proof,
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
                        .get_events_by_event_access_path(
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
            .collect::<Result<Vec<_>>>()?;

        Ok((
            response_items,
            ledger_info_with_sigs,
            vec![], /* TODO: validator_change_events */
        ))
    }

    // =========================== Execution Internal APIs ========================================

    /// Gets an account state by account address, out of the ledger state indicated by the state
    /// Merkle tree root hash.
    ///
    /// This is used by the executor module internally.
    pub fn get_account_state_with_proof_by_state_root(
        &self,
        address: AccountAddress,
        state_root: HashValue,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        self.state_store
            .get_account_state_with_proof_by_state_root(address, state_root)
    }

    /// Gets information needed from storage during the startup of the executor module.
    ///
    /// This is used by the executor module internally.
    pub fn get_executor_startup_info(&self) -> Result<Option<ExecutorStartupInfo>> {
        // Get the latest ledger info. Return None if not bootstrapped.
        let ledger_info_with_sigs = match self.ledger_store.get_latest_ledger_info_option()? {
            Some(x) => x,
            None => return Ok(None),
        };
        let ledger_info = ledger_info_with_sigs.ledger_info().clone();

        let (latest_version, txn_info) = self.ledger_store.get_latest_transaction_info()?;

        let account_state_root_hash = txn_info.state_root_hash();

        let ledger_frozen_subtree_hashes = self
            .ledger_store
            .get_ledger_frozen_subtree_hashes(latest_version)?;

        Ok(Some(ExecutorStartupInfo {
            ledger_info,
            latest_version,
            account_state_root_hash,
            ledger_frozen_subtree_hashes,
        }))
    }

    // ======================= State Synchronizer Internal APIs ===================================
    /// Gets a batch of transactions for the purpose of synchronizing state to another node.
    ///
    /// This is used by the State Synchronizer module internally.
    pub fn get_transactions(
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
        let txn_and_txn_info_list = (start_version..start_version + limit)
            .into_iter()
            .map(|version| {
                Ok((
                    self.transaction_store.get_transaction(version)?,
                    self.ledger_store.get_transaction_info(version)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let proof_of_first_transaction = Some(
            self.ledger_store
                .get_transaction_proof(start_version, ledger_version)?,
        );
        let proof_of_last_transaction = if limit == 1 {
            None
        } else {
            Some(
                self.ledger_store
                    .get_transaction_proof(start_version + limit - 1, ledger_version)?,
            )
        };
        let events = if fetch_events {
            Some(
                (start_version..start_version + limit)
                    .into_iter()
                    .map(|version| Ok(self.event_store.get_events_by_version(version)?))
                    .collect::<Result<Vec<_>>>()?,
            )
        } else {
            None
        };

        Ok(TransactionListWithProof::new(
            txn_and_txn_info_list,
            events,
            Some(start_version),
            proof_of_first_transaction,
            proof_of_last_transaction,
        ))
    }

    // ================================== Private APIs ==================================
    /// Write the whole schema batch including all data necessary to mutate the ledge
    /// state of some transaction by leveraging rocksdb atomicity support.
    fn commit(&self, batch: SchemaBatch) -> Result<()> {
        self.db.write_schemas(batch)?;

        match self.db.get_approximate_sizes_cf() {
            Ok(cf_sizes) => {
                for (cf_name, size) in cf_sizes {
                    OP_COUNTER.set(&format!("cf_size_bytes_{}", cf_name), size as usize);
                }
            }
            Err(err) => warn!(
                "Failed to get approximate size of column families: {}.",
                err
            ),
        }

        Ok(())
    }

    fn get_account_seq_num_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<u64> {
        let (account_state_blob, _proof) = self
            .state_store
            .get_account_state_with_proof_by_state_root(
                address,
                self.ledger_store
                    .get_transaction_info(version)?
                    .state_root_hash(),
            )?;

        // If an account does not exist, we treat it as if it has sequence number 0.
        Ok(get_account_resource_or_default(&account_state_blob)?.sequence_number())
    }

    fn get_transaction_with_proof(
        &self,
        version: Version,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<SignedTransactionWithProof> {
        let proof = {
            let (txn_info, txn_info_accumulator_proof) = self
                .ledger_store
                .get_transaction_info_with_proof(version, ledger_version)?;
            SignedTransactionProof::new(txn_info_accumulator_proof, txn_info)
        };
        let signed_transaction = self.transaction_store.get_transaction(version)?;

        // If events were requested, also fetch those.
        let events = if fetch_events {
            Some(self.event_store.get_events_by_version(version)?)
        } else {
            None
        };

        Ok(SignedTransactionWithProof {
            version,
            signed_transaction,
            events,
            proof,
        })
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
