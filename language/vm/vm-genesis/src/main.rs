// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::{config::PersistableConfig, generator};
use libra_crypto::HashValue;
use libra_types::transaction::Transaction;
use std::{fs::File, io::prelude::*};
use transaction_builder::default_config;
use vm_genesis::{
    encode_genesis_transaction_with_validator, make_placeholder_discovery_set, GENESIS_KEYPAIR,
};

const CONFIG_LOCATION: &str = "genesis/vm_config.toml";
const GENESIS_LOCATION: &str = "genesis/genesis.blob";

/// Generate the genesis blob used by the Libra blockchain
fn generate_genesis_blob() -> (Vec<u8>, HashValue) {
    let configs = generator::validator_swarm_for_testing(10);
    let consensus_peers = &configs[0].consensus.consensus_peers;
    let network_peers = &configs[0].validator_network.as_ref().unwrap().network_peers;
    let validator_set = consensus_peers.get_validator_set(network_peers);
    let discovery_set = make_placeholder_discovery_set(&validator_set);
    let (txn, hash) = encode_genesis_transaction_with_validator(
        &GENESIS_KEYPAIR.0,
        GENESIS_KEYPAIR.1.clone(),
        validator_set,
        discovery_set,
    );

    (
        lcs::to_bytes(&Transaction::UserTransaction(txn.into_inner()))
            .expect("Generating genesis block failed"),
        hash,
    )
}

fn main() {
    println!(
        "Creating genesis binary blob at {} from configuration file {}",
        GENESIS_LOCATION, CONFIG_LOCATION
    );

    let (genesis, hash) = generate_genesis_blob();
    let config = default_config(hash);
    config
        .save_config(CONFIG_LOCATION)
        .expect("Unable to save genesis config");

    let mut file = File::create(GENESIS_LOCATION).unwrap();
    file.write_all(&genesis).unwrap();
}

// A test that fails if the generated genesis blob is different from the one on disk. Intended
// to catch commits that
// - accidentally change the genesis block
// - change it without remembering to update the on-disk copy
// - cause generation of the genesis block to fail
#[test]
fn genesis_blob_unchanged() {
    let mut genesis_file = File::open(GENESIS_LOCATION).unwrap();
    let mut old_genesis_bytes = vec![];
    genesis_file.read_to_end(&mut old_genesis_bytes).unwrap();
    assert!(old_genesis_bytes == generate_genesis_blob(),
            format!("The freshly generated genesis file is different from the one on disk at {}. Did you forget to regenerate the genesis file via `cargo run` inside libra/language/vm/vm-genesis?", GENESIS_LOCATION));
}
