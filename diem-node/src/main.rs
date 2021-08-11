// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use diem_config::config::NodeConfig;
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
    #[structopt(long, help = "Enabling random ports for testnet")]
    random_ports: bool,
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();

    if args.test {
        println!("Entering test mode, this should never be used in production!");
        diem_node::load_test_environment(args.config, args.random_ports, rand::rngs::OsRng);
    } else {
        let config = NodeConfig::load(args.config.unwrap()).expect("Failed to load node config");
        println!("Using node config {:?}", &config);
        diem_node::start(&config, None);
    };
}
