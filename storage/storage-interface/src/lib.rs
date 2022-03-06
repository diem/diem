// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use diem_crypto::{hash::SPARSE_MERKLE_PLACEHOLDER_HASH, HashValue};
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::diem_root_address,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof, AccountStatesChunkWithProof},
    contract_event::{ContractEvent, EventByVersionWithProof, EventWithProof},
    epoch_change::EpochChangeProof,
    epoch_state::EpochState,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    move_resource::MoveStorage,
    on_chain_config::{
        default_access_path_for_config, experimental_access_path_for_config, ConfigID,
    },
    proof::{
        definition::LeafCount, AccumulatorConsistencyProof, SparseMerkleProof,
        SparseMerkleRangeProof, TransactionAccumulatorSummary,
    },
    state_proof::StateProof,
    transaction::{
        AccountTransactionsWithProof, TransactionInfo, TransactionListWithProof,
        TransactionOutputListWithProof, TransactionToCommit, TransactionWithProof, Version,
    },
    write_set::WriteSet,
};
use move_core_types::resolver::{ModuleResolver, ResourceResolver};
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, sync::Arc};
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

    #[cfg(any(feature = "fuzzing"))]
    pub fn new_for_testing() -> Self {
        use diem_types::on_chain_config::ValidatorSet;

        let latest_ledger_info =
            LedgerInfoWithSignatures::genesis(HashValue::zero(), ValidatorSet::empty());
        let latest_epoch_state = None;
        let committed_tree_state = TreeState {
            num_transactions: 0,
            ledger_frozen_subtree_hashes: Vec::new(),
            state_checkpoint_hash: *SPARSE_MERKLE_PLACEHOLDER_HASH,
            write_sets_after_checkpoint: Vec::new(),
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

    pub fn into_latest_tree_state(self) -> TreeState {
        self.synced_tree_state.unwrap_or(self.committed_tree_state)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TreeState {
    pub num_transactions: LeafCount,
    pub ledger_frozen_subtree_hashes: Vec<HashValue>,
    /// Root hash of the state checkpoint (global sparse merkle tree).
    pub state_checkpoint_hash: HashValue,
    /// The state checkpoint can be at or before the latest version, if the latter case, this is
    /// the write sets between the state checkpoint and the latest version. Applying this to the
    /// state checkpoint results in a in-mem latest world state.
    /// N.b the latest version minus `write_sets_after_checkpoint.len()` is the state checkpoint
    /// version.
    pub write_sets_after_checkpoint: Vec<WriteSet>,
}

impl TreeState {
    pub fn new(
        num_transactions: LeafCount,
        ledger_frozen_subtree_hashes: Vec<HashValue>,
        state_checkpoint_root_hash: HashValue,
        write_set_after_checkpoint: Vec<WriteSet>,
    ) -> Self {
        Self {
            num_transactions,
            ledger_frozen_subtree_hashes,
            state_checkpoint_hash: state_checkpoint_root_hash,
            write_sets_after_checkpoint: write_set_after_checkpoint,
        }
    }

    pub fn new_at_state_checkpoint(
        num_transactions: LeafCount,
        ledger_frozen_subtree_hashes: Vec<HashValue>,
        state_root_hash: HashValue,
    ) -> Self {
        Self {
            num_transactions,
            ledger_frozen_subtree_hashes,
            state_checkpoint_hash: state_root_hash,
            write_sets_after_checkpoint: Vec::new(),
        }
    }

    pub fn describe(&self) -> &'static str {
        if self.num_transactions != 0 {
            "DB has been bootstrapped."
        } else if self.state_checkpoint_hash != *SPARSE_MERKLE_PLACEHOLDER_HASH {
            "DB has no transaction, but a non-empty pre-genesis state."
        } else {
            "DB is empty, has no transaction or state."
        }
    }
}

pub trait StateSnapshotReceiver<V> {
    fn add_chunk(
        &mut self,
        chunk: Vec<(HashValue, V)>,
        proof: SparseMerkleRangeProof,
    ) -> Result<()>;

    fn finish(self) -> Result<()>;

    fn finish_box(self: Box<Self>) -> Result<()>;
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

impl From<bcs::Error> for Error {
    fn from(error: bcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<diem_secure_net::Error> for Error {
    fn from(error: diem_secure_net::Error) -> Self {
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
/// expected of a Diem DB
#[allow(unused_variables)]
pub trait DbReader: Send + Sync {
    /// See [`DiemDB::get_epoch_ending_ledger_infos`].
    ///
    /// [`DiemDB::get_epoch_ending_ledger_infos`]:
    /// ../diemdb/struct.DiemDB.html#method.get_epoch_ending_ledger_infos
    fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<EpochChangeProof> {
        unimplemented!()
    }

    /// See [`DiemDB::get_transactions`].
    ///
    /// [`DiemDB::get_transactions`]: ../diemdb/struct.DiemDB.html#method.get_transactions
    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        unimplemented!()
    }

    /// See [`DiemDB::get_transaction_by_hash`].
    ///
    /// [`DiemDB::get_transaction_by_hash`]: ../diemdb/struct.DiemDB.html#method.get_transaction_by_hash
    fn get_transaction_by_hash(
        &self,
        hash: HashValue,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>> {
        unimplemented!()
    }

    /// See [`DiemDB::get_transaction_by_version`].
    ///
    /// [`DiemDB::get_transaction_by_version`]: ../diemdb/struct.DiemDB.html#method.get_transaction_by_version
    fn get_transaction_by_version(
        &self,
        version: Version,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionWithProof> {
        unimplemented!()
    }

    /// See [`DiemDB::get_txn_set_version`].
    ///
    /// [`DiemDB::get_first_txn_version`]: ../diemdb/struct.DiemDB.html#method.get_first_txn_version
    fn get_first_txn_version(&self) -> Result<Option<Version>> {
        unimplemented!()
    }

    /// See [`DiemDB::get_first_write_set_version`].
    ///
    /// [`DiemDB::get_first_write_set_version`]: ../diemdb/struct.DiemDB.html#method.get_first_write_set_version
    fn get_first_write_set_version(&self) -> Result<Option<Version>> {
        unimplemented!()
    }

    /// See [`DiemDB::get_transaction_outputs`].
    ///
    /// [`DiemDB::get_transaction_outputs`]: ../diemdb/struct.DiemDB.html#method.get_transaction_outputs
    fn get_transaction_outputs(
        &self,
        start_version: Version,
        limit: u64,
        ledger_version: Version,
    ) -> Result<TransactionOutputListWithProof> {
        unimplemented!()
    }

    /// Returns events by given event key
    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        order: Order,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        unimplemented!()
    }

    /// Returns events by given event key
    fn get_events_with_proofs(
        &self,
        event_key: &EventKey,
        start: u64,
        order: Order,
        limit: u64,
        known_version: Option<u64>,
    ) -> Result<Vec<EventWithProof>> {
        unimplemented!()
    }

    /// See [`DiemDB::get_block_timestamp`].
    ///
    /// [`DiemDB::get_block_timestamp`]:
    /// ../diemdb/struct.DiemDB.html#method.get_block_timestamp
    fn get_block_timestamp(&self, version: u64) -> Result<u64> {
        unimplemented!()
    }

    /// Returns the [`NewBlockEvent`] for the block containing the requested
    /// `version` and proof that the block actually contains the `version`.
    fn get_event_by_version_with_proof(
        &self,
        event_key: &EventKey,
        event_version: u64,
        proof_version: u64,
    ) -> Result<EventByVersionWithProof> {
        unimplemented!()
    }

    /// Gets the version of the last transaction committed before timestamp,
    /// a commited block at or after the required timestamp must exist (otherwise it's possible
    /// the next block committed as a timestamp smaller than the one in the request).
    fn get_last_version_before_timestamp(
        &self,
        _timestamp: u64,
        _ledger_version: Version,
    ) -> Result<Version> {
        unimplemented!()
    }

    /// See [`DiemDB::get_latest_account_state`].
    ///
    /// [`DiemDB::get_latest_account_state`]:
    /// ../diemdb/struct.DiemDB.html#method.get_latest_account_state
    fn get_latest_account_state(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        unimplemented!()
    }

    /// Returns the latest ledger info.
    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }

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
    /// See [`DiemDB::get_startup_info`].
    ///
    /// [`DiemDB::get_startup_info`]:
    /// ../diemdb/struct.DiemDB.html#method.get_startup_info
    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    /// Returns a transaction that is the `seq_num`-th one associated with the given account. If
    /// the transaction with given `seq_num` doesn't exist, returns `None`.
    fn get_account_transaction(
        &self,
        address: AccountAddress,
        seq_num: u64,
        include_events: bool,
        ledger_version: Version,
    ) -> Result<Option<TransactionWithProof>> {
        unimplemented!()
    }

    /// Returns the list of transactions sent by an account with `address` starting
    /// at sequence number `seq_num`. Will return no more than `limit` transactions.
    /// Will ignore transactions with `txn.version > ledger_version`. Optionally
    /// fetch events for each transaction when `fetch_events` is `true`.
    fn get_account_transactions(
        &self,
        address: AccountAddress,
        seq_num: u64,
        limit: u64,
        include_events: bool,
        ledger_version: Version,
    ) -> Result<AccountTransactionsWithProof> {
        unimplemented!()
    }

    /// Returns proof of new state for a given ledger info with signatures relative to version known
    /// to client
    fn get_state_proof_with_ledger_info(
        &self,
        known_version: u64,
        ledger_info: LedgerInfoWithSignatures,
    ) -> Result<StateProof> {
        unimplemented!()
    }

    /// Returns proof of new state relative to version known to client
    fn get_state_proof(&self, known_version: u64) -> Result<StateProof> {
        unimplemented!()
    }

    /// Returns the account state corresponding to the given version and account address with proof
    /// based on `ledger_version`
    fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        version: Version,
        ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        self.get_account_state_with_partial_proof(address, version, 0, ledger_version)
    }

    /// Returns the account state corresponding to the given version and account address with proof
    /// based on `ledger_version`
    fn get_account_state_with_partial_proof(
        &self,
        address: AccountAddress,
        version: Version,
        starting_depth: usize,
        ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        unimplemented!()
    }

    // Gets an account state by account address, out of the ledger state indicated by the state
    // Merkle tree root with a sparse merkle proof proving state tree root.
    // See [`DiemDB::get_account_state_with_proof_by_version`].
    //
    // [`DiemDB::get_account_state_with_proof_by_version`]:
    // ../diemdb/struct.DiemDB.html#method.get_account_state_with_proof_by_version
    //
    // This is used by diem core (executor) internally.
    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(
        Option<AccountStateBlob>,
        SparseMerkleProof<AccountStateBlob>,
    )> {
        self.get_account_state_with_partial_proof_by_version(address, version, 0)
    }

    fn get_account_state_with_partial_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
        starting_depth: usize,
    ) -> Result<(
        Option<AccountStateBlob>,
        SparseMerkleProof<AccountStateBlob>,
    )> {
        unimplemented!()
    }

    /// Gets the latest TreeState no matter if db has been bootstrapped.
    /// Used by the Db-bootstrapper.
    fn get_latest_tree_state(&self) -> Result<TreeState> {
        unimplemented!()
    }

    /// Get the ledger info of the epoch that `known_version` belongs to.
    fn get_epoch_ending_ledger_info(&self, known_version: u64) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }

    /// Gets the latest transaction info.
    /// N.B. Unlike get_startup_info(), even if the db is not bootstrapped, this can return `Some`
    /// -- those from a db-restore run.
    fn get_latest_transaction_info_option(&self) -> Result<Option<(Version, TransactionInfo)>> {
        unimplemented!()
    }

    /// Gets the transaction accumulator root hash at specified version.
    /// Caller must guarantee the version is not greater than the latest version.
    fn get_accumulator_root_hash(&self, _version: Version) -> Result<HashValue> {
        unimplemented!()
    }

    /// Gets an [`AccumulatorConsistencyProof`] starting from `client_known_version`
    /// (or pre-genesis if `None`) until `ledger_version`.
    ///
    /// In other words, if the client has an accumulator summary for
    /// `client_known_version`, they can use the result from this API to efficiently
    /// extend their accumulator to `ledger_version` and prove that the new accumulator
    /// is consistent with their old accumulator. By consistent, we mean that by
    /// appending the actual `ledger_version - client_known_version` transactions
    /// to the old accumulator summary you get the new accumulator summary.
    ///
    /// If the client is starting up for the first time and has no accumulator
    /// summary yet, they can call this with `client_known_version=None`, i.e.,
    /// pre-genesis, to get the complete accumulator summary up to `ledger_version`.
    fn get_accumulator_consistency_proof(
        &self,
        _client_known_version: Option<Version>,
        _ledger_version: Version,
    ) -> Result<AccumulatorConsistencyProof> {
        unimplemented!()
    }

    /// A convenience function for building a [`TransactionAccumulatorSummary`]
    /// at the given `ledger_version`.
    ///
    /// Note: this is roughly equivalent to calling
    /// `DbReader::get_accumulator_consistency_proof(None, ledger_version)`.
    fn get_accumulator_summary(
        &self,
        ledger_version: Version,
    ) -> Result<TransactionAccumulatorSummary> {
        let genesis_consistency_proof =
            self.get_accumulator_consistency_proof(None, ledger_version)?;
        TransactionAccumulatorSummary::try_from_genesis_proof(
            genesis_consistency_proof,
            ledger_version,
        )
    }

    /// Returns total number of accounts at given version.
    fn get_account_count(&self, version: Version) -> Result<usize> {
        unimplemented!()
    }

    /// Get a chunk of account data, addressed by the index of the account.
    fn get_account_chunk_with_proof(
        &self,
        version: Version,
        start_idx: usize,
        chunk_size: usize,
    ) -> Result<AccountStatesChunkWithProof> {
        unimplemented!()
    }

    /// Get the state prune window config value.
    fn get_state_prune_window(&self) -> Option<usize> {
        unimplemented!()
    }
}

impl MoveStorage for &dyn DbReader {
    fn fetch_resource(&self, access_path: AccessPath) -> Result<Vec<u8>> {
        self.fetch_resource_by_version(access_path, self.fetch_synced_version()?)
    }

    fn fetch_resource_by_version(
        &self,
        access_path: AccessPath,
        version: Version,
    ) -> Result<Vec<u8>> {
        let (account_state_blob, _) =
            self.get_account_state_with_proof_by_version(access_path.address, version)?;
        let account_state =
            AccountState::try_from(&account_state_blob.ok_or_else(|| {
                format_err!("missing blob in account state/account does not exist")
            })?)?;

        Ok(account_state
            .get(&access_path.path)
            .ok_or_else(|| format_err!("no value found in account state"))?
            .clone())
    }

    fn fetch_config_by_version(&self, config_id: ConfigID, version: Version) -> Result<Vec<u8>> {
        let diem_root_state = AccountState::try_from(
            &self
                .get_account_state_with_proof_by_version(diem_root_address(), version)?
                .0
                .ok_or_else(|| {
                    format_err!("missing blob in account state/account does not exist")
                })?,
        )?;

        match diem_root_state.get(&experimental_access_path_for_config(config_id).path) {
            Some(config) => Ok(config.to_vec()),
            _ => diem_root_state
                .get(&default_access_path_for_config(config_id).path)
                .map_or_else(
                    || {
                        Err(format_err!(
                            "no config {} found in diem root account state",
                            config_id
                        ))
                    },
                    |bytes| Ok(bytes.to_vec()),
                ),
        }
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
/// expected of a Diem DB. This adds write APIs to DbReader.
#[allow(unused_variables)]
pub trait DbWriter: Send + Sync {
    /// Persist transactions. Called by the executor module when either syncing nodes or committing
    /// blocks during normal operation.
    /// See [`DiemDB::save_transactions`].
    ///
    /// [`DiemDB::save_transactions`]: ../diemdb/struct.DiemDB.html#method.save_transactions
    fn save_transactions(
        &self,
        txns_to_commit: &[TransactionToCommit],
        first_version: Version,
        ledger_info_with_sigs: Option<&LedgerInfoWithSignatures>,
    ) -> Result<()> {
        unimplemented!()
    }

    /// Get a (stateful) state snapshot receiver.
    ///
    /// Chunk of accounts need to be added via `add_chunk()` before finishing up with `finish_box()`
    fn get_state_snapshot_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<Box<dyn StateSnapshotReceiver<AccountStateBlob>>> {
        unimplemented!()
    }
}

pub trait MoveDbReader:
    DbReader + ResourceResolver<Error = anyhow::Error> + ModuleResolver<Error = anyhow::Error>
{
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
