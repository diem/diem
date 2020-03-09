// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::NodeConfig;
use libra_types::PeerId;
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
    #[structopt(short = "f", long, parse(from_os_str))]
    /// Path to NodeConfig
    config: PathBuf,
    #[structopt(short = "d", long)]
    /// Disable logging
    no_logging: bool,
}

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    let args = Args::from_args();

    let mut config = NodeConfig::load(args.config).expect("Failed to load node config");
    println!("Using node config {:?}", &config);
    crash_handler::setup_panic_handler();

    if !args.no_logging {
        libra_logger::Logger::new()
            .channel_size(config.logger.chan_size)
            .is_async(config.logger.is_async)
            .level(config.logger.level)
            .init();
    }

    if config.metrics.enabled {
        for network in &config.full_node_networks {
            setup_metrics(network.peer_id, &config);
        }

        if let Some(network) = config.validator_network.as_ref() {
            setup_metrics(network.peer_id, &config);
        }
    }

    let _node_handle = libra_node::main_node::setup_environment(&mut config);

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
