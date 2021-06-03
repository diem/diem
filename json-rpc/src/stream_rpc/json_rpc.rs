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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: JsonRpcVersion,
    #[serde(flatten)]
    pub method_request: MethodRequest,
    pub id: Id,
}

impl JsonRpcRequest {
    pub fn from_value(
        value: serde_json::Value,
    ) -> Result<Self, (JsonRpcError, Option<Method>, Option<Id>)> {
        let RawJsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        } = serde_json::from_value(value)
            .map_err(|_| (JsonRpcError::invalid_format(), None, None))?;
        JsonRpcRequest::finish_parsing(jsonrpc, method, params, id)
    }

    fn finish_parsing(
        jsonrpc: serde_json::Value,
        method: serde_json::Value,
        params: serde_json::Value,
        id: Id,
    ) -> Result<Self, (JsonRpcError, Option<Method>, Option<Id>)> {
        let jsonrpc: JsonRpcVersion = serde_json::from_value(jsonrpc)
            .map_err(|_| (JsonRpcError::invalid_request(), None, Some(id.clone())))?;
        let method: Method = serde_json::from_value(method)
            .map_err(|_| (JsonRpcError::method_not_found(), None, Some(id.clone())))?;
        let method_request = MethodRequest::from_value(method, params).map_err(|_| {
            (
                JsonRpcError::invalid_params(method.as_str()),
                Some(method),
                Some(id.clone()),
            )
        })?;

        Ok(JsonRpcRequest {
            jsonrpc,
            method_request,
            id,
        })
    }

    pub fn call_method(
        &self,
        db: Arc<dyn DbReader>,
        client: ClientConnection,
    ) -> Result<JoinHandle<()>, JsonRpcError> {
        self.method_request.call_method(db, client, self.id.clone())
    }

    pub fn method_name(&self) -> &'static str {
        self.method_request.method_name()
    }
}

impl FromStr for JsonRpcRequest {
    type Err = (JsonRpcError, Option<Method>, Option<Id>);

    fn from_str(string: &str) -> Result<Self, (JsonRpcError, Option<Method>, Option<Id>)> {
        let RawJsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        } = serde_json::from_str(string)
            .map_err(|_| (JsonRpcError::invalid_format(), None, None))?;
        JsonRpcRequest::finish_parsing(jsonrpc, method, params, id)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: JsonRpcVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
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

impl From<JsonRpcResponse> for serde_json::Value {
    fn from(response: JsonRpcResponse) -> Self {
        serde_json::to_value(&response).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct SubscribeToTransactionsParams {
    pub starting_version: u64,
    pub include_events: Option<bool>,

    #[serde(skip)]
    pub(crate) latest_version: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct SubscribeToEventsParams {
    pub event_key: EventKey,
    pub event_seq_num: u64,

    #[serde(skip)]
    pub(crate) latest_event: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SubscriptionResult {
    #[serde(rename = "OK")]
    OK,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "method", content = "params")]
pub enum MethodRequest {
    SubscribeToTransactions(SubscribeToTransactionsParams),
    SubscribeToEvents(SubscribeToEventsParams),
}

impl MethodRequest {
    pub fn call_method(
        self,
        db: Arc<dyn DbReader>,
        client: ClientConnection,
        jsonrpc_id: Id,
    ) -> Result<JoinHandle<()>, JsonRpcError> {
        match self {
            MethodRequest::SubscribeToTransactions(params) => params.run(SubscriptionHelper::new(
                db,
                client,
                jsonrpc_id,
                self.method(),
            )),
            MethodRequest::SubscribeToEvents(params) => params.run(SubscriptionHelper::new(
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

    pub fn from_value(method: Method, value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let method_request = match method {
            Method::SubscribeToTransactions => {
                MethodRequest::SubscribeToTransactions(serde_json::from_value(value)?)
            }
            Method::SubscribeToEvents => {
                MethodRequest::SubscribeToEvents(serde_json::from_value(value)?)
            }
        };

        Ok(method_request)
    }

    pub fn method(&self) -> Method {
        match self {
            MethodRequest::SubscribeToTransactions(_) => Method::SubscribeToTransactions,
            MethodRequest::SubscribeToEvents(_) => Method::SubscribeToEvents,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    SubscribeToTransactions,
    SubscribeToEvents,
}

impl Method {
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::SubscribeToTransactions => "subscribe_to_transactions",
            Method::SubscribeToEvents => "subscribe_to_events",
        }
    }
}
