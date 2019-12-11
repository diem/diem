// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::ValidatorConfig;
use libra_config::config::NodeConfig;
use parity_multiaddr::Multiaddr;
use std::{convert::TryInto, fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to create Libra Validator Configs")]
struct Args {
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

    let mut config_builder = ValidatorConfig::new();
    config_builder
        .advertised(args.advertised)
        .bootstrap(args.bootstrap)
        .index(args.index)
        .listen(args.listen)
        .nodes(args.nodes)
        .template(template);

    if let Some(seed) = args.seed.as_ref() {
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        config_builder.seed(seed[..32].try_into().expect("Invalid seed"));
    }

    fs::create_dir_all(&args.output_dir).expect("Unable to create output directory");
    let mut node_config = config_builder.build().expect("ConfigBuilder failed");
    node_config.set_data_dir(args.data_dir);
    node_config
        .save(args.output_dir.join("node.config.toml"))
        .expect("Unable to save configs");
}
