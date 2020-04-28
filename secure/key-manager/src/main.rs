// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Usage: ./key-manager node.config

#![forbid(unsafe_code)]

use libra_config::config::{KeyManagerConfig, NetworkConfig, NodeConfig, SecureBackend};
use libra_key_manager::{libra_interface::JsonRpcLibraInterface, Error, KeyManager};
use libra_logger::info;
use libra_secure_json_rpc::JsonRpcClient;
use libra_secure_storage::VaultStorage;
use libra_secure_time::RealTimeService;
use std::{env, net::SocketAddr, process};

// TODO(joshlind): initialize the metrics components for the key manager!
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
    let time_service = RealTimeService::new();

    match key_manager_config.secure_backend {
        SecureBackend::Vault(config) => {
            let storage = VaultStorage::new(
                config.server.clone(),
                config.token.clone(),
                config.namespace.clone(),
            );
            info!("Creating a key manager with vault secure storage. Vault server: {:?}, token: {:?}, namespace: {:?}.",
                  config.server,
                  config.token,
                  config.namespace);
            KeyManager::new(account, libra_interface, storage, time_service).execute()
        }
        _ => panic!("Unable to create a key manager with a secure backend that is not Vault!"),
    }
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
