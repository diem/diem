// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_config::config::StorageServiceConfig;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{TransactionListWithProof, TransactionOutputListWithProof, Version},
};
use num_traits::{int::PrimInt, Zero};
#[cfg(test)]
use proptest::{
    arbitrary::{any, Arbitrary},
    strategy::{BoxedStrategy, Strategy},
};
use serde::{de, Deserialize, Serialize};
use std::convert::TryFrom;
use thiserror::Error;

/// A type alias for different epochs.
pub type Epoch = u64;

pub type Result<T, E = StorageServiceError> = ::std::result::Result<T, E>;

/// A storage service error that can be returned to the client on a failure
/// to process a service request.
#[derive(Clone, Debug, Deserialize, Eq, Error, PartialEq, Serialize)]
pub enum StorageServiceError {
    #[error("Internal service error: {0}")]
    InternalError(String),
}

/// A single storage service message sent or received over DiemNet.
#[derive(Clone, Debug, Deserialize, Serialize)]
// TODO(philiphayes): do something about this without making it ugly :(
#[allow(clippy::large_enum_variant)]
pub enum StorageServiceMessage {
    /// A request to the storage service.
    Request(StorageServiceRequest),
    /// A response from the storage service. If there was an error while handling
    /// the request, the service will return an [`StorageServiceError`] error.
    Response(Result<StorageServiceResponse>),
}

/// A storage service request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum StorageServiceRequest {
    GetAccountStatesChunkWithProof(AccountStatesChunkWithProofRequest), // Fetches a list of account states with a proof
    GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest), // Fetches a list of epoch ending ledger infos
    GetNumberOfAccountsAtVersion(Version), // Fetches the number of accounts at the specified version
    GetServerProtocolVersion,              // Fetches the protocol version run by the server
    GetStorageServerSummary,               // Fetches a summary of the storage server state
    GetTransactionOutputsWithProof(TransactionOutputsWithProofRequest), // Fetches a list of transaction outputs with a proof
    GetTransactionsWithProof(TransactionsWithProofRequest), // Fetches a list of transactions with a proof
}

impl StorageServiceRequest {
    /// Returns a summary label for the request
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::GetAccountStatesChunkWithProof(_) => "get_account_states_chunk_with_proof",
            Self::GetEpochEndingLedgerInfos(_) => "get_epoch_ending_ledger_infos",
            Self::GetNumberOfAccountsAtVersion(_) => "get_number_of_accounts_at_version",
            Self::GetServerProtocolVersion => "get_server_protocol_version",
            Self::GetStorageServerSummary => "get_storage_server_summary",
            Self::GetTransactionOutputsWithProof(_) => "get_transaction_outputs_with_proof",
            Self::GetTransactionsWithProof(_) => "get_transactions_with_proof",
        }
    }

    pub fn is_get_storage_server_summary(&self) -> bool {
        matches!(self, &Self::GetStorageServerSummary)
    }
}

/// A storage service response
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
// TODO(philiphayes): do something about this without making it ugly :(
#[allow(clippy::large_enum_variant)]
pub enum StorageServiceResponse {
    AccountStatesChunkWithProof(AccountStatesChunkWithProof),
    EpochEndingLedgerInfos(EpochChangeProof),
    NumberOfAccountsAtVersion(u64),
    ServerProtocolVersion(ServerProtocolVersion),
    StorageServerSummary(StorageServerSummary),
    TransactionOutputsWithProof(TransactionOutputListWithProof),
    TransactionsWithProof(TransactionListWithProof),
}

// TODO(philiphayes): is there a proc-macro for this?
impl StorageServiceResponse {
    /// Returns a summary label for the response
    pub fn get_label(&self) -> &'static str {
        match self {
            Self::AccountStatesChunkWithProof(_) => "account_states_chunk_with_proof",
            Self::EpochEndingLedgerInfos(_) => "epoch_ending_ledger_infos",
            Self::NumberOfAccountsAtVersion(_) => "number_of_accounts_at_version",
            Self::ServerProtocolVersion(_) => "server_protocol_version",
            Self::StorageServerSummary(_) => "storage_server_summary",
            Self::TransactionOutputsWithProof(_) => "transaction_outputs_with_proof",
            Self::TransactionsWithProof(_) => "transactions_with_proof",
        }
    }
}

#[derive(Clone, Debug, Error)]
#[error("unexpected response variant: {0}")]
pub struct UnexpectedResponseError(pub String);

