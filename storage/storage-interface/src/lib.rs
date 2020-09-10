// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use itertools::Itertools;
use libra_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    move_resource::MoveStorage,
    proof::{definition::LeafCount, AccumulatorConsistencyProof, SparseMerkleProof},
    transaction::{
        TransactionInfo, TransactionListWithProof, TransactionToCommit, TransactionWithProof,
        Version,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};
use thiserror::Error;

#[cfg(any(feature = "testing", feature = "fuzzing"))]
pub mod mock;
pub mod state_view;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartupInfo {
    /// The latest ledger info.
    pub latest_ledger_info: LedgerInfoWithSignatures,
    /// If the above ledger info doesn't carry a validator set, the latest validator set. Otherwise
    /// `None`.
    pub latest_epoch_state: Option<EpochState>,
    pub committed_tree_state: TreeState,
    pub synced_tree_state: Option<TreeState>,
}

impl StartupInfo {
    pub fn new(
        latest_ledger_info: LedgerInfoWithSignatures,
        latest_epoch_state: Option<EpochState>,
        committed_tree_state: TreeState,
        synced_tree_state: Option<TreeState>,
    ) -> Self {
        Self {
            latest_ledger_info,
            latest_epoch_state,
            committed_tree_state,
            synced_tree_state,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_for_testing() -> Self {
        use libra_types::on_chain_config::ValidatorSet;

        let latest_ledger_info =
            LedgerInfoWithSignatures::genesis(HashValue::zero(), ValidatorSet::empty());
        let latest_epoch_state = None;
        let committed_tree_state = TreeState {
            num_transactions: 0,
            ledger_frozen_subtree_hashes: Vec::new(),
            account_state_root_hash: *SPARSE_MERKLE_PLACEHOLDER_HASH,
        };
        let synced_tree_state = None;

        Self {
            latest_ledger_info,
            latest_epoch_state,
            committed_tree_state,
            synced_tree_state,
        }
    }

    pub fn get_epoch_state(&self) -> &EpochState {
        self.latest_ledger_info
            .ledger_info()
            .next_epoch_state()
            .unwrap_or_else(|| {
                self.latest_epoch_state
                    .as_ref()
                    .expect("EpochState must exist")
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreeState {
    pub num_transactions: LeafCount,
    pub ledger_frozen_subtree_hashes: Vec<HashValue>,
    pub account_state_root_hash: HashValue,
}

impl TreeState {
    pub fn new(
        num_transactions: LeafCount,
        ledger_frozen_subtree_hashes: Vec<HashValue>,
        account_state_root_hash: HashValue,
    ) -> Self {
        Self {
            num_transactions,
            ledger_frozen_subtree_hashes,
            account_state_root_hash,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.num_transactions == 0
            && self.account_state_root_hash == *SPARSE_MERKLE_PLACEHOLDER_HASH
    }

    pub fn describe(&self) -> String {
        if self.num_transactions != 0 {
            format!(
                "DB has {} transactions in it, state root hash is {}.",
                self.num_transactions, self.account_state_root_hash
            )
        } else if self.account_state_root_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
            format!(
                "DB has no transactions in it, but has pre-genesis state, state root hash is {}.",
                self.account_state_root_hash
            )
        } else {
            "DB is empty, has no transactions or state.".into()
        }
    }
}

#[derive(Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Service error: {:?}", error)]
    ServiceError { error: String },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self::ServiceError {
            error: format!("{}", error),
        }
    }
}

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<libra_secure_net::Error> for Error {
    fn from(error: libra_secure_net::Error) -> Self {
        Self::ServiceError {
            error: format!("{}", error),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Order {
    Ascending,
    Descending,
}

/// Trait that is implemented by a DB that supports certain public (to client) read APIs
/// expected of a Libra DB
pub trait DbReader: Send + Sync {
    /// See [`LibraDB::get_epoch_ending_ledger_infos`].
    ///
    /// [`LibraDB::get_epoch_ending_ledger_infos`]:
    /// ../libradb/struct.LibraDB.html#method.get_epoch_ending_ledger_infos
    fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<EpochChangeProof>;

    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// Returns events by given event key
    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        order: Order,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>>;

    /// See [`LibraDB::get_block_timestamp`].
    ///
    /// [`LibraDB::get_block_timestamp`]:
    /// ../libradb/struct.LibraDB.html#method.get_block_timestamp
    fn get_block_timestamp(&self, version: u64) -> Result<u64>;

    /// See [`LibraDB::get_latest_account_state`].
    ///
    /// [`LibraDB::get_latest_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_account_state
    fn get_latest_account_state(&self, address: AccountAddress)
        -> Result<Option<AccountStateBlob>>;

    /// Returns the latest ledger info.
    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures>;

    /// Returns the latest ledger info.
    fn get_latest_version(&self) -> Result<Version> {
        Ok(self.get_latest_ledger_info()?.ledger_info().version())
    }

    /// Returns the latest version and committed block timestamp
    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)> {
        let ledger_info_with_sig = self.get_latest_ledger_info()?;
        let ledger_info = ledger_info_with_sig.ledger_info();
        Ok((ledger_info.version(), ledger_info.timestamp_usecs()))
    }

    /// Gets information needed from storage during the main node startup.
    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    fn get_txn_by_account(
        &self,
        address: AccountAddress,
        seq_num: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>>;

    /// Returns proof of new state for a given ledger info with signatures relative to version known
    /// to client
    fn get_state_proof_with_ledger_info(
        &self,
        known_version: u64,
        ledger_info: LedgerInfoWithSignatures,
    ) -> Result<(EpochChangeProof, AccumulatorConsistencyProof)>;

    /// Returns proof of new state relative to version known to client
    fn get_state_proof(
        &self,
        known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        EpochChangeProof,
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

    // Gets an account state by account address, out of the ledger state indicated by the state
    // Merkle tree root with a sparse merkle proof proving state tree root.
    // See [`LibraDB::get_account_state_with_proof_by_version`].
    //
    // [`LibraDB::get_account_state_with_proof_by_version`]:
    // ../libradb/struct.LibraDB.html#method.get_account_state_with_proof_by_version
    //
    // This is used by libra core (executor) internally.
    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// See [`LibraDB::get_latest_state_root`].
    ///
    /// [`LibraDB::get_latest_state_root`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_state_root
    fn get_latest_state_root(&self) -> Result<(Version, HashValue)>;

    /// Gets the latest TreeState no matter if db has been bootstrapped.
    /// Used by the Db-bootstrapper.
    fn get_latest_tree_state(&self) -> Result<TreeState>;

    /// Get the ledger info of the epoch that `known_version` belongs to.
    fn get_epoch_ending_ledger_info(&self, known_version: u64) -> Result<LedgerInfoWithSignatures>;

    /// Gets the latest transaction info.
    /// N.B. Unlike get_startup_info(), even if the db is not bootstrapped, this can return `Some`
    /// -- those from a db-restore run.
    fn get_latest_transaction_info_option(&self) -> Result<Option<(Version, TransactionInfo)>> {
        unimplemented!()
    }
}

impl MoveStorage for &dyn DbReader {
    fn batch_fetch_resources(&self, access_paths: Vec<AccessPath>) -> Result<Vec<Vec<u8>>> {
        self.batch_fetch_resources_by_version(access_paths, self.fetch_synced_version()?)
    }

    fn batch_fetch_resources_by_version(
        &self,
        access_paths: Vec<AccessPath>,
        version: Version,
    ) -> Result<Vec<Vec<u8>>> {
        let addresses: Vec<AccountAddress> = access_paths
            .iter()
            .collect::<HashSet<_>>()
            .iter()
            .map(|path| path.address)
            .collect();

        let results = addresses
            .iter()
            .map(|addr| self.get_account_state_with_proof_by_version(*addr, version))
            .collect::<Result<Vec<_>>>()?;

        // Account address --> AccountState
        let account_states = addresses
            .iter()
            .zip_eq(results)
            .map(|(addr, (blob, _proof))| {
                let account_state = AccountState::try_from(&blob.ok_or_else(|| {
                    format_err!("missing blob in account state/account does not exist")
                })?)?;
                Ok((addr, account_state))
            })
            .collect::<Result<HashMap<_, AccountState>>>()?;

        access_paths
            .iter()
            .map(|path| {
                Ok(account_states
                    .get(&path.address)
                    .ok_or_else(|| format_err!("missing account state for queried access path"))?
                    .get(&path.path)
                    .ok_or_else(|| format_err!("no value found in account state"))?
                    .clone())
            })
            .collect()
    }

    fn fetch_synced_version(&self) -> Result<u64> {
        let (synced_version, _) = self
            .get_latest_transaction_info_option()
            .map_err(|e| {
                format_err!(
                    "[MoveStorage] Failed fetching latest transaction info: {}",
                    e
                )
            })?
            .ok_or_else(|| format_err!("[MoveStorage] Latest transaction info not found."))?;
        Ok(synced_version)
    }
}

/// Trait that is implemented by a DB that supports certain public (to client) write APIs
/// expected of a Libra DB. This adds write APIs to DbReader.
pub trait DbWriter: Send + Sync {
    /// Persist transactions. Called by the executor module when either syncing nodes or committing
    /// blocks during normal operation.
    /// See [`LibraDB::save_transactions`].
    ///
    /// [`LibraDB::save_transactions`]: ../libradb/struct.LibraDB.html#method.save_transactions
    fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: Version,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct DbReaderWriter {
    pub reader: Arc<dyn DbReader>,
    pub writer: Arc<dyn DbWriter>,
}

impl DbReaderWriter {
    pub fn new<D: 'static + DbReader + DbWriter>(db: D) -> Self {
        let reader = Arc::new(db);
        let writer = Arc::clone(&reader);

        Self { reader, writer }
    }

    pub fn from_arc<D: 'static + DbReader + DbWriter>(arc_db: Arc<D>) -> Self {
        let reader = Arc::clone(&arc_db);
        let writer = Arc::clone(&arc_db);

        Self { reader, writer }
    }

    pub fn wrap<D: 'static + DbReader + DbWriter>(db: D) -> (Arc<D>, Self) {
        let arc_db = Arc::new(db);
        (Arc::clone(&arc_db), Self::from_arc(arc_db))
    }
}

impl<D> From<D> for DbReaderWriter
where
    D: 'static + DbReader + DbWriter,
{
    fn from(db: D) -> Self {
        Self::new(db)
    }
}

/// Network types for storage service
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum StorageRequest {
    GetAccountStateWithProofByVersionRequest(Box<GetAccountStateWithProofByVersionRequest>),
    GetStartupInfoRequest,
    SaveTransactionsRequest(Box<SaveTransactionsRequest>),
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct GetAccountStateWithProofByVersionRequest {
    /// The access path to query with.
    pub address: AccountAddress,

    /// The version the query is based on.
    pub version: Version,
}

impl GetAccountStateWithProofByVersionRequest {
    /// Constructor.
    pub fn new(address: AccountAddress, version: Version) -> Self {
        Self { address, version }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SaveTransactionsRequest {
    pub txns_to_commit: Vec<TransactionToCommit>,
    pub first_version: Version,
    pub ledger_info_with_signatures: Option<LedgerInfoWithSignatures>,
}

impl SaveTransactionsRequest {
    /// Constructor.
    pub fn new(
        txns_to_commit: Vec<TransactionToCommit>,
        first_version: Version,
        ledger_info_with_signatures: Option<LedgerInfoWithSignatures>,
    ) -> Self {
        SaveTransactionsRequest {
            txns_to_commit,
            first_version,
            ledger_info_with_signatures,
        }
    }
}
