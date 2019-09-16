// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::{
    config::{NodeConfig, NodeConfigHelpers},
    trusted_peers::{ConfigHelpers, ConsensusPeersConfig, NetworkPeersConfig},
};
use crypto::{ed25519::*, test_utils::KeyPair};
use failure::prelude::*;
use proto_conv::IntoProtoBytes;
use rand::{Rng, SeedableRng};
use std::{fs::File, io::prelude::*, path::Path};
use vm_genesis::encode_genesis_transaction_with_validator;

pub fn gen_genesis_transaction<P: AsRef<Path>>(
    path: P,
    faucet_account_keypair: &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    consensus_peers_config: &ConsensusPeersConfig,
    network_peers_config: &NetworkPeersConfig,
) -> Result<()> {
    let transaction = encode_genesis_transaction_with_validator(
        &faucet_account_keypair.private_key,
        faucet_account_keypair.public_key.clone(),
        consensus_peers_config.get_validator_set(network_peers_config),
    );
    let mut file = File::create(path)?;
    file.write_all(&transaction.into_proto_bytes()?)?;
    Ok(())
}

/// Returns the config as well as the genesis keyapir
pub fn get_test_config() -> (NodeConfig, KeyPair<Ed25519PrivateKey, Ed25519PublicKey>) {
    // TODO: test config should be moved here instead of config crate
    let config = NodeConfigHelpers::get_single_node_test_config(true);
    // Those configs should be different on every call. We bypass the
    // costly StdRng initialization
    let mut seed_rng = rand::rngs::OsRng::new().expect("can't access OsRng");
    let seed_buf: [u8; 32] = seed_rng.gen();
    let mut rng = rand::rngs::StdRng::from_seed(seed_buf);
    let (private_key, _) = compat::generate_keypair(&mut rng);
    let keypair = KeyPair::from(private_key);

    let (_, test_consensus_peers) = ConfigHelpers::get_test_consensus_config(1, None);
    let (_, test_network_peers) =
        ConfigHelpers::get_test_network_peers_config(&test_consensus_peers, None);
    gen_genesis_transaction(
        &config.execution.genesis_file_location,
        &keypair,
        &test_consensus_peers,
        &test_network_peers,
    )
    .expect("[config] failed to create genesis transaction");
    (config, keypair)
}
