// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./safety-rules node.config

#![forbid(unsafe_code)]

use libra_config::config::NodeConfig;
use libra_secure_push_metrics::MetricsPusher;
use safety_rules::{Process, COUNTERS};
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Incorrect number of parameters, expected a path to a config file");
        process::exit(1);
    }

    let config = NodeConfig::load(&args[1]).unwrap_or_else(|e| {
        eprintln!("Unable to read provided config: {}", e);
        process::exit(1);
    });

    libra_logger::Logger::new()
        .channel_size(config.logger.chan_size)
        .is_async(config.logger.is_async)
        .level(config.logger.level)
        .init();
    MetricsPusher::new(COUNTERS.clone()).start();
    let mut service = Process::new(config);
    service.start();
}
