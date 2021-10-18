// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};

/// A storage service request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageServiceRequest {
    GetAccountStatesChunkWithProof(AccountStatesChunkWithProofRequest), // Fetches a list of account states with a proof
    GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest), // Fetches a list of epoch ending ledger infos
    GetNumberOfAccountsAtVersion(Version), // Fetches the number of accounts at the specified version
    GetServerProtocolVersion,              // Fetches the protocol version run by the server
    GetStorageServerSummary,               // Fetches a summary of the storage server state
    GetTransactionOutputsWithProof(TransactionOutputsWithProofRequest), // Fetches a list of transaction outputs with a proof
    GetTransactionsWithProof(TransactionsWithProofRequest), // Fetches a list of transactions with a proof
}

/// A storage service response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageServiceResponse {
    AccountStatesChunkWithProof(AccountStatesChunkWithProof),
    EpochEndingLedgerInfos(EpochChangeProof),
    NumberOfAccountsAtVersion(u64),
    ServerProtocolVersion(ServerProtocolVersion),
    StorageServiceError(StorageServiceError),
    StorageServerSummary(StorageServerSummary),
    TransactionOutputsWithProof(TransactionOutputListWithProof),
    TransactionsWithProof(TransactionListWithProof),
}

/// A storage service request for fetching a list of account states at a
/// specified version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountStatesChunkWithProofRequest {
    pub version: u64,                     // The version to fetch the account states at
    pub start_account_index: u64,         // The account index to start fetching account states
    pub expected_num_account_states: u64, // Expected number of account states to fetch
}

/// A storage service request for fetching a transaction output list with a
/// corresponding proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutputsWithProofRequest {
    pub proof_version: u64,        // The version the proof should be relative to
    pub start_version: u64,        // The starting version of the transaction output list
    pub expected_num_outputs: u64, // Expected number of transaction outputs in the list
}

/// A storage service request for fetching a transaction list with a
/// corresponding proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionsWithProofRequest {
    pub proof_version: u64, // The version the proof should be relative to
    pub start_version: u64, // The starting version of the transaction list
    pub expected_num_transactions: u64, // Expected number of transactions in the list
    pub include_events: bool, // Whether or not to include events in the response
}

/// A storage service request for fetching a list of epoch ending ledger infos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochEndingLedgerInfoRequest {
    pub start_epoch: u64,
    pub expected_end_epoch: u64,
}

/// A storage service error that can be returned to the client on a failure
/// to process a service request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageServiceError {
    InternalError,
}

/// The protocol version run by this server. Clients request this first to
/// identify what API calls and data requests the server supports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerProtocolVersion {
    pub protocol_version: u64, // The storage server version run by this instance.
}

/// A storage server summary, containing a summary of the information held
/// by the corresponding server instance. This is useful for identifying the
/// data that a server instance can provide, as well as relevant metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageServerSummary {
    pub protocol_metadata: ProtocolMetadata,
    pub data_summary: DataSummary,
}

/// A summary of the protocol metadata for the storage service instance, such as
/// the maximum chunk sizes supported for different requests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolMetadata {
    pub max_epoch_chunk_size: u64, // The max number of epochs the server can return in a single chunk
    pub max_transaction_chunk_size: u64, // The max number of transactions the server can return in a single chunk
    pub max_transaction_output_chunk_size: u64, // The max number of transaction outputs the server can return in a single chunk
    pub max_account_states_chunk_size: u64, // The max number of account states the server can return in a single chunk
}

/// A type alias for different epochs.
pub type Epoch = u64;

/// A summary of the data actually held by the storage service instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataSummary {
    /// The ledger info corresponding to the highest synced version in storage.
    /// This indicates the highest version and epoch that storage can prove.
    pub synced_ledger_info: LedgerInfoWithSignatures,
    /// The range of epoch ending ledger infos in storage, e.g., if the range
    /// is [(X,Y)], it means all epoch ending ledger infos for epochs X->Y
    /// (inclusive) are held.
    pub epoch_ending_ledger_infos: CompleteDataRange<Epoch>,
    /// The range of transactions held in storage, e.g., if the range is
    /// [(X,Y)], it means all transactions for versions X->Y (inclusive) are held.
    pub transactions: CompleteDataRange<Version>,
    /// The range of transaction outputs held in storage, e.g., if the range
    /// is [(X,Y)], it means all transaction outputs for versions X->Y
    /// (inclusive) are held.
    pub transaction_outputs: CompleteDataRange<Version>,
    /// The range of account states held in storage, e.g., if the range is
    /// [(X,Y)], it means all account states are held for every version X->Y
    /// (inclusive).
    pub account_states: CompleteDataRange<Version>,
}

/// A struct representing a data range (lowest to highest, inclusive) where data
/// is complete (i.e. there are no missing pieces of data).
/// This is used to provide a summary of the data currently held in storage, e.g.
/// a CompleteDataRange<Version> of (A,B) means all versions A->B (inclusive).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteDataRange<T> {
    pub lowest: T,
    pub highest: T,
}

impl<T: Ord> CompleteDataRange<T> {
    pub fn new(lowest: T, highest: T) -> Self {
        Self { lowest, highest }
    }

    /// Returns true iff the given data item is within this range
    pub fn contains(&self, data_item: T) -> bool {
        data_item >= self.lowest && data_item <= self.highest
    }
}
