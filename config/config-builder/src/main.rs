// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use config_builder::{FullNodeConfig, ValidatorConfig};
use libra_config::config::NodeConfig;
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

const NODE_CONFIG: &str = "node.config.toml";

#[allow(clippy::large_enum_variant)]
#[derive(Debug, StructOpt)]
#[structopt(about = "Tool to create and extend Libra Configs")]
enum Args {
    #[structopt(about = "Generate a Libra faucet key")]
    Faucet(FaucetArgs),
    #[structopt(about = "Create or extend a FullNode network")]
    FullNode(FullNodeCommand),
    #[structopt(about = "Create a new validator config")]
    Validator(ValidatorArgs),
}

#[derive(Debug, StructOpt)]
struct FaucetArgs {
    #[structopt(short = "o", long, parse(from_os_str))]
    /// The output directory
    output_dir: PathBuf,
    #[structopt(short = "s", long)]
    /// Use the provided seed for generating keys for each of the validators
    seed: Option<String>,
}

#[derive(Debug, StructOpt)]
enum FullNodeCommand {
    #[structopt(about = "Create a new config")]
    Create(FullNodeArgs),
    #[structopt(about = "Create a new config")]
    Extend(FullNodeArgs),
}

#[derive(Debug, StructOpt)]
struct FullNodeArgs {
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

#[derive(Debug, StructOpt)]
struct ValidatorArgs {
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

    match args {
        Args::Faucet(faucet_args) => build_faucet(faucet_args),
        Args::FullNode(full_node_args) => build_full_node(full_node_args),
        Args::Validator(validator_args) => build_validator(validator_args),
    };
}

fn build_faucet(args: FaucetArgs) {
    let mut config_builder = ValidatorConfig::new();

    if let Some(seed) = args.seed.as_ref() {
        let seed = hex::decode(seed).expect("Invalid hex in seed.");
        config_builder.seed(seed[..32].try_into().expect("Invalid seed"));
    }

    let (_, faucet_key) = config_builder
        .build_faucet_client()
        .expect("ConfigBuilder failed");

    let key_path = args.output_dir.join("mint.key");
    let faucet_keypair = KeyPair::<Ed25519PrivateKey, Ed25519PublicKey>::from(faucet_key);
    let serialized_keys = lcs::to_bytes(&faucet_keypair).expect("Unable to serialize keys");

    fs::create_dir_all(&args.output_dir).expect("Unable to create output directory");
    let mut key_file = File::create(key_path).expect("Unable to create key file");
    key_file
        .write_all(&serialized_keys)
        .expect("Unable to write to key file");
}

fn build_full_node(command: FullNodeCommand) {
    let config_builder = match &command {
        FullNodeCommand::Create(args) => build_full_node_config_builder(&args),
        FullNodeCommand::Extend(args) => build_full_node_config_builder(&args),
    };

    match command {
        FullNodeCommand::Create(args) => {
            if node_config_exists(&args.output_dir) {
                eprintln!("Node config already exists in this directory");
                return;
            }
            let mut new_config = config_builder.build().expect("ConfigBuilder failed");
            new_config.set_data_dir(args.data_dir);
            save_config(new_config, &args.output_dir);
        },
        FullNodeCommand::Extend(args) => {
            if !node_config_exists(&args.output_dir) {
                eprintln!("No node config in this directory");
                return;
            }

            let config_file = args.output_dir.join(NODE_CONFIG);
            let mut orig_config = NodeConfig::load(&config_file).expect("Unable to load node config");
            if orig_config.base.role.is_validator() {
                config_builder
                    .extend_validator(&mut orig_config)
                    .expect("Unable to add full node network to validator");
            } else {
                config_builder
                    .extend(&mut orig_config)
                    .expect("Unable to append full node network");
            }
            save_config(orig_config, &args.output_dir);
        },
    };
}

fn build_full_node_config_builder(args: &FullNodeArgs) -> FullNodeConfig {
    let mut config_builder = FullNodeConfig::new();
    config_builder
        .advertised(args.advertised.clone())
        .bootstrap(args.bootstrap.clone())
        .full_node_index(args.index)
        .full_nodes(args.full_nodes)
        .listen(args.listen.clone())
        .nodes(args.nodes)
        .template(load_template(args.template.clone()));

    if let Some(fn_seed) = args.full_node_seed.as_ref() {
        config_builder.full_node_seed(parse_seed(fn_seed));
    }

    if args.public {
        config_builder.public();
    }

    if let Some(seed) = args.seed.as_ref() {
        config_builder.seed(parse_seed(seed));
    }

    config_builder
}

fn build_validator(args: ValidatorArgs) {
    if node_config_exists(&args.output_dir) {
        eprintln!("Node config already exists in this directory");
        return;
    }

    let mut config_builder = ValidatorConfig::new();
    config_builder
        .advertised(args.advertised)
        .bootstrap(args.bootstrap)
        .index(args.index)
        .listen(args.listen)
        .nodes(args.nodes)
        .template(load_template(args.template));

    if let Some(seed) = args.seed.as_ref() {
        config_builder.seed(parse_seed(seed));
    }

    let mut node_config = config_builder.build().expect("ConfigBuilder failed");
    node_config.set_data_dir(args.data_dir);
    save_config(node_config, &args.output_dir);
}

fn node_config_exists(output_dir: &PathBuf) -> bool {
    output_dir.join(NODE_CONFIG).exists()
}

fn load_template(template: Option<PathBuf>) -> NodeConfig {
    if let Some(template_path) = template {
        NodeConfig::load(template_path).expect("Unable to load template")
    } else {
        NodeConfig::default()
    }
}

fn parse_seed(seed: &str) -> [u8; 32] {
    let seed = hex::decode(seed).expect("Invalid hex in seed.");
    seed[..32].try_into().expect("Invalid seed")
}

fn save_config(mut node_config: NodeConfig, output_dir: &PathBuf) {
    fs::create_dir_all(output_dir).expect("Unable to create output directory");
    node_config
        .save(output_dir.join(NODE_CONFIG))
        .expect("Unable to save configs");
}
