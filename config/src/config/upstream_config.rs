// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_types::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpstreamConfig {
    // The upstream networks that this node belongs to as statically defined in the NodeConfig of this UpstreamConfig
    // A network is considered upstream if all peers in this network are upstream w.r.t. this node
    pub primary_networks: HashSet<PeerId>,
    // All upstream peers of this node, across all the networks that are statically defined in this node's config
    // this is mostly meaningful in VFNs, where there is a strict hierarchy in a network
    pub upstream_peers: HashSet<PeerNetworkId>,
    // ID of network that connects to VFNs
    // used to connect to more upstream peers if upstream peers in the primary_network
    // or `upstream_peers` are not available
    // TODO replace PeerId with `NetworkConfig` to contain actual info needed to build fallback_network
    pub fallback_network: Option<PeerId>,
}

impl UpstreamConfig {
    pub fn is_upstream_peer(&self, network_id: PeerId, peer_id: PeerId) -> bool {
        self.primary_networks.contains(&network_id)
            || self
                .upstream_peers
                .contains(&PeerNetworkId(network_id, peer_id))
            || self.fallback_network == Some(peer_id)
    }
}

impl Default for UpstreamConfig {
    fn default() -> UpstreamConfig {
        UpstreamConfig {
            primary_networks: HashSet::new(),
            upstream_peers: HashSet::new(),
            fallback_network: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Identifier of a node, represented as (network_id, peer_id)
/// In general, a network ID is a PeerId that this node uses to uniquely identify a network it belongs to.
/// This is equivalent to the `peer_id` field in the NetworkConfig of this NodeConfig
/// Here, `network_id` is the ID of the network that the peer and this node belong to.
pub struct PeerNetworkId(pub PeerId, pub PeerId);

impl PeerNetworkId {
    pub fn network_id(&self) -> PeerId {
        self.0
    }

    pub fn peer_id(&self) -> PeerId {
        self.1
    }
}
