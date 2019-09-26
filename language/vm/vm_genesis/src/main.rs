// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use config::{config::PersistableConfig, trusted_peers::ConfigHelpers};
use std::{fs::File, io::prelude::*};

use transaction_builder::default_config;
use vm_genesis::{encode_genesis_transaction_with_validator, GENESIS_KEYPAIR};

const CONFIG_LOCATION: &str = "genesis/vm_config.toml";
const GENESIS_LOCATION: &str = "genesis/genesis.blob";

use proto_conv::IntoProtoBytes;

/// Generate the genesis blob used by the Libra blockchain
fn generate_genesis_blob() -> Vec<u8> {
    let (_, consensus_config, network_config) = ConfigHelpers::gen_validator_nodes(10, None);
    encode_genesis_transaction_with_validator(
        &GENESIS_KEYPAIR.0,
        GENESIS_KEYPAIR.1.clone(),
        consensus_config.get_validator_set(&network_config),
    )
    .into_proto_bytes()
    .expect("Generating genesis block failed")
}

fn main() {
    println!(
        "Creating genesis binary blob at {} from configuration file {}",
        GENESIS_LOCATION, CONFIG_LOCATION
    );
    let config = default_config();
    config.save_config(CONFIG_LOCATION);

    let mut file = File::create(GENESIS_LOCATION).unwrap();
    file.write_all(&generate_genesis_blob()).unwrap();
}

// A test that fails if the generated genesis blob is different from the one on  disk. Intended
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
            format!("The freshly generated genesis file is different from the one on disk at {}. Did you forget to regenerate the genesis file via `cargo run` inside libra/language/vm/vm_genesis?", GENESIS_LOCATION));
}
