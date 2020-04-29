// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::views::{
    AccountStateWithProofView, AccountView, BlockMetadata, EventView, StateProofView,
    TransactionView,
};
use anyhow::{ensure, format_err, Error, Result};

use serde_json::Value;
use std::convert::TryFrom;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Debug)]
pub enum JsonRpcResponse {
    SubmissionResponse,
    AccountResponse(Option<AccountView>),
    StateProofResponse(StateProofView),
    AccountTransactionResponse(Option<TransactionView>),
    TransactionsResponse(Vec<TransactionView>),
    EventsResponse(Vec<EventView>),
    BlockMetadataResponse(BlockMetadata),
    AccountStateWithProofResponse(AccountStateWithProofView),
    UnknownResponse(Value),
}

impl TryFrom<(String, Value)> for JsonRpcResponse {
    type Error = Error;

    fn try_from((method, value): (String, Value)) -> Result<JsonRpcResponse> {
        match method.as_str() {
            "submit" => {
                ensure!(
                    value == Value::Null,
                    "received unexpected payload for submit: {}",
                    value
                );
                Ok(JsonRpcResponse::SubmissionResponse)
            }
            "get_account_state" => {
                let account = match value {
                    Value::Null => None,
                    _ => {
                        let account: AccountView = serde_json::from_value(value)?;
                        Some(account)
                    }
                };
                Ok(JsonRpcResponse::AccountResponse(account))
            }
            "get_events" => {
                let events: Vec<EventView> = serde_json::from_value(value)?;
                Ok(JsonRpcResponse::EventsResponse(events))
            }
            "get_metadata" => {
                let metadata: BlockMetadata = serde_json::from_value(value)?;
                Ok(JsonRpcResponse::BlockMetadataResponse(metadata))
            }
            "get_account_state_with_proof" => {
                let account_with_proof: AccountStateWithProofView = serde_json::from_value(value)?;
                Ok(JsonRpcResponse::AccountStateWithProofResponse(
                    account_with_proof,
                ))
            }
            "get_state_proof" => {
                let state_proof: StateProofView = serde_json::from_value(value)?;
                Ok(JsonRpcResponse::StateProofResponse(state_proof))
            }
            "get_account_transaction" => {
                let txn = match value {
                    Value::Null => None,
                    _ => {
                        let txn: TransactionView = serde_json::from_value(value)?;
                        Some(txn)
                    }
                };
                Ok(JsonRpcResponse::AccountTransactionResponse(txn))
            }
            "get_transactions" => {
                let txns: Vec<TransactionView> = serde_json::from_value(value)?;
                Ok(JsonRpcResponse::TransactionsResponse(txns))
            }
            _ => Ok(JsonRpcResponse::UnknownResponse(value)),
        }
    }
}

/// For JSON RPC views that are returned as part of a `JsonRpcResponse` instance, this trait
/// can be used to extract the view from a `JsonRpcResponse` instance when applicable
pub trait ResponseAsView: Sized {
    fn unexpected_response_error<T>(response: JsonRpcResponse) -> Result<T> {
        Err(format_err!("did not receive expected view: {:?}", response))
    }

    fn from_response(_response: JsonRpcResponse) -> Result<Self> {
        unimplemented!()
    }

    fn optional_from_response(_response: JsonRpcResponse) -> Result<Option<Self>> {
        unimplemented!()
    }

    fn vec_from_response(_response: JsonRpcResponse) -> Result<Vec<Self>> {
        unimplemented!()
    }
}

impl ResponseAsView for AccountView {
    fn optional_from_response(response: JsonRpcResponse) -> Result<Option<Self>> {
        if let JsonRpcResponse::AccountResponse(view) = response {
            Ok(view)
        } else {
            Self::unexpected_response_error::<Option<Self>>(response)
        }
    }
}

impl ResponseAsView for EventView {
    fn vec_from_response(response: JsonRpcResponse) -> Result<Vec<Self>> {
        if let JsonRpcResponse::EventsResponse(events) = response {
            Ok(events)
        } else {
            Self::unexpected_response_error::<Vec<Self>>(response)
        }
    }
}

impl ResponseAsView for BlockMetadata {
    fn from_response(response: JsonRpcResponse) -> Result<Self> {
        if let JsonRpcResponse::BlockMetadataResponse(metadata) = response {
            Ok(metadata)
        } else {
            Self::unexpected_response_error::<Self>(response)
        }
    }
}

impl ResponseAsView for TransactionView {
    fn optional_from_response(response: JsonRpcResponse) -> Result<Option<Self>> {
        if let JsonRpcResponse::AccountTransactionResponse(view) = response {
            Ok(view)
        } else {
            Self::unexpected_response_error::<Option<Self>>(response)
        }
    }

    fn vec_from_response(response: JsonRpcResponse) -> Result<Vec<Self>> {
        if let JsonRpcResponse::TransactionsResponse(txns) = response {
            Ok(txns)
        } else {
            Self::unexpected_response_error::<Vec<Self>>(response)
        }
    }
}

impl ResponseAsView for StateProofView {
    fn from_response(response: JsonRpcResponse) -> Result<Self> {
        if let JsonRpcResponse::StateProofResponse(view) = response {
            Ok(view)
        } else {
            Self::unexpected_response_error::<Self>(response)
        }
    }
}

impl ResponseAsView for AccountStateWithProofView {
    fn from_response(response: JsonRpcResponse) -> Result<Self> {
        if let JsonRpcResponse::AccountStateWithProofResponse(resp) = response {
            Ok(resp)
        } else {
            Self::unexpected_response_error::<Self>(response)
        }
    }
}