// Conversions from the outer StorageServiceResponse enum to the inner types.
// TODO(philiphayes): is there a proc-macro for this?

impl TryFrom<StorageServiceResponse> for AccountStatesChunkWithProof {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::AccountStatesChunkWithProof(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected account_states_chunk_with_proof, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for EpochChangeProof {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::EpochEndingLedgerInfos(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected epoch_ending_ledger_infos, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for u64 {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::NumberOfAccountsAtVersion(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected number_of_accounts_at_version, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for ServerProtocolVersion {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::ServerProtocolVersion(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected server_protocol_version, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for StorageServerSummary {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::StorageServerSummary(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected storage_server_summary, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for TransactionOutputListWithProof {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::TransactionOutputsWithProof(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected transaction_outputs_with_proof, found {}",
                response.get_label()
            ))),
        }
    }
}

impl TryFrom<StorageServiceResponse> for TransactionListWithProof {
    type Error = UnexpectedResponseError;
    fn try_from(response: StorageServiceResponse) -> Result<Self, Self::Error> {
        match response {
            StorageServiceResponse::TransactionsWithProof(inner) => Ok(inner),
            _ => Err(UnexpectedResponseError(format!(
                "expected transactions_with_proof, found {}",
                response.get_label()
            ))),
        }
    }
}

/// A storage service request for fetching a list of account states at a
/// specified version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AccountStatesChunkWithProofRequest {
    pub version: u64,             // The version to fetch the account states at
    pub start_account_index: u64, // The account index to start fetching account states
    pub end_account_index: u64,   // The account index to stop fetching account states (inclusive)
}

/// A storage service request for fetching a transaction output list with a
/// corresponding proof.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransactionOutputsWithProofRequest {
    pub proof_version: u64, // The version the proof should be relative to
    pub start_version: u64, // The starting version of the transaction output list
    pub end_version: u64,   // The ending version of the transaction output list (inclusive)
}

/// A storage service request for fetching a transaction list with a
/// corresponding proof.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransactionsWithProofRequest {
    pub proof_version: u64,   // The version the proof should be relative to
    pub start_version: u64,   // The starting version of the transaction list
    pub end_version: u64,     // The ending version of the transaction list (inclusive)
    pub include_events: bool, // Whether or not to include events in the response
}

/// A storage service request for fetching a list of epoch ending ledger infos.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpochEndingLedgerInfoRequest {
    pub start_epoch: u64,
    pub expected_end_epoch: u64,
}

/// The protocol version run by this server. Clients request this first to
/// identify what API calls and data requests the server supports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ServerProtocolVersion {
    pub protocol_version: u64, // The storage server version run by this instance.
}

/// A storage server summary, containing a summary of the information held
/// by the corresponding server instance. This is useful for identifying the
/// data that a server instance can provide, as well as relevant metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageServerSummary {
    pub protocol_metadata: ProtocolMetadata,
    pub data_summary: DataSummary,
}

impl StorageServerSummary {
    pub fn can_service(&self, request: &StorageServiceRequest) -> bool {
        self.protocol_metadata.can_service(request) && self.data_summary.can_service(request)
    }
}

/// A summary of the protocol metadata for the storage service instance, such as
/// the maximum chunk sizes supported for different requests.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProtocolMetadata {
    pub max_epoch_chunk_size: u64, // The max number of epochs the server can return in a single chunk
    pub max_transaction_chunk_size: u64, // The max number of transactions the server can return in a single chunk
    pub max_transaction_output_chunk_size: u64, // The max number of transaction outputs the server can return in a single chunk
    pub max_account_states_chunk_size: u64, // The max number of account states the server can return in a single chunk
}

impl ProtocolMetadata {
    pub fn can_service(&self, request: &StorageServiceRequest) -> bool {
        use StorageServiceRequest::*;
        match request {
            GetServerProtocolVersion
            | GetStorageServerSummary
            | GetNumberOfAccountsAtVersion(_) => true,
            GetAccountStatesChunkWithProof(request) => {
                CompleteDataRange::new(request.start_account_index, request.end_account_index)
                    .map_or(false, |range| {
                        range.len().map_or(false, |chunk_size| {
                            self.max_account_states_chunk_size >= chunk_size
                        })
                    })
            }
            GetEpochEndingLedgerInfos(request) => CompleteDataRange::new(
                request.start_epoch,
                request.expected_end_epoch,
            )
            .map_or(false, |range| {
                range
                    .len()
                    .map_or(false, |chunk_size| self.max_epoch_chunk_size >= chunk_size)
            }),
            GetTransactionOutputsWithProof(request) => CompleteDataRange::new(
                request.start_version,
                request.end_version,
            )
            .map_or(false, |range| {
                range.len().map_or(false, |chunk_size| {
                    self.max_transaction_output_chunk_size >= chunk_size
                })
            }),
            GetTransactionsWithProof(request) => CompleteDataRange::new(
                request.start_version,
                request.end_version,
            )
            .map_or(false, |range| {
                range.len().map_or(false, |chunk_size| {
                    self.max_transaction_chunk_size >= chunk_size
                })
            }),
        }
    }
}

