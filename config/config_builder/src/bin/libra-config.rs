// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{value_t, App, Arg};
use config_builder::swarm_config::{LibraSwarmTopology, SwarmConfigBuilder};
use std::convert::TryInto;

const BASE_ARG: &str = "base";
const NODES_ARG: &str = "nodes";
const OUTPUT_DIR_ARG: &str = "output-dir";
const DISCOVERY_ARG: &str = "discovery";
const KEY_SEED_ARG: &str = "key-seed";
const FAUCET_ACCOUNT_FILE_ARG: &str = "faucet_account_file";

fn main() {
    let args = App::new("Libra Config Tool")
        .version("0.1.0")
        .author("Libra Association <opensource@libra.org>")
        .about("Tool to manage and create Libra Configs")
        .arg(
            Arg::with_name(BASE_ARG)
                .short("b")
                .long(BASE_ARG)
                .takes_value(true)
                .required(true)
                .help("Base config to use"),
        )
        .arg(
            Arg::with_name(NODES_ARG)
                .short("n")
                .long(NODES_ARG)
                .takes_value(true)
                .default_value("1")
                .help("Specify the number of nodes to configure"),
        )
        .arg(
            Arg::with_name(OUTPUT_DIR_ARG)
                .short("o")
                .long(OUTPUT_DIR_ARG)
                .takes_value(true)
                .help("The output directory"),
        )
        .arg(
            Arg::with_name(DISCOVERY_ARG)
                .short("d")
                .long(DISCOVERY_ARG)
                .help("Generate peer config with one peer only (to force discovery)"),
        )
        .arg(
            Arg::with_name(KEY_SEED_ARG)
                .short("s")
                .long(KEY_SEED_ARG)
                .takes_value(true)
                .help("Use the provided seed for generating keys for each of the validators"),
        )
        .arg(
            Arg::with_name(FAUCET_ACCOUNT_FILE_ARG)
                .short("m")
                .long(FAUCET_ACCOUNT_FILE_ARG)
                .help("File location from which to load faucet account generated via generate_keypair tool")
                .takes_value(true),
        )
        .get_matches();
    let base_path = value_t!(args, BASE_ARG, String).expect("Missing path to base config.");
    let nodes_count = value_t!(args, NODES_ARG, usize).expect("Missing node count.");
    let output_dir = if args.is_present(OUTPUT_DIR_ARG) {
        let dir = value_t!(args, OUTPUT_DIR_ARG, String).expect("Missing output directory.");
        dir.into()
    } else {
        ::std::env::current_dir().expect("Failed to access current directory.")
    };
    let faucet_account_file_path = value_t!(args, FAUCET_ACCOUNT_FILE_ARG, String)
        .expect("Must provide faucet account file path");
    let (faucet_account_keypair, _faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(Some(faucet_account_file_path));

    let topology = LibraSwarmTopology::create_validator_network(nodes_count);
    let mut config_builder = SwarmConfigBuilder::new();
    config_builder
        .with_topology(topology)
        .with_base(base_path)
        .with_output_dir(output_dir)
        .with_faucet_keypair(faucet_account_keypair);
    if args.is_present(DISCOVERY_ARG) {
        config_builder.force_discovery();
    }
    if args.is_present(KEY_SEED_ARG) {
        let seed_hex = value_t!(args, KEY_SEED_ARG, String).expect("Missing Seed");
        let seed = hex::decode(seed_hex).expect("Invalid hex in seed.");
        config_builder.with_key_seed(
            seed[..32]
                .try_into()
                .expect("Seed should be 32 bytes long."),
        );
    }
    let generated_configs = config_builder.build().expect("Unable to generate configs");

    println!(
        "Trusted Peers Config: {:?}",
        generated_configs.get_trusted_peers_config().0
    );

    println!(
        "Seed Peers Config: {:?}",
        generated_configs.get_seed_peers_config().0
    );

    for (path, node_config) in generated_configs.get_configs() {
        println!(
            "Node Config for PeerId({}): {:?}",
            node_config.network.peer_id, path
        );
        println!(
            "Node Keys for PeerId({}): {:?}",
            node_config.network.peer_id, node_config.network.peer_keypairs_file
        );
    }
}
