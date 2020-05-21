// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use cli::client_proxy::ClientProxy;
use debug_interface::{libra_trace, node_debug_service::parse_events, NodeDebugClient};
use libra_config::config::{NodeConfig, OnDiskStorageConfig, RoleType, SecureBackend, TestConfig};
use libra_crypto::{ed25519::Ed25519PrivateKey, hash::CryptoHash, PrivateKey, SigningKey, Uniform};
use libra_global_constants::{CONSENSUS_KEY, OPERATOR_KEY};
use libra_json_rpc::views::{ScriptView, TransactionDataView};
use libra_key_manager::libra_interface::{JsonRpcLibraInterface, LibraInterface};
use libra_logger::prelude::*;
use libra_secure_storage::{Policy, Storage, Value};
use libra_swarm::swarm::{LibraNode, LibraSwarm};
use libra_temppath::TempPath;
use libra_types::{
    account_address::AccountAddress,
    account_config::{association_address, LBR_NAME},
    ledger_info::LedgerInfo,
    transaction::authenticator::AuthenticationKey,
    waypoint::Waypoint,
};
use num_traits::cast::FromPrimitive;
use rust_decimal::Decimal;
use std::{
    collections::BTreeMap,
    convert::TryInto,
    fs,
    io::{Result, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
    thread::sleep,
    time::Duration,
};

const KEY_MANAGER_BIN: &str = "libra-key-manager";

struct TestEnvironment {
    validator_swarm: LibraSwarm,
    full_node_swarm: Option<LibraSwarm>,
    faucet_key: (Ed25519PrivateKey, String),
    mnemonic_file: TempPath,
}

impl TestEnvironment {
    fn new(num_validators: usize) -> Self {
        ::libra_logger::Logger::new().init();
        let mut template = NodeConfig::default();
        template.test = Some(TestConfig::open_module());
        template.state_sync.chunk_limit = 2;
        template.consensus.safety_rules.backend =
            SecureBackend::OnDiskStorage(OnDiskStorageConfig::default());

        let validator_swarm = LibraSwarm::configure_swarm(
            num_validators,
            RoleType::Validator,
            None,
            Some(template),
            None,
        )
        .unwrap();

        let mnemonic_file = libra_temppath::TempPath::new();
        mnemonic_file
            .create_as_file()
            .expect("could not create temporary mnemonic_file_path");

        let key = generate_key::load_key(&validator_swarm.config.faucet_key_path);
        let key_path = validator_swarm
            .config
            .faucet_key_path
            .to_str()
            .expect("Unable to read faucet path")
            .to_string();

        Self {
            validator_swarm,
            full_node_swarm: None,
            faucet_key: (key, key_path),
            mnemonic_file,
        }
    }

    fn setup_full_node_swarm(&mut self, num_full_nodes: usize) {
        self.full_node_swarm = Some(
            LibraSwarm::configure_swarm(
                num_full_nodes,
                RoleType::FullNode,
                None,
                None,
                Some(String::from(
                    self.validator_swarm
                        .dir
                        .as_ref()
                        .join("0")
                        .to_str()
                        .expect("Failed to convert std::fs::Path to String"),
                )),
            )
            .unwrap(),
        );
    }

    fn launch_swarm(&mut self, role: RoleType) {
        let swarm = match role {
            RoleType::Validator => &mut self.validator_swarm,
            RoleType::FullNode => self.full_node_swarm.as_mut().unwrap(),
        };
        let num_attempts = 5;
        for _ in 0..num_attempts {
            match swarm.launch_attempt(role, false) {
                Ok(_) => {
                    return;
                }
                Err(err) => {
                    error!("Error launching swarm: {}", err);
                }
            }
        }
        panic!("Max out {} attempts to launch test swarm", num_attempts);
    }

    fn get_json_rpc_client(&self, port: u16, waypoint: Option<Waypoint>) -> ClientProxy {
        let mnemonic_file_path = self
            .mnemonic_file
            .path()
            .to_path_buf()
            .canonicalize()
            .expect("Unable to get canonical path of mnemonic_file_path")
            .to_str()
            .unwrap()
            .to_string();

        ClientProxy::new(
            &format!("http://localhost:{}", port),
            &self.faucet_key.1,
            false,
            /* faucet server */ None,
            Some(mnemonic_file_path),
            waypoint.unwrap_or_else(|| self.validator_swarm.config.waypoint),
        )
        .unwrap()
    }

    fn get_validator_ac_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        let port = self.validator_swarm.get_client_port(node_index);
        self.get_json_rpc_client(port, waypoint)
    }

    fn get_validator_debug_interface_client(&self, node_index: usize) -> NodeDebugClient {
        let port = self.validator_swarm.get_validators_debug_ports()[node_index];
        NodeDebugClient::new("localhost", port)
    }

    #[allow(dead_code)]
    fn get_validator_debug_interface_clients(&self) -> Vec<NodeDebugClient> {
        self.validator_swarm
            .get_validators_debug_ports()
            .iter()
            .map(|port| NodeDebugClient::new("localhost", *port))
            .collect()
    }

    fn get_full_node_ac_client(
        &self,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        match &self.full_node_swarm {
            Some(swarm) => {
                let port = swarm.get_client_port(node_index);
                self.get_json_rpc_client(port, waypoint)
            }
            None => {
                panic!("Full Node swarm is not initialized");
            }
        }
    }

    fn get_validator(&self, node_index: usize) -> Option<&LibraNode> {
        self.validator_swarm.get_validator(node_index)
    }
}

