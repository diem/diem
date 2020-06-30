// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./safety-rules node.config

#![forbid(unsafe_code)]

use libra_config::config::{PersistableConfig, SafetyRulesConfig};
use libra_secure_push_metrics::MetricsPusher;
use safety_rules::{Process, COUNTERS};
use std::{env, process};
mod safety_rules_sgx_runner;

fn main() {
    let args: Vec<String> = env::args().collect();

    // lwg: this is for testing.. removal will be there
    safety_rules_sgx_runner::start_lsr_enclave();

    if args.len() != 2 {
        eprintln!("Incorrect number of parameters, expected a path to a config file");
        process::exit(1);
    }

    let config = SafetyRulesConfig::load_config(&args[1]).unwrap_or_else(|e| {
        eprintln!("Unable to read provided config: {}", e);
        process::exit(1);
    });

    libra_logger::Logger::new()
        .channel_size(config.logger.chan_size)
        .is_async(config.logger.is_async)
        .level(config.logger.level)
        .init();
    libra_logger::init_struct_log_from_env().expect("Failed to initialize structured logging");

    MetricsPusher::new(COUNTERS.clone()).start();

    let mut service = Process::new(config);
    service.start();
}
