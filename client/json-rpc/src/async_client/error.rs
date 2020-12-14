// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::{types as jsonrpc, JsonRpcError, JsonRpcResponse};

#[derive(Debug)]
pub enum Error {
    // Error when send http request failed
    NetworkError(reqwest::Error),
    // Response http status is not 200
    InvalidHTTPStatus(String, reqwest::StatusCode),
    // Response body can't be decoded as json-rpc response
    InvalidHTTPResponse(reqwest::Error),
    // Decoded JSON-RPC does not match JSON-RPC spec
    InvalidRpcResponse(JsonRpcResponse),
    // Decode response result to specific data type failed
    DeserializeResponseJsonError(serde_json::Error),
    // JSON-RPC error
    JsonRpcError(JsonRpcError),
    // JSON-RPC Response result is null
    ResultNotFound(JsonRpcResponse),
    // Server response is version / timestamp is older than known version / timestamp
    StaleResponseError(JsonRpcResponse),
    // Server response chain id does not match previous response chain id
    ChainIdMismatch(JsonRpcResponse),
    // There was a timeout waiting for the response
    ResponseTimeout(String),
    // Unexpected error, should never happen, likely is a bug if it happens.
    UnexpectedError(UnexpectedError),
}

impl Error {
    pub fn unexpected_bcs_error(e: bcs::Error) -> Self {
        Error::UnexpectedError(UnexpectedError::BCSError(e))
    }
    pub fn unexpected_invalid_response_id(resp: JsonRpcResponse) -> Self {
        Error::UnexpectedError(UnexpectedError::InvalidResponseId(resp))
    }
    pub fn unexpected_invalid_response_id_type(resp: JsonRpcResponse) -> Self {
        Error::UnexpectedError(UnexpectedError::InvalidResponseIdType(resp))
    }
    pub fn unexpected_response_id_not_found(resp: JsonRpcResponse) -> Self {
        Error::UnexpectedError(UnexpectedError::ResponseIdNotFound(resp))
    }
    pub fn unexpected_invalid_batch_response(resps: Vec<JsonRpcResponse>) -> Self {
        Error::UnexpectedError(UnexpectedError::InvalidBatchResponse(resps))
    }
    pub fn unexpected_duplicated_response_id(resp: JsonRpcResponse) -> Self {
        Error::UnexpectedError(UnexpectedError::DuplicatedResponseId(resp))
    }
    pub fn unexpected_no_response(req: serde_json::Value) -> Self {
        Error::UnexpectedError(UnexpectedError::NoResponse(req))
    }
    pub fn unexpected_uncategorized(err: String) -> Self {
        Error::UnexpectedError(UnexpectedError::Uncategorized(err))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NetworkError(e) => Some(e),
            Error::InvalidHTTPResponse(e) => Some(e),
            Error::DeserializeResponseJsonError(e) => Some(e),
            Error::JsonRpcError(e) => Some(e),
            Error::UnexpectedError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum UnexpectedError {
    BCSError(bcs::Error),
    InvalidResponseId(JsonRpcResponse),
    InvalidResponseIdType(JsonRpcResponse),
    ResponseIdNotFound(JsonRpcResponse),
    InvalidBatchResponse(Vec<JsonRpcResponse>),
    DuplicatedResponseId(JsonRpcResponse),
    NoResponse(serde_json::Value),
    Uncategorized(String),
}

impl std::fmt::Display for UnexpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for UnexpectedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            UnexpectedError::BCSError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum WaitForTransactionError {
    // Get account transaction error
    GetTransactionError(Error),
    // Transaction hash does not match transaction hash argument
    TransactionHashMismatchError(jsonrpc::Transaction),
    // Got transaction and it's vm_status#type is not "executed" (execution success)
    TransactionExecutionFailed(jsonrpc::Transaction),
    // Wait timeout, value is waited duration.
    Timeout(std::time::Duration),
    // Transaction not found, latest known block (ledger info) timestamp is more recent
    // than expiration_time_secs argument.
    // Value is the latest known block (ledger info) timestamp.
    TransactionExpired(u64),
}

impl std::fmt::Display for WaitForTransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for WaitForTransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WaitForTransactionError::GetTransactionError(e) => Some(e),
            _ => None,
        }
    }
}
