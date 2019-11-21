// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_config::{
    config::NodeConfig,
    trusted_peers::{ConsensusPeersConfig, NetworkPeersConfig},
};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    Uniform,
};
use libra_types::transaction::Transaction;
use rand::{rngs::StdRng, SeedableRng};
use std::process;
use vm_genesis;

pub fn genesis_transaction(
    faucet_account_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    consensus_peers_config: &ConsensusPeersConfig,
    network_peers_config: &NetworkPeersConfig,
) -> Transaction {
    let genesis_transaction = vm_genesis::encode_genesis_transaction_with_validator(
        &faucet_account_keypair.private_key,
        faucet_account_keypair.public_key.clone(),
        consensus_peers_config.get_validator_set(network_peers_config),
    );
    Transaction::UserTransaction(genesis_transaction.into_inner())
}

/// Returns the config as well as the genesis keypair
pub fn get_test_config() -> (NodeConfig, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>) {
    let mut config = NodeConfig::random();
    config.randomize_ports();

    // Get a deterministic seed per process, as many tests expect the same genesis keys
    let base_seed = process::id().to_be_bytes();
    let mut seed = [0u8; 32];
    seed[..base_seed.len()].clone_from_slice(&base_seed[..]);
    let mut rng = StdRng::from_seed(seed);

    let private_key = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let keypair = KeyPair::from(private_key);
    config.execution.genesis = Some(genesis_transaction(
        &keypair,
        &config.consensus.consensus_peers,
        &config.validator_network.as_ref().unwrap().network_peers,
    ));
    (config, keypair)
}