fn copy_file_with_sender_address(file_path: &Path, sender: AccountAddress) -> Result<PathBuf> {
    let tmp_source_path = TempPath::new().as_ref().with_extension("move");
    let mut tmp_source_file = std::fs::File::create(tmp_source_path.clone())?;
    let mut code = fs::read_to_string(file_path)?;
    code = code.replace("{{sender}}", &format!("0x{}", sender));
    writeln!(tmp_source_file, "{}", code)?;
    Ok(tmp_source_path)
}

fn compare_balances(
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
            } else if balance_str.ends_with("Coin2") {
                ("Coin2", balance_str.trim_end_matches("Coin2"))
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

fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    client_port_index: usize,
) -> (TestEnvironment, ClientProxy) {
    let mut env = TestEnvironment::new(num_nodes);
    env.launch_swarm(RoleType::Validator);
    let ac_client = env.get_validator_ac_client(client_port_index, None);
    (env, ac_client)
}

fn test_smoke_script(mut client_proxy: ClientProxy) {
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "1", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "3", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(7.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(4.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "2", "15", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(15.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "2"]).unwrap()
    ));
}

#[test]
fn test_execute_custom_module_and_script() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "50", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(50.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));

    let recipient_address = client_proxy.create_next_account(false).unwrap().address;
    client_proxy
        .mint_coins(&["mintb", "1", "1", "LBR"], true)
        .unwrap();

    let (sender_account, _) = client_proxy
        .get_account_address_from_parameter("0")
        .unwrap();

    // Get the path to the Move stdlib sources
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();

    // Make a copy of module.move with "{{sender}}" substituted.
    let module_path = workspace_builder::workspace_root()
        .join("testsuite/tests/libratest/dev_modules/module.move");
    let copied_module_path = copy_file_with_sender_address(&module_path, sender_account).unwrap();
    let unwrapped_module_path = copied_module_path.to_str().unwrap();

    // Compile and publish that module.
    let module_params = &["compile", "0", unwrapped_module_path, unwrapped_stdlib_dir];
    let mut module_compiled_paths = client_proxy.compile_program(module_params).unwrap();
    let module_compiled_path = if module_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        module_compiled_paths.pop().unwrap()
    };
    client_proxy
        .publish_module(&["publish", "0", &module_compiled_path[..]])
        .unwrap();

    // Make a copy of script.move with "{{sender}}" substituted.
    let script_path = workspace_builder::workspace_root()
        .join("testsuite/tests/libratest/dev_modules/script.move");
    let copied_script_path = copy_file_with_sender_address(&script_path, sender_account).unwrap();
    let unwrapped_script_path = copied_script_path.to_str().unwrap();

    // Compile and execute the script.
    let script_params = &[
        "compile",
        "0",
        unwrapped_script_path,
        unwrapped_module_path,
        unwrapped_stdlib_dir,
    ];
    let mut script_compiled_paths = client_proxy.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };
    let formatted_recipient_address = format!("0x{}", recipient_address);
    client_proxy
        .execute_script(&[
            "execute",
            "0",
            &script_compiled_path[..],
            &formatted_recipient_address[..],
            "10",
        ])
        .unwrap();

    assert!(compare_balances(
        vec![(49.999_990, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(1.000_010, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn smoke_test_single_node() {
    let (_swarm, client_proxy) = setup_swarm_and_client_proxy(1, 0);
    test_smoke_script(client_proxy);
}

// Test if we commit not only user transactions but also block metadata transactions,
// assert committed version > # of user transactions
#[test]
fn smoke_test_single_node_block_metadata() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    // just need an address to get the latest version
    let address = AccountAddress::from_hex_literal("0xA550C18").unwrap();
    // this script does 4 transactions
    test_smoke_script(swarm.get_validator_ac_client(0, None));
    let (_state, version) = client_proxy
        .get_latest_account_state(&["q", &address.to_string()])
        .unwrap();
    assert!(
        version > 4,
        "BlockMetadata txn not produced, current version: {}",
        version
    );
}

#[test]
fn smoke_test_multi_node() {
    let (_swarm, client_proxy) = setup_swarm_and_client_proxy(4, 0);
    test_smoke_script(client_proxy);
}

#[test]
fn test_concurrent_transfers_single_node() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    for _ in 0..20 {
        client_proxy
            .transfer_coins(&["t", "0", "1", "1", "LBR"], false)
            .unwrap();
    }
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(79.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(21.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn test_trace() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    let mut debug_client = swarm.get_validator_debug_interface_client(0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .transfer_coins(&["t", "0", "1", "1", "LBR"], false)
        .unwrap();
    let events = parse_events(
        debug_client
            .get_events()
            .expect("Failed to get events")
            .events,
    );
    let txn_node = format!("txn::{}::{}", association_address(), 1);
    println!("Tracing {}", txn_node);
    libra_trace::trace_node(&events[..], &txn_node);
}

#[test]
fn test_basic_fault_tolerance() {
    // A configuration with 4 validators should tolerate single node failure.
    let (mut env, client_proxy) = setup_swarm_and_client_proxy(4, 1);
    // kill the first validator
    env.validator_swarm.kill_node(0);
    // run the script for the smoke test by submitting requests to the second validator
    test_smoke_script(client_proxy);
}

#[test]
fn test_basic_restartability() {
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    let peer_to_restart = 0;
    // restart node
    env.validator_swarm.kill_node(peer_to_restart);
    assert!(env
        .validator_swarm
        .add_node(peer_to_restart, RoleType::Validator, false)
        .is_ok());
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn test_startup_sync_state() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap()
    ));
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let node_config = NodeConfig::load(
        env.validator_swarm
            .config
            .config_files
            .get(peer_to_stop)
            .unwrap(),
    )
    .unwrap();
    // TODO Remove hardcoded path to state db
    let state_db_path = node_config.storage.dir().join("libradb");
    // Verify that state_db_path exists and
    // we are not deleting a non-existent directory
    assert!(state_db_path.as_path().exists());
    // Delete the state db to simulate state db lagging
    // behind consensus db and forcing a state sync
    // during a node startup
    fs::remove_dir_all(state_db_path).unwrap();
    assert!(env
        .validator_swarm
        .add_node(peer_to_stop, RoleType::Validator, false)
        .is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_ac_client(0, None);
    let sender_address = accounts[0].address;
    client_proxy_0.set_accounts(accounts);
    client_proxy_0
        .wait_for_transaction(sender_address, 1)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn test_startup_sync_state_with_empty_consensus_db() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap()
    ));
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let node_config = NodeConfig::load(
        env.validator_swarm
            .config
            .config_files
            .get(peer_to_stop)
            .unwrap(),
    )
    .unwrap();
    let consensus_db_path = node_config.storage.dir().join("consensusdb");
    // Verify that consensus db exists and
    // we are not deleting a non-existent directory
    assert!(consensus_db_path.as_path().exists());
    // Delete the consensus db to simulate consensus db is nuked
    fs::remove_dir_all(consensus_db_path).unwrap();
    assert!(env
        .validator_swarm
        .add_node(peer_to_stop, RoleType::Validator, false)
        .is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_ac_client(0, None);
    let sender_address = accounts[0].address;
    client_proxy_0.set_accounts(accounts);
    client_proxy_0
        .wait_for_transaction(sender_address, 1)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn test_basic_state_synchronization() {
    // - Start a swarm of 5 nodes (3 nodes forming a QC).
    // - Kill one node and continue submitting transactions to the others.
    // - Restart the node
    // - Wait for all the nodes to catch up
    // - Verify that the restarted node has synced up with the submitted transactions.
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(5, 1);
    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mb", "0", "100", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));

    // Test single chunk sync, chunk_size = 2
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "LBR"], true)
        .unwrap();

    // Reconnect and synchronize the state
    assert!(env
        .validator_swarm
        .add_node(node_to_restart, RoleType::Validator, false)
        .is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_ac_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(89.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(11.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap()
    ));

    // Test multiple chunk sync
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(89.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(11.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    for _ in 0..10 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1", "LBR"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(env
        .validator_swarm
        .add_node(node_to_restart, RoleType::Validator, false)
        .is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_ac_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(79.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(21.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap()
    ));
}

#[test]
fn test_external_transaction_signer() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);

    // generate key pair
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();

    // create transfer parameters
    let sender_auth_key = AuthenticationKey::ed25519(&public_key);
    let sender_address = sender_auth_key.derived_address();
    let (receiver_address, receiver_auth_key_opt) = client_proxy
        .get_account_address_from_parameter(
            "1bfb3b36384dabd29e38b4a0eafd9797b75141bb007cea7943f8a4714d3d784a",
        )
        .unwrap();
    assert!(
        receiver_auth_key_opt.is_some(),
        "Failed to look up receiver auth key from parameter"
    );
    let receiver_auth_key = receiver_auth_key_opt.unwrap();
    let amount = 1_000_000;
    let gas_unit_price = 1;
    let max_gas_amount = 1_000_000;

    // mint to the sender address
    client_proxy
        .mint_coins(
            &["mintb", &format!("{}", sender_auth_key), "10", "LBR"],
            true,
        )
        .unwrap();

    // prepare transfer transaction
    let sequence_number = client_proxy
        .get_sequence_number(&["sequence", &format!("{}", sender_address)])
        .unwrap();

    let currency_code = LBR_NAME;

    let unsigned_txn = client_proxy
        .prepare_transfer_coins(
            sender_address,
            sequence_number,
            receiver_address,
            receiver_auth_key.prefix().to_vec(),
            amount,
            currency_code.to_owned(),
            Some(gas_unit_price),
            Some(max_gas_amount),
            Some(currency_code.to_owned()),
        )
        .unwrap();

    assert_eq!(unsigned_txn.sender(), sender_address);

    // sign the transaction with the private key
    let signature = private_key.sign_message(&unsigned_txn.hash());

    // submit the transaction
    let submit_txn_result =
        client_proxy.submit_signed_transaction(unsigned_txn, public_key, signature);

    assert!(submit_txn_result.is_ok());

    // query the transaction and check it contains the same values as requested
    let txn = client_proxy
        .get_committed_txn_by_acc_seq(&[
            "txn_acc_seq",
            &format!("{}", sender_address),
            &sequence_number.to_string(),
            "false",
        ])
        .unwrap()
        .unwrap();

    match txn.transaction {
        TransactionDataView::UserTransaction {
            sender: p_sender,
            sequence_number: p_sequence_number,
            gas_unit_price: p_gas_unit_price,
            max_gas_amount: p_max_gas_amount,
            script,
            ..
        } => {
            assert_eq!(p_sender, sender_address.to_string());
            assert_eq!(p_sequence_number, sequence_number);
            assert_eq!(p_gas_unit_price, gas_unit_price);
            assert_eq!(p_max_gas_amount, max_gas_amount);
            match script {
                ScriptView::PeerToPeer {
                    receiver: p_receiver,
                    amount: p_amount,
                    auth_key_prefix,
                    metadata,
                    metadata_signature,
                } => {
                    assert_eq!(p_receiver, receiver_address.to_string());
                    assert_eq!(p_amount, amount);
                    assert_eq!(
                        auth_key_prefix
                            .into_bytes()
                            .expect("failed to turn key to bytes"),
                        receiver_auth_key.prefix()
                    );
                    assert_eq!(
                        metadata
                            .into_bytes()
                            .expect("failed to turn metadata to bytes"),
                        Vec::<u8>::new()
                    );
                    assert_eq!(
                        metadata_signature
                            .into_bytes()
                            .expect("failed to turn metadata_signature to bytes"),
                        Vec::<u8>::new()
                    );
                }
                _ => panic!("Expected peer-to-peer script for user txn"),
            }
        }
        _ => panic!("Query should get user transaction"),
    }
}

#[test]
fn test_full_node_basic_flow() {
    // launch environment of 4 validator nodes and 2 full nodes
    let mut env = TestEnvironment::new(4);
    env.setup_full_node_swarm(2);
    env.launch_swarm(RoleType::Validator);
    env.launch_swarm(RoleType::FullNode);

    // execute smoke script
    test_smoke_script(env.get_validator_ac_client(0, None));

    // read state from full node client
    let mut validator_ac_client = env.get_validator_ac_client(1, None);
    let mut full_node_client = env.get_full_node_ac_client(1, None);
    let mut full_node_client_2 = env.get_full_node_ac_client(0, None);

    // ensure the client has up-to-date sequence number after test_smoke_script(3 minting)
    let sender_account = association_address();
    full_node_client
        .wait_for_transaction(sender_account, 4)
        .unwrap();
    for idx in 0..3 {
        validator_ac_client.create_next_account(false).unwrap();
        full_node_client.create_next_account(false).unwrap();
        full_node_client_2.create_next_account(false).unwrap();
        assert_eq!(
            validator_ac_client
                .get_balances(&["b", &idx.to_string()])
                .unwrap(),
            full_node_client
                .get_balances(&["b", &idx.to_string()])
                .unwrap(),
        );
    }

    // mint from full node and check both validator and full node have correct balance
    let account3 = validator_ac_client
        .create_next_account(false)
        .unwrap()
        .address;
    full_node_client.create_next_account(false).unwrap();
    full_node_client_2.create_next_account(false).unwrap();

    let sequence_reset = format!("sequence {} true", sender_account);
    let sequence_reset_command: Vec<_> = sequence_reset.split(' ').collect();
    full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .mint_coins(&["mintb", "3", "10", "LBR"], true)
        .expect("Fail to mint!");

    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        full_node_client.get_balances(&["b", "3"]).unwrap()
    ));
    let sequence = full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    validator_ac_client
        .wait_for_transaction(sender_account, sequence)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        validator_ac_client.get_balances(&["b", "3"]).unwrap()
    ));

    // reset sequence number for sender account
    validator_ac_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();

    // mint from validator and check both nodes have correct balance
    validator_ac_client.create_next_account(false).unwrap();
    full_node_client.create_next_account(false).unwrap();
    full_node_client_2.create_next_account(false).unwrap();

    validator_ac_client
        .mint_coins(&["mintb", "4", "10", "LBR"], true)
        .unwrap();
    let sequence = validator_ac_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .wait_for_transaction(sender_account, sequence)
        .unwrap();

    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        validator_ac_client.get_balances(&["b", "4"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        full_node_client.get_balances(&["b", "4"]).unwrap()
    ));

    // minting again on validator doesn't cause error since client sequence has been updated
    validator_ac_client
        .mint_coins(&["mintb", "4", "10", "LBR"], true)
        .unwrap();

    // test transferring balance from 0 to 1 through full node proxy
    full_node_client
        .transfer_coins(&["tb", "3", "4", "10", "LBR"], true)
        .unwrap();

    assert!(compare_balances(
        vec![(0.0, "LBR".to_string())],
        full_node_client.get_balances(&["b", "3"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(30.0, "LBR".to_string())],
        validator_ac_client.get_balances(&["b", "4"]).unwrap()
    ));

    let sequence = validator_ac_client
        .get_sequence_number(&["sequence", &format!("{}", account3), "true"])
        .unwrap();
    full_node_client_2
        .wait_for_transaction(account3, sequence)
        .unwrap();
    assert!(compare_balances(
        vec![(0.0, "LBR".to_string())],
        full_node_client_2.get_balances(&["b", "3"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(30.0, "LBR".to_string())],
        full_node_client_2.get_balances(&["b", "4"]).unwrap()
    ));
}

#[test]
fn test_e2e_reconfiguration() {
    let (env, mut client_proxy_1) = setup_swarm_and_client_proxy(3, 1);
    // the client connected to the removed validator
    let mut client_proxy_0 = env.get_validator_ac_client(0, None);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_0.set_accounts(client_proxy_1.copy_all_accounts());
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    // wait for the mint txn in node 0
    client_proxy_0
        .wait_for_transaction(association_address(), 2)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    let peer_id = env
        .get_validator(0)
        .unwrap()
        .validator_peer_id()
        .unwrap()
        .to_string();
    client_proxy_1
        .remove_validator(&["remove_validator", &peer_id], true)
        .unwrap();
    // mint another 10 coins after remove node 0
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    // client connected to removed validator can not see the update
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
    // Add the node back
    client_proxy_1
        .add_validator(&["add_validator", &peer_id], true)
        .unwrap();
    // Wait for it catches up, mint1 + remove + mint2 + add => seq == 5
    client_proxy_0
        .wait_for_transaction(association_address(), 5)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap()
    ));
}

