// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::PeerId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StateSyncConfig {
    // Size of chunk to request for state synchronization
    pub chunk_limit: u64,
    // interval used for checking state synchronization progress
    pub tick_interval_ms: u64,
    // default timeout used for long polling to remote peer
    pub long_poll_timeout_ms: u64,
    // valid maximum chunk limit for sanity check
    pub max_chunk_limit: u64,
    // valid maximum timeout limit for sanity check
    pub max_timeout_ms: u64,
    // List of peers to use as upstream in state sync protocols.
    #[serde(flatten)]
    pub upstream_peers: UpstreamPeersConfig,
}

impl Default for StateSyncConfig {
    fn default() -> Self {
        Self {
            chunk_limit: 250,
            tick_interval_ms: 100,
            long_poll_timeout_ms: 30000,
            max_chunk_limit: 1000,
            max_timeout_ms: 120_000,
            upstream_peers: UpstreamPeersConfig::default(),
        }
    }
}

// This is separated to another config so that it can be written to its own file
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UpstreamPeersConfig {
    /// List of PeerIds serialized as string.
    pub upstream_peers: Vec<PeerId>,
}
