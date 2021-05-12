// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StateSyncConfig {
    // Size of chunk to request for state synchronization
    pub chunk_limit: u64,
    // The timeout of the state sync client to process a commit notification (in milliseconds)
    pub client_commit_timeout_ms: u64,
    // default timeout used for long polling to remote peer
    pub long_poll_timeout_ms: u64,
    // valid maximum chunk limit for sanity check
    pub max_chunk_limit: u64,
    // valid maximum timeout limit for sanity check
    pub max_timeout_ms: u64,
    // The timeout of the state sync coordinator to receive a commit ack from mempool (in milliseconds)
    pub mempool_commit_timeout_ms: u64,
    // default timeout to make state sync progress by sending chunk requests to a certain number of networks
    // if no progress is made by sending chunk requests to a number of networks,
    // the next sync request will be multicasted, i.e. sent to more networks
    pub multicast_timeout_ms: u64,
    // The timeout for ensuring sync requests are making progress (i.e., the maximum time between
    // commits when processing a sync request).
    pub sync_request_timeout_ms: u64,
    // interval used for checking state synchronization progress
    pub tick_interval_ms: u64,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            chunk_limit: 3000,
            client_commit_timeout_ms: 5_000,
            long_poll_timeout_ms: 10_000,
            max_chunk_limit: 3000,
            max_timeout_ms: 120_000,
            mempool_commit_timeout_ms: 5_000,
            multicast_timeout_ms: 30_000,
            sync_request_timeout_ms: 60_000,
            tick_interval_ms: 100,
        }
    }
}
