// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{trusted_peers::NetworkPeersConfig, utils::get_available_port};
use libra_types::PeerId;
use parity_multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct SeedPeersConfig {
    // All peers config. Key:a unique peer id, will be PK in future, Value: peer discovery info
    pub seed_peers: HashMap<PeerId, Vec<Multiaddr>>,
}

pub struct SeedPeersConfigHelpers {}

impl SeedPeersConfigHelpers {
    /// Creates a new SeedPeersConfig based on provided NetworkPeersConfig.
    /// Each node gets a random port, unless we have only 1 peer and the port is supplied
    pub fn get_test_config(
        network_peers: &NetworkPeersConfig,
        port: Option<u16>,
    ) -> SeedPeersConfig {
        Self::get_test_config_with_ipver(network_peers, port, true)
    }

    /// Creates a new SeedPeersConfig based on provided NetworkPeersConfig.
    /// Each node gets a random port, unless we have only 1 peer and the port is supplied
    pub fn get_test_config_with_ipver(
        network_peers: &NetworkPeersConfig,
        port: Option<u16>,
        ipv4: bool,
    ) -> SeedPeersConfig {
        let mut seed_peers = HashMap::new();
        // sort to have same repeatable order
        let mut peers: Vec<_> = network_peers.peers.keys().cloned().collect();
        peers.sort_unstable_by_key(std::clone::Clone::clone);
        // If a port is supplied, we should have only 1 peer.
        if port.is_some() {
            assert_eq!(1, peers.len());
        }
        for peer_id in peers {
            // Create a new PeerInfo and increment the ports
            let mut addr = Multiaddr::empty();
            if ipv4 {
                addr.push(Protocol::Ip4("0.0.0.0".parse().unwrap()));
            } else {
                addr.push(Protocol::Ip6("::1".parse().unwrap()));
            }
            addr.push(Protocol::Tcp(port.unwrap_or_else(get_available_port)));
            seed_peers.insert(peer_id, vec![addr]);
        }
        SeedPeersConfig { seed_peers }
    }
}

#[cfg(test)]
mod test {
    use super::SeedPeersConfigHelpers;
    use crate::trusted_peers::ConfigHelpers;

    #[test]
    fn generate_test_config() {
        let (_, _, network_peers_config) = ConfigHelpers::gen_validator_nodes(10, None);
        let _ = SeedPeersConfigHelpers::get_test_config(&network_peers_config, None);
    }
}
