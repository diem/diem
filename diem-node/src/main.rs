// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_config::config::NodeConfig;
use diem_types::on_chain_config::VMPublishingOption;
use hex::FromHex;
use rand::{rngs::StdRng, SeedableRng};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Diem Node")]
struct Args {
    #[structopt(
        short = "f",
        long,
        required_unless = "test",
        help = "Path to NodeConfig"
    )]
    config: Option<PathBuf>,
    #[structopt(long, help = "Enable a single validator testnet")]
    test: bool,

    #[structopt(
        long,
        help = "RNG Seed to use when starting single validator testnet",
        parse(try_from_str = FromHex::from_hex),
        requires("test")
    )]
    seed: Option<[u8; 32]>,

    #[structopt(
        long,
        help = "Enable open publishing when starting single validator testnet",
        requires("test")
    )]
    open_publishing: bool,

    #[structopt(long, help = "Enabling random ports for testnet", requires("test"))]
    random_ports: bool,
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();

    if args.test {
        println!("Entering test mode, this should never be used in production!");
        let rng = args
            .seed
            .map(StdRng::from_seed)
            .unwrap_or_else(StdRng::from_entropy);
        let publishing_option = if args.open_publishing {
            Some(VMPublishingOption::open())
        } else {
            None
        };
        diem_node::load_test_environment(args.config, args.random_ports, publishing_option, rng);
    } else {
        let config = NodeConfig::load(args.config.unwrap()).expect("Failed to load node config");
        println!("Using node config {:?}", &config);
        diem_node::start(&config, None);
    };
}
