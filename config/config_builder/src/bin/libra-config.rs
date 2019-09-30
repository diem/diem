// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::{value_t, App, Arg};
use libra_config::config::RoleType;
use libra_config_builder::swarm_config::SwarmConfigBuilder;
use std::convert::TryInto;

const BASE_ARG: &str = "base";
const NODES_ARG: &str = "nodes";
const OUTPUT_DIR_ARG: &str = "output-dir";
const DISCOVERY_ARG: &str = "discovery";
const KEY_SEED_ARG: &str = "key-seed";
const FAUCET_ACCOUNT_FILE_ARG: &str = "faucet_account_file";
const ROLE_ARG: &str = "role";
const UPSTREAM_CONFIG_DIR_ARG: &str = "upstream_config_dir";

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
        .arg(
            Arg::with_name(ROLE_ARG)
            .short("r")
            .long(ROLE_ARG)
            .help("Role for the nodes: one of {\"validator\", \"full_node\"}")
            .default_value("validator")
            .takes_value(true),
        )
        .arg(
            Arg::with_name(UPSTREAM_CONFIG_DIR_ARG)
            .short("u")
            .long(UPSTREAM_CONFIG_DIR_ARG)
            .help("Config directory for upstream node. This field is needed if role is \"full_node\"")
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
        libra_generate_keypair::load_faucet_key_or_create_default(Some(faucet_account_file_path));
    let role: RoleType = value_t!(args, ROLE_ARG, String)
        .expect("Missing role type")
        .into();

    let mut config_builder = SwarmConfigBuilder::new();
    config_builder
        .with_num_nodes(nodes_count)
        .with_role(role)
        .with_base(base_path)
        .with_output_dir(output_dir)
        .with_faucet_keypair(faucet_account_keypair)
        .with_upstream_config_dir(value_t!(args, UPSTREAM_CONFIG_DIR_ARG, String).ok());

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
    config_builder.build().expect("Unable to generate configs");
}
