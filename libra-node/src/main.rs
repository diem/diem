// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::NodeConfig;
use libra_logger::prelude::*;
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

    let config = NodeConfig::load(args.config).expect("Failed to load node config");
    println!("Using node config {:?}", &config);
    crash_handler::setup_panic_handler();

    let logger = if !args.no_logging {
        Some(
            libra_logger::Logger::new()
                .channel_size(config.logger.chan_size)
                .is_async(config.logger.is_async)
                .level(config.logger.level)
                .read_env()
                .build(),
        )
    } else {
        None
    };

    // Let's now log some important information, since the logger is set up
    info!(StructuredLogEntry::new_named("config", "startup").data("config", &config));

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

    let _node_handle = libra_node::main_node::setup_environment(&config, logger);
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
