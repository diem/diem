// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MempoolConfig {
    pub capacity: usize,
    pub capacity_per_user: usize,
    // number of failovers to broadcast to when the primary network is alive
    pub default_failovers: usize,
    pub max_broadcasts_per_peer: usize,
    pub mempool_snapshot_interval_secs: u64,
    pub shared_mempool_ack_timeout_ms: u64,
    pub shared_mempool_backoff_interval_ms: u64,
    pub shared_mempool_batch_size: usize,
    pub shared_mempool_max_concurrent_inbound_syncs: usize,
    pub shared_mempool_tick_interval_ms: u64,
    pub system_transaction_timeout_secs: u64,
    pub system_transaction_gc_interval_ms: u64,
}

impl Default for MempoolConfig {
    fn default() -> MempoolConfig {
        MempoolConfig {
            shared_mempool_tick_interval_ms: 50,
            shared_mempool_backoff_interval_ms: 30_000,
            shared_mempool_batch_size: 100,
            shared_mempool_ack_timeout_ms: 2_000,
            shared_mempool_max_concurrent_inbound_syncs: 2,
            max_broadcasts_per_peer: 1,
            mempool_snapshot_interval_secs: 180,
            capacity: 1_000_000,
            capacity_per_user: 100,
            default_failovers: 3,
            system_transaction_timeout_secs: 600,
            system_transaction_gc_interval_ms: 60_000,
        }
    }
}
