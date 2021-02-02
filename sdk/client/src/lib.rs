// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod error;
pub use error::{Error, Result, WaitForTransactionError};

#[cfg(feature = "blocking")]
mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::BlockingClient;

#[cfg(feature = "async")]
mod client;
#[cfg(feature = "async")]
pub use client::Client;

mod request;
pub use request::{JsonRpcRequest, MethodRequest};

mod response;
pub use response::{MethodResponse, Response};

mod state;
pub use state::State;

mod retry;
pub use retry::Retry;

pub use diem_json_rpc_types::{errors, views};
pub use diem_types::{account_address::AccountAddress, transaction::SignedTransaction};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
enum JsonRpcVersion {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Submit,
    GetMetadata,
    GetAccount,
    GetTransactions,
    GetAccountTransaction,
    GetAccountTransactions,
    GetEvents,
    GetCurrencies,
    GetNetworkStatus,

    //
    // Experimental APIs
    //
    GetStateProof,
    GetAccountStateWithProof,
    GetTransactionsWithProofs,
    GetEventsWithProofs,
}

#[cfg(any(feature = "async", feature = "blocking"))]
fn validate(
    state_manager: &state::StateManager,
    resp: &diem_json_rpc_types::response::JsonRpcResponse,
) -> Result<(u64, State, serde_json::Value)> {
    if resp.jsonrpc != "2.0" {
        return Err(Error::rpc_response(format!(
            "unsupported jsonrpc version {}",
            resp.jsonrpc
        )));
    }
    let id = get_id(resp)?;

    if let Some(err) = &resp.error {
        return Err(Error::json_rpc(err.clone()));
    }

    let state = State::from_response(resp);
    state_manager.update_state(&state)?;

    // Result being empty is an acceptable response
    let result = resp.result.clone().unwrap_or(serde_json::Value::Null);

    Ok((id, state, result))
}

#[cfg(any(feature = "async", feature = "blocking"))]
fn validate_batch(
    state_manager: &state::StateManager,
    requests: &[JsonRpcRequest],
    raw_responses: Vec<diem_json_rpc_types::response::JsonRpcResponse>,
) -> Result<Vec<Result<Response<MethodResponse>>>> {
    let mut responses = std::collections::HashMap::new();
    for raw_response in &raw_responses {
        let id = get_id(&raw_response)?;
        let response = validate(state_manager, &raw_response);

        responses.insert(id, response);
    }

    let mut result = Vec::new();

    for request in requests {
        let response = if let Some(response) = responses.remove(&request.id()) {
            response
        } else {
            return Err(Error::batch(format!("{:?}", raw_responses)));
        };

        let response = response.and_then(|(_id, state, result)| {
            MethodResponse::from_json(request.method(), result)
                .map(|result| Response::new(result, state))
        });

        result.push(response);
    }

    if !responses.is_empty() {
        return Err(Error::batch(format!("{:?}", raw_responses)));
    }

    Ok(result)
}

#[cfg(any(feature = "async", feature = "blocking"))]
fn get_id(resp: &diem_json_rpc_types::response::JsonRpcResponse) -> Result<u64> {
    let id = if let Some(id) = &resp.id {
        if let Ok(index) = serde_json::from_value::<u64>(id.clone()) {
            index
        } else {
            return Err(Error::rpc_response("invalid response id type"));
        }
    } else {
        return Err(Error::rpc_response("missing response id"));
    };

    Ok(id)
}

#[cfg(any(feature = "async", feature = "blocking"))]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum BatchResponse {
    Success(Vec<diem_json_rpc_types::response::JsonRpcResponse>),
    Error(Box<diem_json_rpc_types::response::JsonRpcResponse>),
}

#[cfg(any(feature = "async", feature = "blocking"))]
impl BatchResponse {
    pub fn success(self) -> Result<Vec<diem_json_rpc_types::response::JsonRpcResponse>> {
        match self {
            BatchResponse::Success(inner) => Ok(inner),
            BatchResponse::Error(e) => Err(Error::json_rpc(e.error.unwrap())),
        }
    }
}
