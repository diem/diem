// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use async_trait::async_trait;

pub mod k8s_node;
pub mod k8s_swarm;
pub mod local_node;
pub mod local_swarm;

use diem_crypto::test_utils::KeyPair;
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use diem_types::waypoint::Waypoint;
use reqwest::{Client, Url};
use diem_types::chain_id::ChainId;
use std::fmt;
use std::str::FromStr;
use diem_client::Client as JsonRpcClient;
use diem_crypto::Uniform;

#[derive(Clone)]
enum InstanceBackend {
    Swarm,
    K8s,
}

#[derive(Clone)]
pub struct Cluster {
    validator_instances: Vec<Instance>,
    fullnode_instances: Vec<Instance>,
    lsr_instances: Vec<Instance>,
    vault_instances: Vec<Instance>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    waypoint: Option<Waypoint>,
    pub chain_id: ChainId,
}

pub fn dummy_key_pair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    Ed25519PrivateKey::generate_for_testing().into()
}

impl Cluster {
    pub fn new(
        peers: Vec<(String, u32, Option<u32>)>,
        mint_file: &str,
        chain_id: ChainId,
        vasp: bool,
    ) -> Cluster {
        let http_client = Client::new();
        let nodes: Vec<Instance> = peers
            .into_iter()
            .map(|host_port| {
                Instance::new(
                    format!("{}:{}", &host_port.0, host_port.1), /* short_hash */
                    host_port.0,
                    host_port.1,
                    host_port.2,
                    http_client.clone(),
                )
            })
            .collect();

        let mint_key_pair = if vasp {
            dummy_key_pair()
        } else {
            KeyPair::from(generate_key::load_key(mint_file))
        };
        Cluster {
            validator_instances: nodes,
            fullnode_instances: vec![],
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair,
            waypoint: None,
            chain_id,
        }
    }

    pub fn validator_instances(&self) -> &[Instance] {
        &self.validator_instances
    }

    pub fn fullnode_instances(&self) -> &[Instance] {
        &self.fullnode_instances
    }

    pub fn lsr_instances(&self) -> &[Instance] {
        &self.lsr_instances
    }

    pub fn vault_instances(&self) -> &[Instance] {
        &self.vault_instances
    }

    pub fn waypoint(&self) -> Option<Waypoint> {
        self.waypoint
    }

    pub fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        &self.mint_key_pair
    }

    pub fn validator_and_fullnode_instances(&self) -> impl Iterator<Item = &Instance> {
        self.validator_instances
            .iter()
            .chain(self.fullnode_instances.iter())
    }
}

#[derive(Clone)]
pub struct Instance {
    peer_name: String,
    ip: String,
    json_rpc_port: u32,
    debug_interface_port: Option<u32>,
    http_client: Client,
    backend: InstanceBackend,
}

impl Instance {
    pub fn new(
        peer_name: String,
        ip: String,
        json_rpc_port: u32,
        debug_interface_port: Option<u32>,
        http_client: Client,
    ) -> Instance {
        let backend = InstanceBackend::Swarm;
        Instance {
            peer_name,
            ip,
            json_rpc_port,
            debug_interface_port,
            http_client,
            backend,
        }
    }

    pub fn peer_name(&self) -> &str {
        &self.peer_name
    }

    pub fn debug_interface_port(&self) -> Option<u32> {
        self.debug_interface_port
    }

    pub fn http_clinet(&self) -> Client {
        self.http_client.clone()
    }

    pub fn ip(&self) -> &str {
        &self.ip
    }

    pub fn json_rpc_port(&self) -> u32 {
        self.json_rpc_port
    }

    pub fn json_rpc_url(&self) -> Url {
        Url::from_str(&format!("http://{}:{}/v1", self.ip(), self.json_rpc_port())).expect("Invalid URL.")
    }

    pub fn json_rpc_client(&self) -> JsonRpcClient {
        JsonRpcClient::new(self.json_rpc_url().to_string())
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({})", self.peer_name, self.ip)
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[async_trait]
pub trait Swarm {
}

#[async_trait]
pub trait Node {
}
