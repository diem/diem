// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./key-manager node.config

#![forbid(unsafe_code)]

use libra_config::config::KeyManagerConfig;
use libra_key_manager::{
    libra_interface::JsonRpcLibraInterface,
    logging::{LogEntry, LogEvent, LogField},
    Error, KeyManager,
};
use libra_secure_push_metrics::MetricsPusher;
use libra_secure_storage::Storage;
use libra_secure_time::RealTimeService;
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

    libra_logger::Logger::new()
        .channel_size(key_manager_config.logger.chan_size)
        .is_async(key_manager_config.logger.is_async)
        .level(key_manager_config.logger.level)
        .read_env()
        .init();

    let _mp = MetricsPusher::start();

    create_and_execute_key_manager(key_manager_config).unwrap_or_else(|e| {
        eprintln!("Error! The Key Manager has failed during execution: {}", e);
        process::exit(1);
    });
}

fn create_and_execute_key_manager(key_manager_config: KeyManagerConfig) -> Result<(), Error> {
    let json_rpc_endpoint = key_manager_config.json_rpc_endpoint;
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_endpoint.clone());
    let storage: Storage = (&key_manager_config.secure_backend)
        .try_into()
        .expect("Unable to initialize storage");
    let time_service = RealTimeService::new();

    let mut key_manager = KeyManager::new(
        libra_interface,
        storage,
        time_service,
        key_manager_config.rotation_period_secs,
        key_manager_config.sleep_period_secs,
        key_manager_config.txn_expiration_secs,
        key_manager_config.chain_id,
    );

    key_manager.log(
        LogEntry::Initialized,
        Some(LogEvent::Success),
        Some((LogField::JsonRpcEndpoint, json_rpc_endpoint)),
    );
    key_manager.execute()
}
