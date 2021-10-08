// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use async_trait::async_trait;
use diem_crypto::hash::HashValue;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};
use serde::{Deserialize, Serialize};
use storage_service_types::{CompleteDataRange, Epoch};
use thiserror::Error;

/// An error returned by the Diem Data Client for failed API calls.
#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("The requested data is unavailable and cannot be found! Error: {0}")]
    DataIsUnavailable(String),
    #[error("The requested data is too large: {0}")]
    DataIsTooLarge(String),
    #[error("Timed out waiting for a response: {0}")]
    TimeoutWaitingForResponse(String),
    #[error("Unexpected error encountered: {0}")]
    UnexpectedErrorEncountered(String),
}

/// A response error that users of the Diem Data Client can use to notify
/// the Data Client about invalid or malformed responses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ResponseError {
    InvalidPayloadDataType,
    MissingData,
    ProofVerificationError,
}

/// The API offered by the Diem Data Client.
#[async_trait]
pub trait DiemDataClient {
    /// Returns a single account states chunk with proof, containing the accounts
    /// from start to end index (inclusive) at the specified version. The proof
    /// version is the same as the specified version.
    async fn get_account_states_with_proof(
        &self,
        version: u64,
        start_index: HashValue,
        end_index: HashValue,
    ) -> Result<DataClientResponse, Error>;

    /// Returns all epoch ending ledger infos between start and end (inclusive).
    /// If the data cannot be fetched (e.g., the number of epochs is too large),
    /// an error is returned.
    async fn get_epoch_ending_ledger_infos(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> Result<DataClientResponse, Error>;

    /// Returns a global summary of the data currently available in the network.
    fn get_global_data_summary(&self) -> Result<DataClientResponse, Error>;

    /// Returns the number of account states at the specified version.
    async fn get_number_of_account_states(&self, version: u64)
        -> Result<DataClientResponse, Error>;

    /// Returns a transaction output list with proof object, with transaction
    /// outputs from start to end versions (inclusive). The proof is relative to
    /// the specified `proof_version`. If the data cannot be fetched (e.g., the
    /// number of transaction outputs is too large), an error is returned.
    async fn get_transaction_outputs_with_proof(
        &self,
        proof_version: u64,
        start_version: u64,
        end_version: u64,
    ) -> Result<DataClientResponse, Error>;

    /// Returns a transaction list with proof object, with transactions from
    /// start to end versions (inclusive). The proof is relative to the specified
    /// `proof_version`. If `include_events` is true, events are included in the
    /// proof. If the data cannot be fetched (e.g., the number of transactions is
    /// too large), an error is returned.
    async fn get_transactions_with_proof(
        &self,
        proof_version: u64,
        start_version: u64,
        end_version: u64,
        include_events: bool,
    ) -> Result<DataClientResponse, Error>;

    /// Notifies the Diem Data Client about a previously received response that
    /// was bad (e.g., invalid or malformed).
    ///
    /// Note: this is required because the Diem Data Client can only fetch
    /// data from peers in the network, but it is not able to fully verify that
    /// the given data responses are valid (e.g., it is unable to verify proofs).
    /// This API call provides a simple feedback mechanism for users of the Diem
    /// Data Client to alert it to bad responses so that the peers responsible
    /// for providing this data can be penalized. The `response_id` is the handle
    /// used by clients to notify the Diem Data Client of invalid responses.
    async fn notify_bad_response(
        &self,
        response_id: u64,
        response_error: ResponseError,
    ) -> Result<(), Error>;
}

/// A response from the Data Client for a single API call.
///
/// Note: the `response_id` is a simple handle returned by the Diem Data Client
/// that allows API callers to notify the Diem Data Client that the given
/// response payload is bad (e.g., it contains invalid or malformed data, or
/// the proof failed verification). This can be done using the
/// `notify_bad_response()` API call above.
#[derive(Clone, Debug)]
pub struct DataClientResponse {
    pub response_id: u64,
    pub response_payload: DataClientPayload,
}

/// The payload returned in a Data Client response.
#[derive(Clone, Debug)]
pub enum DataClientPayload {
    AccountStatesWithProof(AccountStatesChunkWithProof),
    EpochEndingLedgerInfos(Vec<LedgerInfoWithSignatures>),
    GlobalDataSummary(GlobalDataSummary),
    NumberOfAccountStates(u64),
    TransactionOutputsWithProof(TransactionOutputListWithProof),
    TransactionsWithProof(TransactionListWithProof),
}

/// A snapshot of the global state of data available in the Diem network.
#[derive(Clone, Debug)]
pub struct GlobalDataSummary {
    pub advertised_data: AdvertisedData,
    pub optimal_chunk_sizes: OptimalChunkSizes,
}

impl GlobalDataSummary {
    /// Returns an empty global data summary. This can be used on startup
    /// before the global state is known, or for testing.
    pub fn empty() -> Self {
        GlobalDataSummary {
            advertised_data: AdvertisedData {
                account_states: vec![],
                epoch_ending_ledger_infos: vec![],
                synced_ledger_infos: vec![],
                transactions: vec![],
                transaction_outputs: vec![],
            },
            optimal_chunk_sizes: OptimalChunkSizes {
                account_states_chunk_size: 0,
                epoch_chunk_size: 0,
                transaction_chunk_size: 0,
                transaction_output_chunk_size: 0,
            },
        }
    }
}

/// Holds the optimal chunk sizes that clients should use when
/// requesting data. This makes the request *more likely* to succeed.
#[derive(Clone, Debug)]
pub struct OptimalChunkSizes {
    pub account_states_chunk_size: u64,
    pub epoch_chunk_size: u64,
    pub transaction_chunk_size: u64,
    pub transaction_output_chunk_size: u64,
}

/// A summary of all data that is currently advertised in the network.
#[derive(Clone, Debug)]
pub struct AdvertisedData {
    /// The ranges of account states advertised, e.g., if a range is
    /// (X,Y), it means all account states are held for every version X->Y
    /// (inclusive).
    pub account_states: Vec<CompleteDataRange<Version>>,

    /// The ranges of epoch ending ledger infos advertised, e.g., if a range
    /// is (X,Y), it means all epoch ending ledger infos for epochs X->Y
    /// (inclusive) are available.
    pub epoch_ending_ledger_infos: Vec<CompleteDataRange<Epoch>>,

    /// The ledger infos corresponding to the highest synced versions
    /// currently advertised.
    pub synced_ledger_infos: Vec<LedgerInfoWithSignatures>,

    /// The ranges of transactions advertised, e.g., if a range is
    /// (X,Y), it means all transactions for versions X->Y (inclusive)
    /// are available.
    pub transactions: Vec<CompleteDataRange<Version>>,

    /// The ranges of transaction outputs advertised, e.g., if a range
    /// is (X,Y), it means all transaction outputs for versions X->Y
    /// (inclusive) are available.
    pub transaction_outputs: Vec<CompleteDataRange<Version>>,
}