impl Default for ProtocolMetadata {
    fn default() -> Self {
        let config = StorageServiceConfig::default();
        Self {
            max_epoch_chunk_size: config.max_epoch_chunk_size,
            max_transaction_chunk_size: config.max_transaction_chunk_size,
            max_transaction_output_chunk_size: config.max_transaction_output_chunk_size,
            max_account_states_chunk_size: config.max_account_states_chunk_sizes,
        }
    }
}

/// A summary of the data actually held by the storage service instance.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DataSummary {
    /// The ledger info corresponding to the highest synced version in storage.
    /// This indicates the highest version and epoch that storage can prove.
    pub synced_ledger_info: Option<LedgerInfoWithSignatures>,
    /// The range of epoch ending ledger infos in storage, e.g., if the range
    /// is [(X,Y)], it means all epoch ending ledger infos for epochs X->Y
    /// (inclusive) are held.
    pub epoch_ending_ledger_infos: Option<CompleteDataRange<Epoch>>,
    /// The range of transactions held in storage, e.g., if the range is
    /// [(X,Y)], it means all transactions for versions X->Y (inclusive) are held.
    pub transactions: Option<CompleteDataRange<Version>>,
    /// The range of transaction outputs held in storage, e.g., if the range
    /// is [(X,Y)], it means all transaction outputs for versions X->Y
    /// (inclusive) are held.
    pub transaction_outputs: Option<CompleteDataRange<Version>>,
    /// The range of account states held in storage, e.g., if the range is
    /// [(X,Y)], it means all account states are held for every version X->Y
    /// (inclusive).
    pub account_states: Option<CompleteDataRange<Version>>,
}

impl DataSummary {
    pub fn can_service(&self, request: &StorageServiceRequest) -> bool {
        use StorageServiceRequest::*;
        match request {
            // storage services can always serve these metadata requests
            GetServerProtocolVersion => true,
            GetStorageServerSummary => true,
            GetAccountStatesChunkWithProof(request) => {
                let proof_version = request.version;

                let can_serve_accounts = self
                    .account_states
                    .map(|range| range.contains(request.version))
                    .unwrap_or(false);

                let can_create_proof = self
                    .synced_ledger_info
                    .as_ref()
                    .map(|li| li.ledger_info().version() >= proof_version)
                    .unwrap_or(false);

                can_serve_accounts && can_create_proof
            }
            GetEpochEndingLedgerInfos(request) => {
                let desired_range =
                    match CompleteDataRange::new(request.start_epoch, request.expected_end_epoch) {
                        Ok(desired_range) => desired_range,
                        Err(_) => return false,
                    };
                self.epoch_ending_ledger_infos
                    .map(|range| range.superset_of(&desired_range))
                    .unwrap_or(false)
            }
            GetNumberOfAccountsAtVersion(version) => self
                .account_states
                .map(|range| range.contains(*version))
                .unwrap_or(false),
            GetTransactionOutputsWithProof(request) => {
                let desired_range =
                    match CompleteDataRange::new(request.start_version, request.end_version) {
                        Ok(desired_range) => desired_range,
                        Err(_) => return false,
                    };

                let can_serve_outputs = self
                    .transaction_outputs
                    .map(|range| range.superset_of(&desired_range))
                    .unwrap_or(false);

                let can_create_proof = self
                    .synced_ledger_info
                    .as_ref()
                    .map(|li| li.ledger_info().version() >= request.proof_version)
                    .unwrap_or(false);

                can_serve_outputs && can_create_proof
            }
            GetTransactionsWithProof(request) => {
                let desired_range =
                    match CompleteDataRange::new(request.start_version, request.end_version) {
                        Ok(desired_range) => desired_range,
                        Err(_) => return false,
                    };

                let can_serve_txns = self
                    .transactions
                    .map(|range| range.superset_of(&desired_range))
                    .unwrap_or(false);

                let can_create_proof = self
                    .synced_ledger_info
                    .as_ref()
                    .map(|li| li.ledger_info().version() >= request.proof_version)
                    .unwrap_or(false);

                can_serve_txns && can_create_proof
            }
        }
    }
}

