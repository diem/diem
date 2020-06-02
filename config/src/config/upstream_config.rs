// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// In general, a network ID is a PeerId that this node uses to uniquely identify a network it belongs to.
/// This is equivalent to the `peer_id` field in the NetworkConfig of this NodeConfig
pub type UpstreamNetworkId = PeerId;

#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpstreamConfig {
    // primary upstream network ids. All peers in such network are used as upstream for this node
    pub primary_networks: Vec<UpstreamNetworkId>,
    // All upstream peers of this node, across all the networks that are statically defined in this node's config
    // this is mostly meaningful in VFN networks, where there is a strict hierarchy in a network
    pub upstream_peers: HashSet<PeerNetworkId>,
    // optional fallback network id. Used to as a failover if preferred upstream peers are not available
    // TODO replace PeerId with `NetworkConfig` to contain actual info needed to build fallback_network
    pub fallback_networks: Vec<UpstreamNetworkId>,
}

impl UpstreamConfig {
    /// Determines whether a node `peer_id` in network `network_id` is an upstream peer of a node with this NodeConfig.
    pub fn is_upstream_peer(&self, peer: PeerNetworkId) -> bool {
        self.is_primary_upstream_peer(peer) || self.fallback_networks.contains(&peer.network_id())
    }

    pub fn is_primary_upstream_peer(&self, peer: PeerNetworkId) -> bool {
        self.primary_networks.contains(&peer.network_id()) || self.upstream_peers.contains(&peer)
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Identifier of a node, represented as (network_id, peer_id)
pub struct PeerNetworkId(pub UpstreamNetworkId, pub PeerId);

impl PeerNetworkId {
    pub fn network_id(&self) -> UpstreamNetworkId {
        self.0
    }

    pub fn peer_id(&self) -> PeerId {
        self.1
    }

    pub fn random() -> Self {
        Self(UpstreamNetworkId::random(), PeerId::random())
    }
}
