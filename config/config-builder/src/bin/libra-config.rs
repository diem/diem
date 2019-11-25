// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::swarm_config::SwarmConfigBuilder;
use libra_config::config::RoleType;
use std::convert::TryInto;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to manage and create Libra Configs")]
struct Args {
    #[structopt(short = "b", long, parse(from_os_str))]
    /// Base config to use
    base: PathBuf,
    #[structopt(short = "n", long, default_value = "1")]
    /// Specify the number of nodes to configure
    nodes: usize,
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory
    output_dir: Option<PathBuf>,
    #[structopt(short = "d", long)]
    /// Generate peer config with one peer only (to force discovery)
    discovery: bool,
    #[structopt(short = "s", long)]
    /// Use the provided seed for generating keys for each of the validators
    key_seed: Option<String>,
    #[structopt(short = "m", long)]
    /// File location from which to load faucet account generated via generate_keypair tool
    faucet_account_file: String,
    #[structopt(short = "r", long, default_value = "validator")]
    /// Role for the nodes: one of {"validator", "full_node"}
    role: String,
    #[structopt(short = "u", long)]
    /// Config directory for upstream node. This field is needed if role is "full_node"
    upstream_config_dir: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let output_dir = if let Some(output_dir) = args.output_dir.as_ref() {
        output_dir.clone()
    } else {
        ::std::env::current_dir().expect("Failed to access current directory.")
    };
    let (faucet_account_keypair, _faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(Some(args.faucet_account_file.clone()));
    let role: RoleType = args.role.clone().into();

    let mut config_builder = SwarmConfigBuilder::new();
    config_builder
        .with_num_nodes(args.nodes)
        .with_role(role)
        .with_base(&args.base)
        .with_output_dir(output_dir)
        .with_faucet_keypair(faucet_account_keypair)
        .with_upstream_config_dir(args.upstream_config_dir.clone());

    if args.discovery {
        config_builder.force_discovery();
    }
    if let Some(key_seed) = args.key_seed.as_ref() {
        let seed = hex::decode(key_seed).expect("Invalid hex in seed.");
        config_builder.with_key_seed(
            seed[..32]
                .try_into()
                .expect("Seed should be 32 bytes long."),
        );
    }
    config_builder.build().expect("Unable to generate configs");
}
