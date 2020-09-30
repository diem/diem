// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::JsonRpcError;
use serde::{Deserialize, Serialize};

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
}

#[cfg(test)]
mod tests {
    use crate::response::JsonRpcResponse;
    use libra_types::chain_id::ChainId;

    #[test]
    fn test_new() {
        let resp = JsonRpcResponse::new(ChainId::test(), 1, 2);
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.libra_chain_id, 4);
        assert_eq!(resp.libra_ledger_version, 1);
        assert_eq!(resp.libra_ledger_timestampusec, 2);
        assert!(resp.id.is_none());
        assert!(resp.result.is_none());
        assert!(resp.error.is_none());
    }
}
