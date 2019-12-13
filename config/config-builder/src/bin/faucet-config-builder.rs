// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::ValidatorConfig;
use libra_config::config::{NodeConfig, PersistableConfig};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use std::{
    convert::TryInto,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to create Libra Faucet Configs")]
struct Args {
    #[structopt(short = "n", long, default_value = "1")]
    /// Specify the number of nodes to configure
    nodes: usize,
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory
    output_dir: PathBuf,
    #[structopt(short = "s", long)]
    /// Use the provided seed for generating keys for each of the validators
    seed: Option<String>,
}

fn main() {
    let args = Args::from_args();

    let mut config_builder = ValidatorConfig::new();
    config_builder
        .nodes(args.nodes)
        .template(NodeConfig::default());

    if let Some(seed) = args.seed.as_ref() {
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        config_builder.seed(seed[..32].try_into().expect("Invalid seed"));
    }

    fs::create_dir_all(&args.output_dir).expect("Unable to create output directory");
    let (consensus_peers, faucet_key) = config_builder
        .build_faucet_client()
        .expect("ConfigBuilder failed");

    let key_path = args.output_dir.join("mint.key");
    let faucet_keypair = KeyPair::<Ed25519PrivateKey, Ed25519PublicKey>::from(faucet_key);
    let serialized_keys = lcs::to_bytes(&faucet_keypair).expect("Unable to serialize keys");
    let mut key_file = File::create(key_path).expect("Unable to create key file");
    key_file
        .write_all(&serialized_keys)
        .expect("Unable to write to key file");

    let consensus_peers_path = args.output_dir.join("consensus_peers.config.toml");
    consensus_peers
        .save_config(consensus_peers_path)
        .expect("Unable to save consensus_peers.config");
}
