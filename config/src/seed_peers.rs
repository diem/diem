// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{trusted_peers::TrustedPeersConfig, utils::get_available_port};
use parity_multiaddr::{Multiaddr, Protocol};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[cfg(test)]
#[path = "unit_tests/seed_peers_test.rs"]
mod seed_peers_test;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedPeersConfig {
    // All peers config. Key:a unique peer id, will be PK in future, Value: peer discovery info
    pub seed_peers: HashMap<String, Vec<Multiaddr>>,
}

impl SeedPeersConfig {
    pub fn load_config<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        let mut file =
            File::open(path).unwrap_or_else(|_| panic!("Cannot open Seed Peers file {:?}", path));
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .unwrap_or_else(|_| panic!("Error reading Seed Peers file {:?}", path));
        Self::parse(&contents)
    }

    pub fn save_config<P: AsRef<Path>>(&self, output_file: P) {
        let contents = toml::to_vec(&self).expect("Error serializing");
        let mut file = File::create(output_file).expect("Error opening file");
        file.write_all(&contents).expect("Error writing file");
    }

    fn parse(config_string: &str) -> Self {
        toml::from_str(config_string).expect("Unable to parse Config")
    }
}

impl Default for SeedPeersConfig {
    fn default() -> SeedPeersConfig {
        Self {
            seed_peers: HashMap::new(),
        }
    }
}

pub struct SeedPeersConfigHelpers {}

impl SeedPeersConfigHelpers {
    /// Creates a new SeedPeersConfig based on provided TrustedPeersConfig.
    /// Each node gets a random port, unless we have only 1 peer and the port is supplied
    pub fn get_test_config(
        trusted_peers: &TrustedPeersConfig,
        port: Option<u16>,
    ) -> SeedPeersConfig {
        Self::get_test_config_with_ipver(trusted_peers, port, true)
    }

    /// Creates a new SeedPeersConfig based on provided TrustedPeersConfig.
    /// Each node gets a random port, unless we have only 1 peer and the port is supplied
    pub fn get_test_config_with_ipver(
        trusted_peers: &TrustedPeersConfig,
        port: Option<u16>,
        ipv4: bool,
    ) -> SeedPeersConfig {
        let mut seed_peers = HashMap::new();
        // sort to have same repeatable order
        let mut peers: Vec<String> = trusted_peers
            .peers
            .clone()
            .into_iter()
            .map(|(peer_id, _)| peer_id)
            .collect();
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
            seed_peers.insert(peer_id.clone(), vec![addr]);
        }
        SeedPeersConfig { seed_peers }
    }
}
