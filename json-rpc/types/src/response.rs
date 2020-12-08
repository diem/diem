// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::errors::JsonRpcError;
use serde::{Deserialize, Serialize};

// http response header names
pub const X_DIEM_CHAIN_ID: &str = "X-Diem-Chain-Id";
pub const X_DIEM_VERSION_ID: &str = "X-Diem-Ledger-Version";
pub const X_DIEM_TIMESTAMP_USEC_ID: &str = "X-Diem-Ledger-TimestampUsec";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub diem_chain_id: u8,
    pub diem_ledger_version: u64,
    pub diem_ledger_timestampusec: u64,

    pub jsonrpc: String,

    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn new(
        chain_id: diem_types::chain_id::ChainId,
        diem_ledger_version: u64,
        diem_ledger_timestampusec: u64,
    ) -> Self {
        Self {
            diem_chain_id: chain_id.id(),
            diem_ledger_version,
            diem_ledger_timestampusec,
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
    use diem_types::chain_id::ChainId;

    #[test]
    fn test_new() {
        let resp = JsonRpcResponse::new(ChainId::test(), 1, 2);
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.diem_chain_id, 4);
        assert_eq!(resp.diem_ledger_version, 1);
        assert_eq!(resp.diem_ledger_timestampusec, 2);
        assert!(resp.id.is_none());
        assert!(resp.result.is_none());
        assert!(resp.error.is_none());
    }
}
