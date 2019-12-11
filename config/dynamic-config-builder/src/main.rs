// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use dynamic_config_builder::DynamicConfigBuilder;
use libra_config::config::{NodeConfig, PersistableConfig};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use parity_multiaddr::Multiaddr;
use std::{
    convert::TryInto,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to create Libra Configs")]
struct Args {
    // The following are generic options for generating both validator and full node networks
    #[structopt(short = "6", long)]
    /// Use IPv6
    ipv6: bool,
    #[structopt(short = "a", long, parse(from_str = parse_addr))]
    /// Advertised address for this node, if this is null, listen is reused
    advertised: Multiaddr,
    #[structopt(short = "b", long, parse(from_str = parse_addr))]
    /// Advertised address for the first node in this test net
    bootstrap: Multiaddr,
    #[structopt(short = "d", long, parse(from_os_str))]
    /// The data directory for the configs (e.g. /opt/libra/etc)
    data_dir: PathBuf,
    #[structopt(short = "i", long, default_value = "0")]
    /// Specify the index into the number of nodes to write to output dir
    index: usize,
    #[structopt(short = "l", long, parse(from_str = parse_addr))]
    /// Listening address for this node
    listen: Multiaddr,
    #[structopt(short = "n", long, default_value = "1")]
    /// Specify the number of nodes to configure
    nodes: usize,
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory
    output_dir: PathBuf,
    #[structopt(short = "s", long)]
    /// Use the provided seed for generating keys for each of the validators
    seed: Option<String>,
    #[structopt(short = "t", long, parse(from_os_str))]
    /// Path to a template NodeConfig
    template: Option<PathBuf>,

    // Specialized options
    #[structopt(short = "c", long)]
    /// Build a faucet / client config
    faucet_client: bool,
}

fn parse_addr(src: &str) -> Multiaddr {
    src.parse::<Multiaddr>().unwrap()
}

fn main() {
    let args = Args::from_args();

    let template = if let Some(template_path) = args.template {
        NodeConfig::load(template_path).expect("Unable to load template")
    } else {
        NodeConfig::default()
    };

    let mut config_builder = DynamicConfigBuilder::new();
    config_builder
        .advertised(args.advertised)
        .bootstrap(args.bootstrap)
        .index(args.index)
        .ipv4(!args.ipv6)
        .listen(args.listen)
        .nodes(args.nodes)
        .template(template);

    if let Some(seed) = args.seed.as_ref() {
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        config_builder.seed(seed[..32].try_into().expect("Invalid seed"));
    }

    fs::create_dir_all(&args.output_dir).expect("Unable to create output directory");

    if args.faucet_client {
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
        consensus_peers.save_config(consensus_peers_path);
    } else {
        let mut node_config = config_builder.build().expect("ConfigBuilder failed");
        node_config
            .set_data_dir(args.output_dir.clone())
            .expect("Unable to set directory");
        let config_file = args.output_dir.join("node.config.toml");
        node_config.save(&config_file);
        node_config
            .set_data_dir(args.data_dir)
            .expect("Unable to set directory");
        node_config.save_config(&config_file);
    }
}