#[test]
fn test_e2e_modify_publishing_option() {
    let (_env, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();

    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    let script_path = workspace_builder::workspace_root()
        .join("testsuite/tests/libratest/dev_modules/test_script.move");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();
    let script_params = &["compile", "0", unwrapped_script_path, unwrapped_stdlib_dir];
    let mut script_compiled_paths = client_proxy.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };

    // Initially publishing option was set to CustomScript, this transaction should be executed.
    client_proxy
        .execute_script(&["execute", "0", &script_compiled_path[..], "10", "0x0"])
        .unwrap();

    // Make sure the transaction is executed by checking if the sequence is bumped to 1.
    assert_eq!(
        client_proxy
            .get_sequence_number(&["sequence", "0", "true"])
            .unwrap(),
        1
    );

    client_proxy
        .disable_custom_script(&["disable_custom_script"], true)
        .unwrap();

    // mint another 10 coins after restart
    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));

    // Now that publishing option was changed to locked, this transaction will be rejected.
    assert!(format!(
        "{:?}",
        client_proxy
            .execute_script(&["execute", "0", &script_compiled_path[..], "10", "0x0"])
            .unwrap_err()
            .root_cause()
    )
    .contains("UNKNOWN_SCRIPT"));

    assert_eq!(
        client_proxy
            .get_sequence_number(&["sequence", "0", "true"])
            .unwrap(),
        1
    );
}

