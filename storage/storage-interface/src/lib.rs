// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_crypto::HashValue;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccumulatorConsistencyProof, SparseMerkleProof},
    transaction::{TransactionListWithProof, TransactionToCommit, TransactionWithProof, Version},
    validator_change::ValidatorChangeProof,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_proto::StartupInfo;

/// Trait that is implemented by a DB that supports certain public (to client) read APIs
/// expected of a Libra DB
pub trait DbReader: Send + Sync {
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
        ascending: bool,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>>;

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
    ) -> Result<(ValidatorChangeProof, AccumulatorConsistencyProof)>;

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

    /// Gets an account state by account address, out of the ledger state indicated by the state
    /// Merkle tree root hash.
    ///
    /// This is used by libra core (executor) internally.
    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)>;

    /// Gets the latest state root hash together with its version.
    fn get_latest_state_root(&self) -> Result<(Version, HashValue)>;
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

    pub fn wrap<D: 'static + DbReader + DbWriter>(db: D) -> (Arc<D>, Self) {
        let arc_db = Arc::new(db);
        let reader = Arc::clone(&arc_db);
        let writer = Arc::clone(&arc_db);

        (arc_db, Self { reader, writer })
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
pub enum StorageMsg {
    /// RPC to get an account state with corresponding sparse merkle proof..
    GetAccountStateWithProofByVersionRequest(Box<GetAccountStateWithProofByVersionRequest>),
    GetAccountStateWithProofByVersionResponse(Box<GetAccountStateWithProofByVersionResponse>),

    // RPC to save transactions
    SaveTransactionsRequest(Box<SaveTransactionsRequest>),
    SaveTransactionsResponse,
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

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct GetAccountStateWithProofByVersionResponse {
    /// The account state blob requested.
    pub account_state_blob: Option<AccountStateBlob>,

    /// The state root hash the query is based on.
    pub sparse_merkle_proof: SparseMerkleProof,
}

impl GetAccountStateWithProofByVersionResponse {
    /// Constructor.
    pub fn new(
        account_state_blob: Option<AccountStateBlob>,
        sparse_merkle_proof: SparseMerkleProof,
    ) -> Self {
        Self {
            account_state_blob,
            sparse_merkle_proof,
        }
    }
}

impl Into<(Option<AccountStateBlob>, SparseMerkleProof)>
    for GetAccountStateWithProofByVersionResponse
{
    fn into(self) -> (Option<AccountStateBlob>, SparseMerkleProof) {
        (self.account_state_blob, self.sparse_merkle_proof)
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