#[derive(Clone, Debug, Error)]
#[error("data range cannot be degenerate")]
pub struct DegenerateRangeError;

/// A struct representing a contiguous, non-empty data range (lowest to highest,
/// inclusive) where data is complete (i.e. there are no missing pieces of data).
///
/// This is used to provide a summary of the data currently held in storage, e.g.
/// a CompleteDataRange<Version> of (A,B) means all versions A->B (inclusive).
///
/// Note: `CompleteDataRanges` are never degenerate (lowest > highest) and the
/// range length is always expressible without overflowing. Constructing a
/// degenerate range via `new` will return an `Err`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CompleteDataRange<T> {
    lowest: T,
    highest: T,
}

fn range_length_checked<T: PrimInt>(lowest: T, highest: T) -> Result<T, DegenerateRangeError> {
    // len = highest - lowest + 1
    // Note: the order of operations here is important; we need to subtract first
    // before we (+1) to ensure we don't underflow when highest == lowest.
    highest
        .checked_sub(&lowest)
        .and_then(|value| value.checked_add(&T::one()))
        .ok_or(DegenerateRangeError)
}

impl<T: PrimInt> CompleteDataRange<T> {
    pub fn new(lowest: T, highest: T) -> Result<Self, DegenerateRangeError> {
        if lowest > highest || range_length_checked(lowest, highest).is_err() {
            Err(DegenerateRangeError)
        } else {
            Ok(Self { lowest, highest })
        }
    }

    /// Create a data range given the lower bound and the length of the range.
    pub fn from_len(lowest: T, len: T) -> Result<Self, DegenerateRangeError> {
        // highest = lowest + len - 1
        // Note: the order of operations here is important
        let highest = len
            .checked_sub(&T::one())
            .and_then(|addend| lowest.checked_add(&addend))
            .ok_or(DegenerateRangeError)?;
        Self::new(lowest, highest)
    }

    #[inline]
    pub fn lowest(&self) -> T {
        self.lowest
    }

    #[inline]
    pub fn highest(&self) -> T {
        self.highest
    }

    /// Returns the length of the data range.
    #[inline]
    pub fn len(&self) -> Result<T, DegenerateRangeError> {
        self.highest
            .checked_sub(&self.lowest)
            .and_then(|value| value.checked_add(&T::one()))
            .ok_or(DegenerateRangeError)
    }

    /// Returns true iff the given item is within this range
    pub fn contains(&self, item: T) -> bool {
        self.lowest <= item && item <= self.highest
    }

    /// Returns true iff this range is a superset of the other data range.
    pub fn superset_of(&self, other: &Self) -> bool {
        self.lowest <= other.lowest && other.highest <= self.highest
    }
}

impl<T: Zero> CompleteDataRange<T> {
    pub fn from_genesis(highest: T) -> Self {
        Self {
            lowest: T::zero(),
            highest,
        }
    }
}

impl<'de, T> de::Deserialize<'de> for CompleteDataRange<T>
where
    T: PrimInt + de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(rename = "CompleteDataRange")]
        struct Value<U> {
            lowest: U,
            highest: U,
        }

        let value = Value::<T>::deserialize(deserializer)?;
        Self::new(value.lowest, value.highest).map_err(D::Error::custom)
    }
}

