// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::smoke_test_environment::SmokeTestEnvironment;
use cli::client_proxy::ClientProxy;
use libra_config::config::{Identity, NodeConfig, SecureBackend};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{collections::BTreeMap, str::FromStr, string::ToString};

// TODO(joshlind): Refactor all of these so that they can be contained within the calling
// test files and not shared across all tests.
pub fn compare_balances(
    expected_balances: Vec<(f64, String)>,
    extracted_balances: Vec<String>,
) -> bool {
    if extracted_balances.len() != extracted_balances.len() {
        return false;
    }

    let extracted_balances_dec: BTreeMap<_, _> = extracted_balances
        .into_iter()
        .map(|balance_str| {
            let (currency_code, stripped_str) = if balance_str.ends_with("Coin1") {
                ("Coin1", balance_str.trim_end_matches("Coin1"))
            } else if balance_str.ends_with("LBR") {
                ("LBR", balance_str.trim_end_matches("LBR"))
            } else {
                panic!("Unexpected currency type returned for balance")
            };
            (currency_code, Decimal::from_str(stripped_str).ok())
        })
        .collect();

    expected_balances
        .into_iter()
        .all(|(balance, currency_code)| {
            if let Some(extracted_balance) = extracted_balances_dec.get(currency_code.as_str()) {
                Decimal::from_f64(balance) == *extracted_balance
            } else {
                false
            }
        })
}

pub fn test_smoke_script(mut client_proxy: ClientProxy) {
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "1", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "3", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(7.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(4.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "2", "15", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(15.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "2"]).unwrap(),
    ));
}

/// Sets up a SmokeTestEnvironment with specified size and connects a client
/// proxy to the node_index.
pub fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    node_index: usize,
) -> (SmokeTestEnvironment, ClientProxy) {
    let mut env = SmokeTestEnvironment::new(num_nodes);
    env.validator_swarm.launch();

    let client = env.get_validator_client(node_index, None);
    (env, client)
}

/// This module provides useful functions for operating, handling and managing
/// LibraSwarm instances. It is particularly useful for working with tests that
/// require a SmokeTestEnvironment, as it provides a generic interface across
/// LibraSwarms, regardless of if the swarm is a validator swarm, validator full
/// node swarm, or a public full node swarm.
pub mod libra_swarm_utils {
    use crate::test_utils::fetch_backend_storage;
    use cli::client_proxy::ClientProxy;
    use libra_config::config::{NodeConfig, SecureBackend, WaypointConfig};
    use libra_key_manager::libra_interface::JsonRpcLibraInterface;
    use libra_operational_tool::test_helper::OperationalTool;
    use libra_secure_storage::{KVStorage, Storage};
    use libra_swarm::swarm::LibraSwarm;
    use libra_transaction_replay::LibraDebugger;
    use libra_types::{chain_id::ChainId, waypoint::Waypoint};
    use std::path::PathBuf;

    /// Returns a new client proxy connected to the given swarm at the specified
    /// node index.
    pub fn get_client_proxy(
        swarm: &LibraSwarm,
        node_index: usize,
        libra_root_key_path: &str,
        mnemonic_file_path: PathBuf,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        let port = swarm.get_client_port(node_index);

        let mnemonic_file_path = mnemonic_file_path
            .canonicalize()
            .expect("Unable to get canonical path of mnemonic_file_path")
            .to_str()
            .unwrap()
            .to_string();

        ClientProxy::new(
            ChainId::test(),
            &format!("http://localhost:{}/v1", port),
            &libra_root_key_path,
            &libra_root_key_path,
            &libra_root_key_path,
            false,
            /* faucet server */ None,
            Some(mnemonic_file_path),
            waypoint.unwrap_or_else(|| swarm.config.waypoint),
        )
        .unwrap()
    }

    /// Returns a JSON RPC based Libra Interface pointing to a node at the given
    /// node index.
    pub fn get_json_rpc_libra_interface(
        swarm: &LibraSwarm,
        node_index: usize,
    ) -> JsonRpcLibraInterface {
        let json_rpc_endpoint = format!("http://127.0.0.1:{}", swarm.get_client_port(node_index));
        JsonRpcLibraInterface::new(json_rpc_endpoint)
    }

    /// Returns a Libra Debugger pointing to a node at the given node index.
    pub fn get_libra_debugger(swarm: &LibraSwarm, node_index: usize) -> LibraDebugger {
        let (node_config, _) = load_node_config(swarm, node_index);
        let swarm_rpc_endpoint = format!("http://localhost:{}", node_config.rpc.address.port());
        LibraDebugger::json_rpc(swarm_rpc_endpoint.as_str()).unwrap()
    }

    /// Returns an operational tool pointing to a validator node at the given node index.
    pub fn get_op_tool(swarm: &LibraSwarm, node_index: usize) -> OperationalTool {
        OperationalTool::new(
            format!("http://127.0.0.1:{}", swarm.get_client_port(node_index)),
            ChainId::test(),
        )
    }

    /// Loads the nodes's storage backend identified by the node index in the given swarm.
    pub fn load_backend_storage(swarm: &LibraSwarm, node_index: usize) -> SecureBackend {
        let (node_config, _) = load_node_config(swarm, node_index);
        fetch_backend_storage(&node_config, None)
    }

    /// Loads the libra root's storage backend identified by the node index in the given swarm.
    pub fn load_libra_root_storage(swarm: &LibraSwarm, node_index: usize) -> SecureBackend {
        let (node_config, _) = load_node_config(swarm, node_index);
        fetch_backend_storage(&node_config, Some("libra_root".to_string()))
    }

    /// Loads the node config for the validator at the specified index. Also returns the node
    /// config path.
    pub fn load_node_config(swarm: &LibraSwarm, node_index: usize) -> (NodeConfig, PathBuf) {
        let node_config_path = swarm.config.config_files.get(node_index).unwrap();
        let node_config = NodeConfig::load(&node_config_path).unwrap();
        (node_config, node_config_path.clone())
    }

    /// Saves the node config for the node at the specified index in the given swarm.
    pub fn save_node_config(node_config: &mut NodeConfig, swarm: &LibraSwarm, node_index: usize) {
        let node_config_path = swarm.config.config_files.get(node_index).unwrap();
        node_config.save(node_config_path).unwrap();
    }

    pub fn insert_waypoint(node_config: &mut NodeConfig, waypoint: Waypoint) {
        let f = |backend: &SecureBackend| {
            let mut storage: Storage = backend.into();
            storage
                .set(libra_global_constants::WAYPOINT, waypoint)
                .expect("Unable to write waypoint");
            storage
                .set(libra_global_constants::GENESIS_WAYPOINT, waypoint)
                .expect("Unable to write waypoint");
        };
        let backend = &node_config.consensus.safety_rules.backend;
        f(backend);
        match &node_config.base.waypoint {
            WaypointConfig::FromStorage(backend) => {
                f(backend);
            }
            _ => panic!("unexpected waypoint from node config"),
        }
    }
}

/// Loads the node's storage backend from the given node config. If a namespace
/// is specified, the storage namespace will be overridden.
fn fetch_backend_storage(
    node_config: &NodeConfig,
    overriding_namespace: Option<String>,
) -> SecureBackend {
    if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        match storage_identity.backend.clone() {
            SecureBackend::OnDiskStorage(mut config) => {
                if let Some(namespace) = overriding_namespace {
                    config.namespace = Some(namespace);
                }
                SecureBackend::OnDiskStorage(config)
            }
            _ => unimplemented!("On-disk storage is the only backend supported in smoke tests"),
        }
    } else {
        panic!("Couldn't load identity from storage");
    }
}
