// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::instance::{Instance, ValidatorGroup};
use anyhow::{format_err, Result};
use diem_client::AccountAddress;
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    Uniform,
};
use diem_rest_client::Client as RestClient;
use diem_sdk::types::{AccountKey, LocalAccount};
use diem_types::{
    account_config::{
        diem_root_address, testnet_dd_account_address, treasury_compliance_account_address,
    },
    chain_id::ChainId,
    waypoint::Waypoint,
};
use forge::query_sequence_numbers;
use rand::prelude::*;
use reqwest::Client;

const DD_KEY: &str = "dd.key";

#[derive(Clone)]
pub struct Cluster {
    // guaranteed non-empty
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
    pub fn from_host_port(
        peers: Vec<(String, u32, Option<u32>)>,
        mint_file: &str,
        chain_id: ChainId,
        vasp: bool,
    ) -> Self {
        let http_client = Client::new();
        let instances: Vec<Instance> = peers
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
        Self {
            validator_instances: instances,
            fullnode_instances: vec![],
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair,
            waypoint: None,
            chain_id,
        }
    }

    fn get_mint_key_pair_from_file(
        mint_file: &str,
    ) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        let mint_key: Ed25519PrivateKey = generate_key::load_key(mint_file);
        KeyPair::from(mint_key)
    }

    pub fn new(
        validator_instances: Vec<Instance>,
        fullnode_instances: Vec<Instance>,
        lsr_instances: Vec<Instance>,
        vault_instances: Vec<Instance>,
        waypoint: Option<Waypoint>,
    ) -> Self {
        Self {
            validator_instances,
            fullnode_instances,
            lsr_instances,
            vault_instances,
            mint_key_pair: Self::get_mint_key_pair_from_file("/tmp/mint.key"),
            waypoint,
            chain_id: ChainId::test(),
        }
    }

    pub fn random_validator_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.validator_instances
            .choose(&mut rnd)
            .expect("random_validator_instance requires non-empty validator_instances")
            .clone()
    }

    pub fn validator_instances(&self) -> &[Instance] {
        &self.validator_instances
    }

    pub fn random_fullnode_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.fullnode_instances
            .choose(&mut rnd)
            .expect("random_full_node_instance requires non-empty fullnode_instances")
            .clone()
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

    pub fn all_instances(&self) -> impl Iterator<Item = &Instance> {
        self.validator_instances
            .iter()
            .chain(self.fullnode_instances.iter())
            .chain(self.lsr_instances.iter())
            .chain(self.vault_instances.iter())
    }

    pub fn validator_and_fullnode_instances(&self) -> impl Iterator<Item = &Instance> {
        self.validator_instances
            .iter()
            .chain(self.fullnode_instances.iter())
    }

    pub fn into_validator_instances(self) -> Vec<Instance> {
        self.validator_instances
    }

    pub fn into_fullnode_instances(self) -> Vec<Instance> {
        self.fullnode_instances
    }

    pub fn into_lsr_instances(self) -> Vec<Instance> {
        self.lsr_instances
    }

    pub fn into_vault_instances(self) -> Vec<Instance> {
        self.vault_instances
    }

    pub fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        &self.mint_key_pair
    }

    fn account_key(&self) -> AccountKey {
        AccountKey::from_private_key(self.mint_key_pair.private_key.clone())
    }

    async fn load_account_with_mint_key(
        &self,
        client: &RestClient,
        address: AccountAddress,
    ) -> Result<LocalAccount> {
        let sequence_number = query_sequence_numbers(client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for account {} failed: {}",
                    client,
                    address,
                    e
                )
            })?[0];
        Ok(LocalAccount::new(
            address,
            self.account_key(),
            sequence_number,
        ))
    }

    pub async fn load_diem_root_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, diem_root_address())
            .await
    }

    pub async fn load_faucet_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, testnet_dd_account_address())
            .await
    }

    pub async fn load_tc_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, treasury_compliance_account_address())
            .await
    }

    pub async fn load_dd_account(&self, client: &RestClient) -> Result<LocalAccount> {
        let mint_key: Ed25519PrivateKey = generate_key::load_key(DD_KEY);
        let account_key = AccountKey::from_private_key(mint_key);
        let address = account_key.authentication_key().derived_address();
        let sequence_number = query_sequence_numbers(client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        Ok(LocalAccount::new(address, account_key, sequence_number))
    }

    pub fn get_validator_instance(&self, name: &str) -> Option<&Instance> {
        self.validator_instances
            .iter()
            .find(|instance| instance.peer_name() == name)
    }

    /// Splits this cluster into two
    ///
    /// Returns tuple of two clusters:
    /// First element in tuple contains cluster with c random instances from self
    /// Second element in tuple contains cluster with remaining instances from self
    pub fn split_n_validators_random(&self, c: usize) -> (Self, Self) {
        assert!(c <= self.validator_instances.len());
        let mut rng = ThreadRng::default();
        let mut sub = vec![];
        let mut rem = self.validator_instances.clone();
        for _ in 0..c {
            let idx_remove = rng.gen_range(0..rem.len());
            let instance = rem.remove(idx_remove);
            sub.push(instance);
        }
        (
            self.new_validator_sub_cluster(sub),
            self.new_validator_sub_cluster(rem),
        )
    }

    pub fn split_n_fullnodes_random(&self, c: usize) -> (Self, Self) {
        assert!(c <= self.fullnode_instances.len());
        let mut rng = ThreadRng::default();
        let mut sub = vec![];
        let mut rem = self.fullnode_instances.clone();
        for _ in 0..c {
            let idx_remove = rng.gen_range(0..rem.len());
            let instance = rem.remove(idx_remove);
            sub.push(instance);
        }
        (
            self.new_fullnode_sub_cluster(sub),
            self.new_fullnode_sub_cluster(rem),
        )
    }

    fn new_validator_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            validator_instances: instances,
            fullnode_instances: vec![],
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair: self.mint_key_pair.clone(),
            waypoint: self.waypoint,
            chain_id: ChainId::test(),
        }
    }

    fn new_fullnode_sub_cluster(&self, instances: Vec<Instance>) -> Self {
        Cluster {
            validator_instances: vec![],
            fullnode_instances: instances,
            lsr_instances: vec![],
            vault_instances: vec![],
            mint_key_pair: self.mint_key_pair.clone(),
            waypoint: self.waypoint,
            chain_id: ChainId::test(),
        }
    }

    pub fn validator_sub_cluster(&self, ids: Vec<String>) -> Cluster {
        let mut instances = Vec::with_capacity(ids.len());
        for id in ids {
            let instance = self.get_validator_instance(&id);
            match instance {
                Some(instance) => instances.push(instance.clone()),
                None => panic!("Can not make sub_cluster: instance {} is not found", id),
            }
        }
        assert!(!instances.is_empty(), "No instances for subcluster");
        self.new_validator_sub_cluster(instances)
    }

    pub fn find_instance_by_pod(&self, pod: &str) -> Option<&Instance> {
        self.validator_and_fullnode_instances()
            .find(|i| i.peer_name() == pod)
    }

    pub fn instances_for_group(
        &self,
        validator_group: ValidatorGroup,
    ) -> impl Iterator<Item = &Instance> {
        self.all_instances()
            .filter(move |v| v.validator_group() == validator_group)
    }

    pub fn lsr_instances_for_validators(&self, validators: &[Instance]) -> Vec<Instance> {
        validators
            .iter()
            .filter_map(|l| {
                self.lsr_instances
                    .iter()
                    .find(|x| l.validator_group() == x.validator_group())
                    .cloned()
            })
            .collect()
    }

    pub fn vault_instances_for_validators(&self, validators: &[Instance]) -> Vec<Instance> {
        validators
            .iter()
            .filter_map(|v| {
                self.vault_instances
                    .iter()
                    .find(|x| v.validator_group() == x.validator_group())
                    .cloned()
            })
            .collect()
    }
}
