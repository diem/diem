// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::HashValue;
use diem_logger::Schema;
use serde::Serialize;

#[derive(Schema)]
pub struct LogSchema {
    name: LogEntry,
    block_id: Option<HashValue>,
    root_block_id: Option<HashValue>,
    original_reconfiguration_block_id: Option<HashValue>,
    num: Option<u64>,
    local_synced_version: Option<u64>,
    first_version_in_request: Option<Option<u64>>,
    num_txns_in_request: Option<usize>,
    synced_to_version: Option<u64>,
    committed_with_ledger_info: Option<bool>,
    latest_synced_version: Option<u64>,
    first_version_to_keep: Option<u64>,
    num_txns_to_keep: Option<u64>,
    first_version_to_commit: Option<u64>,
}

impl LogSchema {
    pub fn new(name: LogEntry) -> Self {
        Self {
            name,
            block_id: None,
            root_block_id: None,
            original_reconfiguration_block_id: None,
            num: None,
            local_synced_version: None,
            first_version_in_request: None,
            num_txns_in_request: None,
            synced_to_version: None,
            committed_with_ledger_info: None,
            latest_synced_version: None,
            first_version_to_keep: None,
            num_txns_to_keep: None,
            first_version_to_commit: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    ChunkExecutor,
    BlockExecutor,
    SpeculationCache,
}
