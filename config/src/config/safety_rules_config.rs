// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{LoggerConfig, SecureBackend},
    keys::ConfigKey,
};
use libra_crypto::{ed25519::Ed25519PrivateKey, Uniform};
use libra_network_address::NetworkAddress;
use libra_types::{waypoint::Waypoint, PeerId};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SafetyRulesConfig {
    pub backend: SecureBackend,
    pub logger: LoggerConfig,
    pub service: SafetyRulesService,
    pub test: Option<SafetyRulesTestConfig>,
    pub verify_vote_proposal_signature: bool,
}

impl Default for SafetyRulesConfig {
    fn default() -> Self {
        Self {
            backend: SecureBackend::InMemoryStorage,
            logger: LoggerConfig::default(),
            service: SafetyRulesService::Thread,
            test: None,
            verify_vote_proposal_signature: true,
        }
    }
}

impl SafetyRulesConfig {
    pub fn set_data_dir(&mut self, data_dir: PathBuf) {
        if let SecureBackend::OnDiskStorage(backend) = &mut self.backend {
            backend.set_data_dir(data_dir);
        }
    }
}

/// Defines how safety rules should be executed
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SafetyRulesService {
    /// This runs safety rules in the same thread as event processor
    Local,
    /// This is the production, separate service approach
    Process(RemoteService),
    /// This runs safety rules in the same thread as event processor but data is passed through the
    /// light weight RPC (serializer)
    Serializer,
    /// This instructs Consensus that this is an test model, where Consensus should take the
    /// existing config, create a new process, and pass it the config
    SpawnedProcess(RemoteService),
    /// This creates a separate thread to run safety rules, it is similar to a fork / exec style
    Thread,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteService {
    pub server_address: NetworkAddress,
}

impl RemoteService {
    pub fn server_address(&self) -> SocketAddr {
        self.server_address
            .to_socket_addrs()
            .expect("server_address invalid")
            .next()
            .expect("server_address invalid")
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SafetyRulesTestConfig {
    pub author: PeerId,
    pub consensus_key: Option<ConfigKey<Ed25519PrivateKey>>,
    pub execution_key: Option<ConfigKey<Ed25519PrivateKey>>,
    pub waypoint: Option<Waypoint>,
}

impl SafetyRulesTestConfig {
    pub fn new(author: PeerId) -> Self {
        Self {
            author,
            consensus_key: None,
            execution_key: None,
            waypoint: None,
        }
    }

    pub fn random_consensus_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.consensus_key = Some(ConfigKey::<Ed25519PrivateKey>::new(privkey));
    }

    pub fn random_execution_key(&mut self, rng: &mut StdRng) {
        let privkey = Ed25519PrivateKey::generate(rng);
        self.execution_key = Some(ConfigKey::<Ed25519PrivateKey>::new(privkey));
    }
}
