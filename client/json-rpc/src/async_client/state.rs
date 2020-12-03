// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::async_client::JsonRpcResponse;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct State {
    pub chain_id: u8,
    pub version: u64,
    pub timestamp_usecs: u64,
}

impl State {
    pub fn from_response(resp: &JsonRpcResponse) -> Self {
        Self {
            chain_id: resp.diem_chain_id,
            version: resp.diem_ledger_version,
            timestamp_usecs: resp.diem_ledger_timestampusec,
        }
    }
}
