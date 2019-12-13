// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::FullNodeConfig;
use libra_config::config::{NodeConfig, PersistableConfig};
use parity_multiaddr::Multiaddr;
use std::{convert::TryInto, fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to create Libra Validator Configs")]
struct Args {
    // Describe the validator networrk
    #[structopt(short = "n", long, default_value = "1")]
    /// Specify the number of nodes to configure
    nodes: usize,
    #[structopt(short = "s", long)]
    /// Use the provided seed for generating keys for each of the validators
    seed: Option<String>,

    // Parameters for this full node config
    #[structopt(short = "a", long, parse(from_str = parse_addr))]
    /// Advertised address for this node, if this is null, listen is reused
    advertised: Multiaddr,
    #[structopt(short = "b", long, parse(from_str = parse_addr))]
    /// Advertised address for the first node in this test net
    bootstrap: Multiaddr,
    #[structopt(short = "d", long, parse(from_os_str))]
    /// The data directory for the configs (e.g. /opt/libra/data)
    data_dir: PathBuf,
    #[structopt(short = "c", long)]
    /// Use the provided seed for generating keys for each of the FullNodes
    full_node_seed: Option<String>,
    #[structopt(short = "f", long, default_value = "1")]
    /// Total number of full nodes
    full_nodes: usize,
    #[structopt(short = "i", long, default_value = "0")]
    /// Specify the index into the number of nodes to write to output dir
    index: usize,
    #[structopt(short = "l", long, parse(from_str = parse_addr))]
    /// Listening address for this node
    listen: Multiaddr,
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory, note if a config exists already here, it will be updated to include
    /// this full node network
    output_dir: PathBuf,
    #[structopt(short = "p", long)]
    /// Public network, doesn't use any authentication or encryption
    public: bool,
    #[structopt(short = "t", long, parse(from_os_str))]
    /// Path to a template NodeConfig
    template: Option<PathBuf>,
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

    let mut config_builder = FullNodeConfig::new();
    config_builder
        .advertised(args.advertised)
        .bootstrap(args.bootstrap)
        .full_node_index(args.index)
        .full_nodes(args.full_nodes)
        .listen(args.listen)
        .nodes(args.nodes)
        .template(template);

    if let Some(fn_seed) = args.full_node_seed.as_ref() {
        let fn_seed = hex::decode(fn_seed).expect("Invalid hex in full node seed.");
        config_builder.full_node_seed(fn_seed[..32].try_into().expect("Invalid full node seed"));
    }

    if args.public {
        config_builder.public();
    }

    if let Some(seed) = args.seed.as_ref() {
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        config_builder.seed(seed[..32].try_into().expect("Invalid seed"));
    }

    fs::create_dir_all(&args.output_dir).expect("Unable to create output directory");

    let config_file = args.output_dir.join("node.config.toml");

    let orig_config = NodeConfig::load_config(&config_file);
    let mut node_config = if let Ok(mut orig_config) = orig_config {
        if orig_config.base.role.is_validator() {
            config_builder
                .extend_validator(&mut orig_config)
                .expect("Unable to add full node network to validator");
        } else {
            config_builder
                .extend(&mut orig_config)
                .expect("Unable to append full node network");
        }
        orig_config
    } else {
        let mut new_config = config_builder.build().expect("ConfigBuilder failed");
        new_config.set_data_dir(args.data_dir);
        new_config
    };

    node_config
        .save(args.output_dir.join("node.config.toml"))
        .expect("Unable to save configs");
}