#[test]
fn test_client_waypoints() {
    let (env, mut client_proxy) = setup_swarm_and_client_proxy(3, 1);
    // Make sure some txns are committed
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();

    // Create the waypoint for the initial epoch
    let genesis_li = client_proxy
        .latest_epoch_change_li()
        .expect("Failed to retrieve genesis LedgerInfo");
    assert_eq!(genesis_li.ledger_info().epoch(), 0);
    let genesis_waypoint = Waypoint::new_epoch_boundary(genesis_li.ledger_info())
        .expect("Failed to generate waypoint from genesis LI");

    // Start another client with the genesis waypoint and make sure it successfully connects
    let mut client_with_waypoint = env.get_validator_ac_client(0, Some(genesis_waypoint));
    client_with_waypoint.test_trusted_connection().unwrap();
    assert_eq!(
        client_with_waypoint.latest_epoch_change_li().unwrap(),
        genesis_li
    );

    // Start next epoch
    let peer_id = env
        .get_validator(0)
        .unwrap()
        .validator_peer_id()
        .unwrap()
        .to_string();
    client_proxy
        .remove_validator(&["remove_validator", &peer_id], true)
        .unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    let epoch_1_li = client_proxy
        .latest_epoch_change_li()
        .expect("Failed to retrieve end of epoch 1 LedgerInfo");

    assert_eq!(epoch_1_li.ledger_info().epoch(), 1);
    let epoch_1_waypoint = Waypoint::new_epoch_boundary(epoch_1_li.ledger_info())
        .expect("Failed to generate waypoint from end of epoch 1");

    // Start a client with the waypoint for end of epoch 1 and make sure it successfully connects
    client_with_waypoint = env.get_validator_ac_client(1, Some(epoch_1_waypoint));
    client_with_waypoint.test_trusted_connection().unwrap();
    assert_eq!(
        client_with_waypoint.latest_epoch_change_li().unwrap(),
        epoch_1_li
    );

    // Verify that a client with the wrong waypoint is not going to be able to connect to the chain.
    let bad_li = LedgerInfo::mock_genesis(None);
    let bad_waypoint = Waypoint::new_epoch_boundary(&bad_li).unwrap();
    let mut client_with_bad_waypoint = env.get_validator_ac_client(1, Some(bad_waypoint));
    assert!(client_with_bad_waypoint.test_trusted_connection().is_err());
}

