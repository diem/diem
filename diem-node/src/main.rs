// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use diem_config::config::NodeConfig;
use diem_types::on_chain_config::{
    default_access_path_for_config, experimental_version_access_path, DiemVersion, OnChainConfig,
    VMPublishingOption,
};
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

    #[structopt(
        long,
        help = "Paths to module blobs to be included in genesis. Can include both files and directories",
        requires("test")
    )]
    genesis_modules: Option<Vec<PathBuf>>,

    #[structopt(
        long,
        help = "Lazy mode, set this flag will set `consensus#mempool_poll_count` config to `u64::MAX` and only commit a block when there is user transaction in mempool",
        requires("test")
    )]
    lazy: bool,

    #[structopt(
        long,
        help = "Mode of config access path and modules",
        requires("test"),
        possible_values(&["DPN", "experimental"])
    )]
    mode: Option<String>,
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
        let genesis_modules = if let Some(module_paths) = args.genesis_modules {
            diem_framework_releases::load_modules_from_paths(&module_paths)
        } else if args.mode == Some("experimental".to_string()) {
            diem_experimental_framework_releases::current_module_blobs().to_vec()
        } else {
            diem_framework_releases::current_module_blobs().to_vec()
        };
        let diem_version_access_path = if args.mode == Some("experimental".to_string()) {
            experimental_version_access_path()
        } else {
            default_access_path_for_config(DiemVersion::CONFIG_ID)
        };
        diem_node::load_test_environment(
            args.config,
            args.random_ports,
            args.lazy,
            publishing_option,
            genesis_modules,
            Some(diem_version_access_path),
            rng,
        );
    } else {
        let config = NodeConfig::load(args.config.unwrap()).expect("Failed to load node config");
        println!("Using node config {:?}", &config);
        diem_node::start(&config, None);
    };
}
