// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./key-manager node.config

#![forbid(unsafe_code)]

use diem_config::config::KeyManagerConfig;
use diem_key_manager::{
    counters,
    diem_interface::JsonRpcDiemInterface,
    logging::{LogEntry, LogEvent, LogSchema},
    Error, KeyManager,
};
use diem_logger::info;
use diem_secure_push_metrics::MetricsPusher;
use diem_secure_storage::Storage;
use diem_time_service::TimeService;
use std::{convert::TryInto, env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error! Incorrect number of parameters, expected a path to a config file.");
        process::exit(1);
    }

    let key_manager_config = KeyManagerConfig::load(&args[1]).unwrap_or_else(|e| {
        eprintln!(
            "Error! Unable to load provided key manager config: {}, error: {}",
            args[1], e
        );
        process::exit(1);
    });

    diem_logger::Logger::new()
        .channel_size(key_manager_config.logger.chan_size)
        .is_async(key_manager_config.logger.is_async)
        .level(key_manager_config.logger.level)
        .read_env()
        .init();

    crash_handler::setup_panic_handler();
    let _mp = MetricsPusher::start();
    counters::initialize_all_metric_counters();

    create_and_execute_key_manager(key_manager_config).unwrap_or_else(|e| {
        eprintln!("Error! The Key Manager has failed during execution: {}", e);
        process::exit(1);
    });
}

fn create_and_execute_key_manager(key_manager_config: KeyManagerConfig) -> Result<(), Error> {
    info!(
        LogSchema::new(LogEntry::Initialized).event(LogEvent::Pending),
        key_manager_config = key_manager_config
    );

    let json_rpc_endpoint = key_manager_config.json_rpc_endpoint;
    let diem_interface = JsonRpcDiemInterface::new(json_rpc_endpoint.clone());
    let storage: Storage = (&key_manager_config.secure_backend)
        .try_into()
        .expect("Unable to initialize storage");

    let mut key_manager = KeyManager::new(
        diem_interface,
        storage,
        TimeService::real(),
        key_manager_config.rotation_period_secs,
        key_manager_config.sleep_period_secs,
        key_manager_config.txn_expiration_secs,
        key_manager_config.chain_id,
    );

    info!(LogSchema::new(LogEntry::Initialized)
        .event(LogEvent::Success)
        .json_rpc_endpoint(&json_rpc_endpoint));

    key_manager.execute()
}
