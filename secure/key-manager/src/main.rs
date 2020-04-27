// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./key-manager node.config

#![forbid(unsafe_code)]

use libra_config::config::{KeyManagerConfig, NetworkConfig, NodeConfig, SecureBackend};
use libra_key_manager::{libra_interface::JsonRpcLibraInterface, Error, KeyManager};
use libra_secure_json_rpc::JsonRpcClient;
use libra_secure_storage::{InMemoryStorage, OnDiskStorage, VaultStorage};
use libra_secure_time::RealTimeService;
use std::{env, process};

// TODO(joshlind): initialize the metrics components for the key manager!
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

    if config.validator_network.is_none() {
        eprintln!("Validator config missing from node config");
        process::exit(1);
    }
    let network_config = config.validator_network.unwrap();
    let key_manager_config = config.secure.key_manager;

    create_and_execute_key_manager(network_config, key_manager_config).unwrap_or_else(|e| {
        eprintln!("The Key Manager has failed during execution: {}", e);
        process::exit(1);
    });
}

fn create_and_execute_key_manager(
    network_config: NetworkConfig,
    key_manager_config: KeyManagerConfig,
) -> Result<(), Error> {
    // Create the json rpc based libra interface
    let json_rpc_url = format!(
        "https://{}",
        key_manager_config.json_rpc_address.to_string()
    );
    let json_rpc_client = JsonRpcClient::new(json_rpc_url);
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_client);

    // Create the key manager based on the backend and execute
    match key_manager_config.secure_backend {
        SecureBackend::InMemoryStorage => KeyManager::new(
            network_config.peer_id,
            libra_interface,
            InMemoryStorage::new(),
            RealTimeService::new(),
        )
        .execute(),
        SecureBackend::OnDiskStorage(config) => KeyManager::new(
            network_config.peer_id,
            libra_interface,
            OnDiskStorage::new(config.path()),
            RealTimeService::new(),
        )
        .execute(),
        SecureBackend::Vault(config) => KeyManager::new(
            network_config.peer_id,
            libra_interface,
            VaultStorage::new(config.server, config.token, config.namespace),
            RealTimeService::new(),
        )
        .execute(),
    }
}
