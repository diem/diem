// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::NodeConfig;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Libra Node")]
struct Args {
    #[structopt(
        short = "f",
        long,
        required_unless = "test",
        help = "Path to NodeConfig"
    )]
    config: Option<PathBuf>,
    #[structopt(short = "d", long, help = "Disable logging")]
    no_logging: bool,
    #[structopt(long, help = "Enable a single validator testnet")]
    test: bool,
    #[structopt(long, help = "Enabling random ports for testnet")]
    random_ports: bool,
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();

    if args.test {
        println!("Entering test mode, this should never be used in production!");
        libra_node::load_test_environment(args.config, args.no_logging, args.random_ports);
    } else {
        let config = NodeConfig::load(args.config.unwrap()).expect("Failed to load node config");
        println!("Using node config {:?}", &config);
        libra_node::start(args.no_logging, &config, None);
    };
}
