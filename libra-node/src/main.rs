// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
use libra_types::{chain_id::ChainId, PeerId};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
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
        load_test_environment(args.config, args.no_logging, args.random_ports);
    } else {
        let config = NodeConfig::load(args.config.unwrap()).expect("Failed to load node config");
        println!("Using node config {:?}", &config);
        start(args.no_logging, &config, None);
    };
}

fn start(no_logging: bool, config: &NodeConfig, log_file: Option<PathBuf>) {
    crash_handler::setup_panic_handler();

    let logger = if !no_logging {
        let mut logger = libra_logger::Logger::new();
        logger
            .channel_size(config.logger.chan_size)
            .is_async(config.logger.is_async)
            .level(config.logger.level)
            .read_env();
        if let Some(log_file) = log_file {
            logger.printer(Box::new(FileWriter::new(log_file)));
        }
        Some(logger.build())
    } else {
        None
    };

    // Let's now log some important information, since the logger is set up
    info!(config = config, "Loaded config");

    if config.metrics.enabled {
        for network in &config.full_node_networks {
            let peer_id = network.peer_id();
            setup_metrics(peer_id, &config);
        }

        if let Some(network) = config.validator_network.as_ref() {
            let peer_id = network.peer_id();
            setup_metrics(peer_id, &config);
        }
    }
    if fail::has_failpoints() {
        warn!("Failpoints is enabled");
        if let Some(failpoints) = &config.failpoints {
            for (point, actions) in failpoints {
                fail::cfg(point, actions).expect("fail to set actions for failpoint");
            }
        }
    } else if config.failpoints.is_some() {
        warn!("failpoints is set in config, but the binary doesn't compile with this feature");
    }

    let _node_handle = libra_node::setup_environment(&config, logger);
    let term = Arc::new(AtomicBool::new(false));

    while !term.load(Ordering::Acquire) {
        std::thread::park();
    }
}

fn setup_metrics(peer_id: PeerId, config: &NodeConfig) {
    libra_metrics::dump_all_metrics_to_file_periodically(
        &config.metrics.dir(),
        &format!("{}.metrics", peer_id),
        config.metrics.collection_interval_ms,
    );
}

fn load_test_environment(config_path: Option<PathBuf>, no_logging: bool, random_ports: bool) {
    // Either allocate a temppath or reuse the passed in path and make sure the directory exists
    let config_temp_path = libra_temppath::TempPath::new();
    let config_path = config_path.unwrap_or_else(|| config_temp_path.as_ref().to_path_buf());
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&config_path)
        .unwrap();
    let config_path = config_path.canonicalize().unwrap();

    // Build a single validator network
    let template = NodeConfig::default_for_validator();
    let builder =
        libra_genesis_tool::config_builder::ValidatorBuilder::new(1, template, &config_path)
            .randomize_first_validator_ports(random_ports);
    let test_config =
        libra_genesis_tool::swarm_config::SwarmConfig::build(&builder, &config_path).unwrap();

    // Prepare log file since we cannot automatically route logs to stderr
    let mut log_file = config_path.clone();
    log_file.push("validator.log");

    // Build a waypoint file so that clients / docker can grab it easily
    let mut waypoint_file_path = config_path.clone();
    waypoint_file_path.push("waypoint.txt");
    std::io::Write::write_all(
        &mut std::fs::File::create(&waypoint_file_path).unwrap(),
        test_config.waypoint.to_string().as_bytes(),
    )
    .unwrap();

    // Intentionally leave out instructions on how to connect with different applications
    println!("Completed generating configuration:");
    println!("\tLog file: {:?}", log_file);
    println!("\tConfig path: {:?}", test_config.config_files[0]);
    println!(
        "\tLibra root key path: {:?}",
        test_config.libra_root_key_path
    );
    println!("\tWaypoint: {}", test_config.waypoint);
    let config = NodeConfig::load(&test_config.config_files[0]).unwrap();
    println!("\tJSON-RPC endpoint: {}", config.rpc.address);
    println!(
        "\tFullNode network: {}",
        config.full_node_networks[0].listen_address
    );
    println!("\tChainId: {}", ChainId::test());
    println!();
    println!("Libra is running, press ctrl-c to exit");
    println!();

    start(no_logging, &config, Some(log_file))
}
