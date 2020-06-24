// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::network_id::NetworkId;
use libra_types::PeerId;
use serde::{Deserialize, Serialize};

/// If a node considers a network 'upstream', the node will broadcast transactions (via mempool) to and
/// send sync requests (via state sync) to all its peers in this network.
/// For validators, it is unnecessary to declare their validator network as their upstream network in this config
/// Otherwise, any non-validator network not declared here will be treated as a downstream
/// network (i.e. transactions will not be broadcast to and sync requests will not be sent to such networks)
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct UpstreamConfig {
    // list of upstream networks for this node, ordered by preference
    // A validator's primary upstream network is their validator network, and for a FN,
    // it is the first network defined here. If the primary upstream network goes down, the node will fall back to the networks
    // specified here, in this order
    pub networks: Vec<NetworkId>,
}

impl UpstreamConfig {
    /// Returns the upstream network preference of a network according to this config
    /// if network is not an upstream network, returns `None`
    /// else, returns `Some<ranking>`, where `ranking` is zero-indexed and zero represents the highest preference
    pub fn get_upstream_preference(&self, network: NetworkId) -> Option<usize> {
        if network == NetworkId::Validator {
            // validator network is always highest priority
            Some(0)
        } else {
            self.networks
                .iter()
                .position(|upstream_network| upstream_network == &network)
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
/// Identifier of a node, represented as (network_id, peer_id)
pub struct PeerNetworkId(pub NetworkId, pub PeerId);

impl PeerNetworkId {
    pub fn network_id(&self) -> NetworkId {
        self.0.clone()
    }

    pub fn peer_id(&self) -> PeerId {
        self.1
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn random() -> Self {
        Self(NetworkId::default(), PeerId::random())
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn random_validator() -> Self {
        Self(NetworkId::Validator, PeerId::random())
    }
}
