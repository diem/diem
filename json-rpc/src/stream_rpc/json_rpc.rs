// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::errors::JsonRpcError;
use diem_json_rpc_types::{request::RawJsonRpcRequest, Id, JsonRpcVersion};
use diem_types::event::EventKey;
use std::sync::Arc;
use storage_interface::DbReader;
use tokio::task::JoinHandle;

use crate::stream_rpc::{
    connection::ClientConnection,
    subscriptions::{Subscription, SubscriptionHelper},
};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamJsonRpcRequest {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    pub method_request: StreamMethodRequest,
    pub id: Id,
}

impl StreamJsonRpcRequest {
    fn finish_parsing(
        jsonrpc: serde_json::Value,
        method: serde_json::Value,
        params: serde_json::Value,
        id: Id,
    ) -> Result<Self, (JsonRpcError, Option<StreamMethod>, Option<Id>)> {
        let jsonrpc: JsonRpcVersion = serde_json::from_value(jsonrpc)
            .map_err(|_| (JsonRpcError::invalid_request(), None, Some(id.clone())))?;
        let method: StreamMethod = serde_json::from_value(method)
            .map_err(|_| (JsonRpcError::method_not_found(), None, Some(id.clone())))?;
        let method_request = StreamMethodRequest::from_value(method, params).map_err(|_| {
            (
                JsonRpcError::invalid_params(method.as_str()),
                Some(method),
                Some(id.clone()),
            )
        })?;

        Ok(StreamJsonRpcRequest {
            jsonrpc,
            method_request,
            id,
        })
    }

    pub fn method_name(&self) -> &'static str {
        self.method_request.method_name()
    }
}

impl FromStr for StreamJsonRpcRequest {
    type Err = (JsonRpcError, Option<StreamMethod>, Option<Id>);

    fn from_str(string: &str) -> Result<Self, (JsonRpcError, Option<StreamMethod>, Option<Id>)> {
        let RawJsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        } = serde_json::from_str(string)
            .map_err(|_| (JsonRpcError::invalid_format(), None, None))?;
        StreamJsonRpcRequest::finish_parsing(jsonrpc, method, params, id)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamJsonRpcResponse {
    pub jsonrpc: JsonRpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl StreamJsonRpcResponse {
    pub fn result(id: Option<Id>, result: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            result,
            error: None,
        }
    }

    pub fn error(id: Option<Id>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JsonRpcVersion::V2,
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl From<StreamJsonRpcResponse> for serde_json::Value {
    fn from(response: StreamJsonRpcResponse) -> Self {
        serde_json::to_value(&response).unwrap()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SubscribeToTransactionsParams {
    pub starting_version: u64,
    pub include_events: Option<bool>,

    #[serde(skip)]
    pub(crate) latest_version: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SubscribeToEventsParams {
    pub event_key: EventKey,
    pub event_seq_num: u64,

    #[serde(skip)]
    pub(crate) latest_event: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SubscriptionResult {
    #[serde(rename = "OK")]
    OK,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscribeResult {
    pub status: SubscriptionResult,
    pub transaction_version: u64,
}

impl SubscribeResult {
    pub fn ok(transaction_version: u64) -> Self {
        Self {
            status: SubscriptionResult::OK,
            transaction_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum StreamMethodRequest {
    SubscribeToTransactions(SubscribeToTransactionsParams),
    SubscribeToEvents(SubscribeToEventsParams),
}

impl StreamMethodRequest {
    pub fn call_method(
        self,
        db: Arc<dyn DbReader>,
        client: ClientConnection,
        jsonrpc_id: Id,
    ) -> Result<JoinHandle<()>, JsonRpcError> {
        match self {
            StreamMethodRequest::SubscribeToTransactions(params) => params.run(
                SubscriptionHelper::new(db, client, jsonrpc_id, self.method()),
            ),
            StreamMethodRequest::SubscribeToEvents(params) => params.run(SubscriptionHelper::new(
                db,
                client,
                jsonrpc_id,
                self.method(),
            )),
        }
    }

    pub fn method_name(&self) -> &'static str {
        self.method().as_str()
    }

    pub fn from_value(
        method: StreamMethod,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let method_request = match method {
            StreamMethod::SubscribeToTransactions => {
                StreamMethodRequest::SubscribeToTransactions(serde_json::from_value(value)?)
            }
            StreamMethod::SubscribeToEvents => {
                StreamMethodRequest::SubscribeToEvents(serde_json::from_value(value)?)
            }
        };

        Ok(method_request)
    }

    pub fn method(&self) -> StreamMethod {
        match self {
            StreamMethodRequest::SubscribeToTransactions(_) => {
                StreamMethod::SubscribeToTransactions
            }
            StreamMethodRequest::SubscribeToEvents(_) => StreamMethod::SubscribeToEvents,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMethod {
    SubscribeToTransactions,
    SubscribeToEvents,
}

impl StreamMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamMethod::SubscribeToTransactions => "subscribe_to_transactions",
            StreamMethod::SubscribeToEvents => "subscribe_to_events",
        }
    }
}
