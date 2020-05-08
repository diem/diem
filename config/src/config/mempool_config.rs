// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MempoolConfig {
    pub broadcast_transactions: bool,
    pub shared_mempool_tick_interval_ms: u64,
    pub shared_mempool_batch_size: usize,
    pub shared_mempool_max_concurrent_inbound_syncs: usize,
    pub shared_mempool_min_broadcast_recipient_count: Option<usize>,
    pub max_broadcasts_per_peer: usize,
    pub capacity: usize,
    // max number of transactions per user in Mempool
    pub capacity_per_user: usize,
    pub system_transaction_timeout_secs: u64,
    pub system_transaction_gc_interval_ms: u64,
}

impl Default for MempoolConfig {
    fn default() -> MempoolConfig {
        MempoolConfig {
            broadcast_transactions: true,
            shared_mempool_tick_interval_ms: 50,
            shared_mempool_batch_size: 100,
            shared_mempool_max_concurrent_inbound_syncs: 100,
            shared_mempool_min_broadcast_recipient_count: None,
            max_broadcasts_per_peer: 25,
            capacity: 1_000_000,
            capacity_per_user: 1_000_000,
            system_transaction_timeout_secs: 86400,
            system_transaction_gc_interval_ms: 180_000,
        }
    }
}