#[test]
fn test_malformed_script() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "LBR"], true)
        .unwrap();

    let script_path = workspace_builder::workspace_root()
        .join("testsuite/tests/libratest/dev_modules/test_script.move");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let stdlib_source_dir = workspace_builder::workspace_root().join("language/stdlib/modules");
    let unwrapped_stdlib_dir = stdlib_source_dir.to_str().unwrap();
    let script_params = &["compile", "0", unwrapped_script_path, unwrapped_stdlib_dir];
    let mut script_compiled_paths = client_proxy.compile_program(script_params).unwrap();
    let script_compiled_path = if script_compiled_paths.len() != 1 {
        panic!("compiler output has more than one file")
    } else {
        script_compiled_paths.pop().unwrap()
    };

    // the script expects two arguments. Passing only one in the test, which will cause a failure.
    client_proxy
        .execute_script(&["execute", "0", &script_compiled_path[..], "10"])
        .expect_err("malformed script did not fail!");

    // Previous transaction should not choke the system.
    client_proxy
        .mint_coins(&["mintb", "0", "10", "LBR"], true)
        .unwrap();
}

#[test]
fn test_key_manager_consensus_rotation() {
    // Create and launch a local validator swarm of 2 nodes.
    let mut swarm = TestEnvironment::new(2);
    swarm.launch_swarm(RoleType::Validator);

    // Create a node config for the key manager by modifying the first node config in the swarm.
    // TODO(joshlind): see if we can refactor TestEnvironment to clean this up.
    let config_path = swarm.validator_swarm.config.config_files.get(0).unwrap();
    let mut config = NodeConfig::load(&config_path).unwrap();
    let backend = config.consensus.safety_rules.backend.clone();
    let json_rpc_endpoint = format!("http://127.0.0.1:{}", config.rpc.address.port());
    config.secure.key_manager.secure_backend = backend.clone();
    config.secure.key_manager.json_rpc_endpoint = json_rpc_endpoint.clone();
    config.secure.key_manager.rotation_period_secs = 10;
    config.secure.key_manager.sleep_period_secs = 10;
    config.save(config_path).unwrap();

    // Bootstrap secure storage by initializing the keys required by the key manager.
    // TODO(joshlind): set these keys using config manager when initialization is supported.
    let mut storage: Box<dyn Storage> = (&backend).try_into().unwrap();
    storage
        .create_key("consensus_previous", &Policy::default())
        .unwrap();
    let operator_private = config
        .test
        .unwrap()
        .operator_keypair
        .unwrap()
        .take_private()
        .unwrap();
    storage
        .set(OPERATOR_KEY, Value::Ed25519PrivateKey(operator_private))
        .unwrap();

    // Create a json-rpc connection to the blockchain and verify storage matches the on-chain state.
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_endpoint);
    let account = config.validator_network.clone().unwrap().peer_id;
    let current_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&current_consensus, validator_info.consensus_public_key());

    // Spawn the key manager and sleep until a rotation occurs.
    // TODO(joshlind): support a dedicated key manager log (instead of just printing on failure).
    let mut command = Command::new(workspace_builder::get_bin(KEY_MANAGER_BIN));
    command
        .current_dir(workspace_builder::workspace_root())
        .arg(config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let key_manager = command.spawn().unwrap();
    sleep(Duration::from_secs(20));

    // Verify the consensus key has been rotated in secure storage and on-chain.
    let rotated_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&rotated_consensus, validator_info.consensus_public_key());
    assert_ne!(current_consensus, rotated_consensus);

    // Cause a failure (e.g., wipe storage) and verify the key manager exits with an error status.
    storage.reset_and_clear().unwrap();
    let output = key_manager.wait_with_output().unwrap();
    if output.status.success() {
        panic!(
            "Key manager did not return an error as expected! Printing key manager output: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
