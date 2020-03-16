// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_verifier::VerifiedModule;
use libra_config::{config::PersistableConfig, generator};
use libra_types::transaction::Transaction;
use std::{fs::File, io::prelude::*};
use stdlib::{stdlib_modules, StdLibOptions};
use transaction_builder::default_config;
use vm_genesis::{
    encode_genesis_transaction_with_validator_and_modules, make_placeholder_discovery_set,
    GENESIS_KEYPAIR,
};

const CONFIG_LOCATION: &str = "genesis/vm_config.toml";
const GENESIS_LOCATION: &str = "genesis/genesis.blob";

/// Generate the genesis blob used by the Libra blockchain
fn generate_genesis_blob(stdlib_modules: &'static [VerifiedModule]) -> Vec<u8> {
    let swarm = generator::validator_swarm_for_testing(10);
    let discovery_set = make_placeholder_discovery_set(&swarm.validator_set);

    lcs::to_bytes(&Transaction::UserTransaction(
        encode_genesis_transaction_with_validator_and_modules(
            &GENESIS_KEYPAIR.0,
            GENESIS_KEYPAIR.1.clone(),
            &swarm.nodes,
            swarm.validator_set,
            discovery_set,
            stdlib_modules,
        )
        .into_inner(),
    ))
    .expect("Generating genesis block failed")
}

fn main() {
    println!(
        "Creating genesis binary blob at {} from configuration file {}",
        GENESIS_LOCATION, CONFIG_LOCATION
    );
    let config = default_config();
    config
        .save_config(CONFIG_LOCATION)
        .expect("Unable to save genesis config");

    let mut file = File::create(GENESIS_LOCATION).unwrap();
    // Must use staged stdlib files
    file.write_all(&generate_genesis_blob(stdlib_modules(
        StdLibOptions::Staged,
    )))
    .unwrap();
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
    assert!(old_genesis_bytes == generate_genesis_blob(stdlib_modules(StdLibOptions::Staged)),
            format!("The freshly generated genesis file is different from the one on disk at {}. Did you forget to regenerate the genesis file via `cargo run` inside libra/language/tools/vm-genesis?", GENESIS_LOCATION));
}