#[cfg(test)]
impl<T> Arbitrary for CompleteDataRange<T>
where
    T: PrimInt + Arbitrary + 'static,
{
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (any::<T>(), any::<T>())
            .prop_filter_map("degenerate range", |(lowest, highest)| {
                CompleteDataRange::new(lowest, highest).ok()
            })
            .boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok};
    use diem_crypto::hash::HashValue;
    use diem_types::{block_info::BlockInfo, ledger_info::LedgerInfo};
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    fn mock_ledger_info(version: Version) -> LedgerInfoWithSignatures {
        LedgerInfoWithSignatures::new(
            LedgerInfo::new(
                BlockInfo::new(0, 0, HashValue::zero(), HashValue::zero(), version, 0, None),
                HashValue::zero(),
            ),
            BTreeMap::new(),
        )
    }

    fn range(lowest: u64, highest: u64) -> CompleteDataRange<u64> {
        CompleteDataRange::new(lowest, highest).unwrap()
    }

    fn get_epochs_request(start: Epoch, end: Epoch) -> StorageServiceRequest {
        StorageServiceRequest::GetEpochEndingLedgerInfos(EpochEndingLedgerInfoRequest {
            start_epoch: start,
            expected_end_epoch: end,
        })
    }

    fn get_txns_request(proof: Version, start: Version, end: Version) -> StorageServiceRequest {
        StorageServiceRequest::GetTransactionsWithProof(TransactionsWithProofRequest {
            proof_version: proof,
            start_version: start,
            end_version: end,
            include_events: true,
        })
    }

    fn get_txn_outputs_request(
        proof_version: Version,
        start_version: Version,
        end_version: Version,
    ) -> StorageServiceRequest {
        StorageServiceRequest::GetTransactionOutputsWithProof(TransactionOutputsWithProofRequest {
            proof_version,
            start_version,
            end_version,
        })
    }

    fn get_account_state_chunks_request(
        version: Version,
        start_account_index: u64,
        end_account_index: u64,
    ) -> StorageServiceRequest {
        StorageServiceRequest::GetAccountStatesChunkWithProof(AccountStatesChunkWithProofRequest {
            version,
            start_account_index,
            end_account_index,
        })
    }

    fn get_account_states_request(version: Version) -> StorageServiceRequest {
        get_account_state_chunks_request(version, 0, 1000)
    }

    #[test]
    fn test_complete_data_range() {
        // good ranges
        assert_ok!(CompleteDataRange::new(0, 0));
        assert_ok!(CompleteDataRange::new(10, 10));
        assert_ok!(CompleteDataRange::new(10, 20));
        assert_ok!(CompleteDataRange::new(u64::MAX, u64::MAX));

        // degenerate ranges
        assert_err!(CompleteDataRange::new(1, 0));
        assert_err!(CompleteDataRange::new(20, 10));
        assert_err!(CompleteDataRange::new(u64::MAX, 0));
        assert_err!(CompleteDataRange::new(u64::MAX, 1));

        // range length overflow edge case
        assert_ok!(CompleteDataRange::new(1, u64::MAX));
        assert_ok!(CompleteDataRange::new(0, u64::MAX - 1));
        assert_err!(CompleteDataRange::new(0, u64::MAX));
    }

    #[test]
    fn test_data_summary_can_service_epochs_request() {
        let summary = DataSummary {
            epoch_ending_ledger_infos: Some(range(100, 200)),
            ..Default::default()
        };

        // in range, can service

        assert!(summary.can_service(&get_epochs_request(100, 200)));
        assert!(summary.can_service(&get_epochs_request(125, 175)));
        assert!(summary.can_service(&get_epochs_request(100, 100)));
        assert!(summary.can_service(&get_epochs_request(150, 150)));
        assert!(summary.can_service(&get_epochs_request(200, 200)));

        // out of range, can't service

        assert!(!summary.can_service(&get_epochs_request(99, 200)));
        assert!(!summary.can_service(&get_epochs_request(100, 201)));
        assert!(!summary.can_service(&get_epochs_request(50, 250)));
        assert!(!summary.can_service(&get_epochs_request(50, 150)));
        assert!(!summary.can_service(&get_epochs_request(150, 250)));

        // degenerate range, can't service

        assert!(!summary.can_service(&get_epochs_request(150, 149)));
    }

    #[test]
    fn test_data_summary_can_service_txns_request() {
        let summary = DataSummary {
            synced_ledger_info: Some(mock_ledger_info(250)),
            transactions: Some(range(100, 200)),
            ..Default::default()
        };

        // in range, can service

        assert!(summary.can_service(&get_txns_request(225, 100, 200)));
        assert!(summary.can_service(&get_txns_request(225, 125, 175)));
        assert!(summary.can_service(&get_txns_request(225, 100, 100)));
        assert!(summary.can_service(&get_txns_request(225, 150, 150)));
        assert!(summary.can_service(&get_txns_request(225, 200, 200)));
        assert!(summary.can_service(&get_txns_request(250, 200, 200)));

        // out of range, can't service

        assert!(!summary.can_service(&get_txns_request(225, 99, 200)));
        assert!(!summary.can_service(&get_txns_request(225, 100, 201)));
        assert!(!summary.can_service(&get_txns_request(225, 50, 250)));
        assert!(!summary.can_service(&get_txns_request(225, 50, 150)));
        assert!(!summary.can_service(&get_txns_request(225, 150, 250)));

        assert!(!summary.can_service(&get_txns_request(300, 100, 200)));
        assert!(!summary.can_service(&get_txns_request(300, 125, 175)));
        assert!(!summary.can_service(&get_txns_request(300, 100, 100)));
        assert!(!summary.can_service(&get_txns_request(300, 150, 150)));
        assert!(!summary.can_service(&get_txns_request(300, 200, 200)));
        assert!(!summary.can_service(&get_txns_request(251, 200, 200)));
    }

    #[test]
    fn test_data_summary_can_service_txn_outputs_request() {
        let summary = DataSummary {
            synced_ledger_info: Some(mock_ledger_info(250)),
            transaction_outputs: Some(range(100, 200)),
            ..Default::default()
        };

        // in range and can provide proof => can service
        assert!(summary.can_service(&get_txn_outputs_request(225, 100, 200)));
        assert!(summary.can_service(&get_txn_outputs_request(225, 125, 175)));
        assert!(summary.can_service(&get_txn_outputs_request(225, 100, 100)));
        assert!(summary.can_service(&get_txn_outputs_request(225, 150, 150)));
        assert!(summary.can_service(&get_txn_outputs_request(225, 200, 200)));
        assert!(summary.can_service(&get_txn_outputs_request(250, 200, 200)));

        // can provide proof, but out of range => cannot service
        assert!(!summary.can_service(&get_txn_outputs_request(225, 99, 200)));
        assert!(!summary.can_service(&get_txn_outputs_request(225, 100, 201)));
        assert!(!summary.can_service(&get_txn_outputs_request(225, 50, 250)));
        assert!(!summary.can_service(&get_txn_outputs_request(225, 50, 150)));
        assert!(!summary.can_service(&get_txn_outputs_request(225, 150, 250)));

        // in range, but cannot provide proof => cannot service
        assert!(!summary.can_service(&get_txn_outputs_request(300, 100, 200)));
        assert!(!summary.can_service(&get_txn_outputs_request(300, 125, 175)));
        assert!(!summary.can_service(&get_txn_outputs_request(300, 100, 100)));
        assert!(!summary.can_service(&get_txn_outputs_request(300, 150, 150)));
        assert!(!summary.can_service(&get_txn_outputs_request(300, 200, 200)));
        assert!(!summary.can_service(&get_txn_outputs_request(251, 200, 200)));

        // invalid range
        assert!(!summary.can_service(&get_txn_outputs_request(225, 175, 125)));
    }

    #[test]
    fn test_data_summary_can_service_account_states_chunk_request() {
        let summary = DataSummary {
            synced_ledger_info: Some(mock_ledger_info(250)),
            account_states: Some(range(100, 300)),
            ..Default::default()
        };

        // in range and can provide proof => can service
        assert!(summary.can_service(&get_account_states_request(100)));
        assert!(summary.can_service(&get_account_states_request(200)));
        assert!(summary.can_service(&get_account_states_request(250)));

        // in range, but cannot provide proof => cannot service
        assert!(!summary.can_service(&get_account_states_request(251)));
        assert!(!summary.can_service(&get_account_states_request(300)));

        // can provide proof, but out of range ==> cannot service
        assert!(!summary.can_service(&get_account_states_request(50)));
        assert!(!summary.can_service(&get_account_states_request(99)));
    }

    #[test]
    fn test_protocol_metadata_can_service() {
        let metadata = ProtocolMetadata {
            max_transaction_chunk_size: 100,
            max_epoch_chunk_size: 100,
            max_transaction_output_chunk_size: 100,
            max_account_states_chunk_size: 100,
        };

        assert!(metadata.can_service(&get_txns_request(200, 100, 199)));
        assert!(!metadata.can_service(&get_txns_request(200, 100, 200)));

        assert!(metadata.can_service(&get_epochs_request(100, 199)));
        assert!(!metadata.can_service(&get_epochs_request(100, 200)));

        assert!(metadata.can_service(&get_txn_outputs_request(200, 100, 199)));
        assert!(!metadata.can_service(&get_txn_outputs_request(200, 100, 200)));

        assert!(metadata.can_service(&get_account_state_chunks_request(200, 100, 199)));
        assert!(!metadata.can_service(&get_account_state_chunks_request(200, 100, 200)));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        #[test]
        fn test_data_summary_length_invariant(range in any::<CompleteDataRange<u64>>()) {
            // should not panic
            let _ = range.len();
        }
    }
}
