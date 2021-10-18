// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::streaming_client::Epoch;
use diem_data_client::DataClientResponse;
use diem_types::{
    account_state_blob::AccountStatesChunkWithProof,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{
        default_protocol::{TransactionListWithProof, TransactionOutputListWithProof},
        Version,
    },
};
use std::fmt::{Debug, Formatter};

/// A unique ID used to identify each notification.
pub type NotificationId = u64;

/// A single data notification with an ID and data payload.
#[derive(Debug)]
pub struct DataNotification {
    pub notification_id: NotificationId,
    pub data_payload: DataPayload,
}

/// A single payload (e.g. chunk) of requested data.
#[derive(Debug)]
pub enum DataPayload {
    AccountStatesWithProof(AccountStatesChunkWithProof),
    EpochEndingLedgerInfos(Vec<LedgerInfoWithSignatures>),
    TransactionOutputsWithProof(TransactionOutputListWithProof),
    TransactionsWithProof(TransactionListWithProof),
}

/// A sent data notification that tracks the original client request and the
/// client response forwarded along a stream. This is useful for re-fetching.
#[derive(Clone, Debug)]
pub struct SentDataNotification {
    pub client_request: DataClientRequest,
    pub client_response: DataClientResponse,
}

/// A request that has been sent to the Diem data client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataClientRequest {
    AccountsWithProof(AccountsWithProofRequest),
    EpochEndingLedgerInfos(EpochEndingLedgerInfosRequest),
    TransactionsWithProof(TransactionsWithProofRequest),
    TransactionOutputsWithProof(TransactionOutputsWithProofRequest),
}

/// A request for fetching account states.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountsWithProofRequest {
    pub version: Version,
    pub start_index: u64,
    pub end_index: u64,
}

/// A client request for fetching epoch ending ledger infos.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpochEndingLedgerInfosRequest {
    pub start_epoch: Epoch,
    pub end_epoch: Epoch,
}

/// A client request for fetching transactions with proofs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionsWithProofRequest {
    pub start_version: Version,
    pub end_version: Version,
    pub max_proof_version: Version,
    pub include_events: bool,
}

/// A client request for fetching transaction outputs with proofs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionOutputsWithProofRequest {
    pub start_version: Version,
    pub end_version: Version,
    pub max_proof_version: Version,
}

/// A pending client response where data has been requested from the
/// network and will be available in `client_response` when received.
pub struct PendingClientResponse {
    pub client_request: DataClientRequest,
    pub client_response: Option<Result<DataClientResponse, diem_data_client::Error>>,
}

impl Debug for PendingClientResponse {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Client request: {:?}, client response: {:?}",
            self.client_request, self.client_response
        )
    }
}
