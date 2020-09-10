// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::JsonRpcError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonRpcErrorResponse {
    pub jsonrpc: String,
    pub error: JsonRpcError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub libra_chain_id: u8,
    pub libra_ledger_version: u64,
    pub libra_ledger_timestampusec: u64,

    pub jsonrpc: String,

    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn new(
        chain_id: libra_types::chain_id::ChainId,
        libra_ledger_version: u64,
        libra_ledger_timestampusec: u64,
    ) -> Self {
        JsonRpcResponse {
            libra_chain_id: chain_id.id(),
            libra_ledger_version,
            libra_ledger_timestampusec,
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: None,
        }
    }

    pub fn new_error(
        chain_id: libra_types::chain_id::ChainId,
        libra_ledger_version: u64,
        libra_ledger_timestampusec: u64,
        error: JsonRpcError,
    ) -> Self {
        JsonRpcResponse {
            libra_chain_id: chain_id.id(),
            libra_ledger_version,
            libra_ledger_timestampusec,
            jsonrpc: "2.0".to_string(),
            id: None,
            result: None,
            error: Some(error),
        }
    }
}
