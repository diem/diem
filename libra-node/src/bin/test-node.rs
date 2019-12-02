// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use executable_helpers::helpers::setup_executable;
use hex;
use libra_config::config::{NodeConfig, PersistableConfig, SeedPeersConfig};
use libra_crypto::{ed25519::Ed25519PrivateKey, test_utils::KeyPair, PrivateKey, Uniform};
use config_builder::swarm_config::SwarmConfigBuilder;
use parity_multiaddr::{Multiaddr};
use rand::{Rng, rngs::StdRng, SeedableRng};
use std::{
    collections::HashMap,
    convert::TryInto,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::StructOpt;
use libra_tools::tempdir::TempPath;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Node")]
struct Args {
    #[structopt(short = "4", long)]
    /// Use IPv4
    ipv4: bool,
    #[structopt(short = "b", long, parse(from_str = parse_addr))]
    /// Advertised address for the first node in this test net
    bootstrap: Option<Multiaddr>,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
    #[structopt(short = "i", long, default_value = "0")]
    /// Config index into the validator set for this node
    node: usize,
    #[structopt(short = "l", long, parse(from_str = parse_addr))]
    /// Listening address for the first node in this test net
    listen: Option<Multiaddr>,
    #[structopt(short = "n", long, default_value = "1")]
    /// Specify the number of nodes in the validator set
    nodes: usize,
    #[structopt(short = "s", long)]
    /// Optional seed for generating keys for each of the validators
    seed: Option<String>,
    #[structopt(short = "t", long, parse(from_os_str))]
    /// Path to template NodeConfig
    template: Option<PathBuf>,
}

fn parse_addr(src: &str) -> Multiaddr {
    src.parse::<Multiaddr>().unwrap()
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();
    assert!(args.node < args.nodes, format!("Node index {} >= Nodes {}", args.node, args.nodes));

    let seed = if let Some(seed) = args.seed.as_ref() {
        let arg_seed = hex::decode(seed).expect("Invalid hex in seed.");
        arg_seed[..32].try_into().expect("Expected 32 bytes")
    } else {
        [0u8; 32]
    };

    let mut rng = StdRng::from_seed(seed);
    let faucet_privkey = Ed25519PrivateKey::generate_for_testing(&mut rng);
    let faucet_pubkey = faucet_privkey.public_key();
    let faucet_keypair = KeyPair {
        private_key: faucet_privkey,
        public_key: faucet_pubkey,
    };
    let config_seed = rng.gen::<[u8; 32]>();
    let temp_dir = TempPath::new();

    let mut config_builder = SwarmConfigBuilder::new();
    config_builder
        .with_faucet_keypair(faucet_keypair)
        .with_key_seed(config_seed)
        .with_num_nodes(args.nodes)
        .with_output_dir(temp_dir.path());
    if args.ipv4 {
        config_builder.with_ipv4();
    }
    if let Some(template) = args.template.as_ref() {
        config_builder.with_base(template);
    }

    let configs = config_builder.build().expect("Unable to generate configs");

    let config_path = Some(configs.config_files[args.node].as_path());

    let (mut config, _logger) = setup_executable(config_path, args.no_logging);
    if let Some(listen) = args.listen {
        config.validator_network.as_mut().unwrap().listen_address = listen;
    }
    if let Some(bootstrap) = args.bootstrap {
        let peer0_config = NodeConfig::load_config(&configs.config_files[0]).unwrap();
        let peer0_id = peer0_config.validator_network.unwrap().peer_id;

        let network = config.validator_network.as_mut().unwrap();
        let mut seed_peers = HashMap::new();
        seed_peers.insert(peer0_id, vec![bootstrap]);
        network.seed_peers = SeedPeersConfig { seed_peers };
    }

    let _node_handle = libra_node::main_node::setup_environment(&mut config);
    let term = Arc::new(AtomicBool::new(false));
    while !term.load(Ordering::Acquire) {
        std::thread::park();
    }
}
