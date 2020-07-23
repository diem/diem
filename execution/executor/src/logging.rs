// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::StructuredLogEntry;

pub fn executor_log(entry: LogEntry) -> StructuredLogEntry {
    StructuredLogEntry::new_named("executor", entry.as_str())
}

#[derive(Clone, Copy)]
pub enum LogEntry {
    Chunk, // ChunkExecutor
    Block, // BlockExecutor
    Cache, // SpeculationCache
}

impl LogEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEntry::Chunk => "chunk_executor",
            LogEntry::Block => "block_executor",
            LogEntry::Cache => "speculation_cache",
        }
    }
}

pub const EVENT: &str = "event";
