// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::PersistableConfig, keys::NetworkKeyPairs, seed_peers::SeedPeersConfig,
    trusted_peers::NetworkPeersConfig, utils::get_local_ip,
};
use failure::prelude::*;
use libra_crypto::ValidKey;
use libra_types::PeerId;
use parity_multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt,
    path::{Path, PathBuf},
    string::ToString,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    Validator,
    FullNode,
}

impl<T> std::convert::From<T> for RoleType
where
    T: AsRef<str>,
{
    fn from(t: T) -> RoleType {
        match t.as_ref() {
            "validator" => RoleType::Validator,
            "full_node" => RoleType::FullNode,
            _ => unimplemented!("Invalid node role: {}", t.as_ref()),
        }
    }
}

impl fmt::Display for RoleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RoleType::Validator => write!(f, "validator"),
            RoleType::FullNode => write!(f, "full_node"),
        }
    }
}

#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub peer_id: String,
    // TODO: Add support for multiple listen/advertised addresses in config.
    // The address that this node is listening on for new connections.
    pub listen_address: Multiaddr,
    // The address that this node advertises to other nodes for the discovery protocol.
    pub advertised_address: Multiaddr,
    pub discovery_interval_ms: u64,
    pub connectivity_check_interval_ms: u64,
    // Flag to toggle if Noise is used for encryption and authentication.
    pub enable_encryption_and_authentication: bool,
    // If the network is permissioned, only trusted peers are allowed to connect. Otherwise, any
    // node can connect. If this flag is set to true, the `enable_encryption_and_authentication`
    // must also be set to true.
    pub is_permissioned: bool,
    // The role of the node in the network. One of: {"validator", "full_node"}.
    pub role: RoleType,
    // network_keypairs contains the node's network keypairs.
    // it is filled later on from network_keypairs_file.
    #[serde(skip)]
    pub network_keypairs: NetworkKeyPairs,
    pub network_keypairs_file: PathBuf,
    // network peers are the nodes allowed to connect when the network is started in permissioned
    // mode.
    #[serde(skip)]
    pub network_peers: NetworkPeersConfig,
    pub network_peers_file: PathBuf,
    // seed_peers act as seed nodes for the discovery protocol.
    #[serde(skip)]
    pub seed_peers: SeedPeersConfig,
    pub seed_peers_file: PathBuf,
}

impl Default for NetworkConfig {
    fn default() -> NetworkConfig {
        NetworkConfig {
            peer_id: "".to_string(),
            role: RoleType::Validator,
            listen_address: "/ip4/0.0.0.0/tcp/6180".parse::<Multiaddr>().unwrap(),
            advertised_address: "/ip4/127.0.0.1/tcp/6180".parse::<Multiaddr>().unwrap(),
            discovery_interval_ms: 1000,
            connectivity_check_interval_ms: 5000,
            enable_encryption_and_authentication: true,
            is_permissioned: true,
            network_keypairs_file: PathBuf::from("network_keypairs.config.toml"),
            network_keypairs: NetworkKeyPairs::default(),
            network_peers_file: PathBuf::from("network_peers.config.toml"),
            network_peers: NetworkPeersConfig::default(),
            seed_peers_file: PathBuf::from("seed_peers.config.toml"),
            seed_peers: SeedPeersConfig::default(),
        }
    }
}

impl NetworkConfig {
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if !self.network_peers_file.as_os_str().is_empty() {
            self.network_peers = NetworkPeersConfig::load_config(
                path.as_ref().with_file_name(&self.network_peers_file),
            );
        }
        if !self.network_keypairs_file.as_os_str().is_empty() {
            self.network_keypairs = NetworkKeyPairs::load_config(
                path.as_ref().with_file_name(&self.network_keypairs_file),
            );
        }
        if !self.seed_peers_file.as_os_str().is_empty() {
            self.seed_peers =
                SeedPeersConfig::load_config(path.as_ref().with_file_name(&self.seed_peers_file));
        }
        if self.advertised_address.to_string().is_empty() {
            self.advertised_address =
                get_local_ip().ok_or_else(|| ::failure::err_msg("No local IP"))?;
        }
        if self.listen_address.to_string().is_empty() {
            self.listen_address =
                get_local_ip().ok_or_else(|| ::failure::err_msg("No local IP"))?;
        }
        // If PeerId is not set, it is derived from NetworkIdentityKey.
        if self.peer_id == "" {
            self.peer_id = PeerId::try_from(
                self.network_keypairs
                    .get_network_identity_public()
                    .to_bytes(),
            )
            .unwrap()
            .to_string();
        }
        Ok(())
    }
}
