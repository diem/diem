// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{errors::JsonRpcError, Id, JsonRpcVersion};
use serde::{Deserialize, Serialize};

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
