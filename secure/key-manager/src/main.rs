// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./key-manager node.config

#![forbid(unsafe_code)]

use libra_config::config::{KeyManagerConfig, NetworkConfig, NodeConfig};
use libra_key_manager::{
    counters::COUNTERS, libra_interface::JsonRpcLibraInterface, Error, KeyManager,
};
use libra_logger::info;
use libra_secure_json_rpc::JsonRpcClient;
use libra_secure_push_metrics::MetricsPusher;
use libra_secure_storage::{BoxStorage, Storage};
use libra_secure_time::RealTimeService;
use std::{convert::TryInto, env, net::SocketAddr, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Error! Incorrect number of parameters, expected a path to a config file.");
        process::exit(1);
    }

    let config = NodeConfig::load(&args[1]).unwrap_or_else(|e| {
        eprintln!(
            "Error! Unable to load provided config: {}, error: {}",
            args[1], e
        );
        process::exit(1);
    });

    if config.validator_network.is_none() {
        eprintln!("Error! Validator config missing from node config.");
        process::exit(1);
    }
    let network_config = config.validator_network.unwrap();
    let key_manager_config = config.secure.key_manager;

    libra_logger::Logger::new()
        .channel_size(config.logger.chan_size)
        .is_async(config.logger.is_async)
        .level(config.logger.level)
        .init();
    MetricsPusher::new(COUNTERS.clone()).start();

    create_and_execute_key_manager(network_config, key_manager_config).unwrap_or_else(|e| {
        eprintln!("Error! The Key Manager has failed during execution: {}", e);
        process::exit(1);
    });
}

fn create_and_execute_key_manager(
    network_config: NetworkConfig,
    key_manager_config: KeyManagerConfig,
) -> Result<(), Error> {
    let account = network_config.peer_id;
    let libra_interface = create_libra_interface(key_manager_config.json_rpc_address);
    let storage: Box<dyn Storage> = (&key_manager_config.secure_backend)
        .try_into()
        .expect("Unable to initialize storage");
    let time_service = RealTimeService::new();

    KeyManager::new(
        account,
        libra_interface,
        BoxStorage(storage),
        time_service,
        key_manager_config.rotation_period_secs,
        key_manager_config.sleep_period_secs,
        key_manager_config.txn_expiration_secs,
    )
    .execute()
}

fn create_libra_interface(json_rpc_address: SocketAddr) -> JsonRpcLibraInterface {
    let json_rpc_url = format!("https://{}", json_rpc_address.to_string());
    info!(
        "Creating a libra interface that talks to the JSON RPC endpoint at: {:?}.",
        json_rpc_url.clone()
    );
    let json_rpc_client = JsonRpcClient::new(json_rpc_url);
    JsonRpcLibraInterface::new(json_rpc_client)
}
