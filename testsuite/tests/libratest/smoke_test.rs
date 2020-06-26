// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use cli::client_proxy::ClientProxy;
use debug_interface::{libra_trace, NodeDebugClient};
use libra_config::config::{KeyManagerConfig, NodeConfig, OnDiskStorageConfig, SecureBackend};
use libra_crypto::{ed25519::Ed25519PrivateKey, hash::CryptoHash, PrivateKey, SigningKey, Uniform};
use libra_global_constants::{CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY};
use libra_json_rpc::views::{ScriptView, TransactionDataView};
use libra_key_manager::libra_interface::{JsonRpcLibraInterface, LibraInterface};
use libra_management::config_builder::FullnodeType;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage, Value};
use libra_swarm::swarm::{LibraNode, LibraSwarm};
use libra_temppath::TempPath;
use libra_types::{
    account_address::AccountAddress,
    account_config::{treasury_compliance_account_address, LBR_NAME},
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
    vfn_swarm: Option<LibraSwarm>,
    public_fn_swarm: Option<LibraSwarm>,
    faucet_key: (Ed25519PrivateKey, String),
    mnemonic_file: TempPath,
}

impl TestEnvironment {
    fn new(num_validators: usize) -> Self {
        ::libra_logger::Logger::new().init();
        let mut template = NodeConfig::default_for_validator();
        template.state_sync.chunk_limit = 5;

        let validator_swarm =
            LibraSwarm::configure_validator_swarm(num_validators, None, Some(template)).unwrap();

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
            vfn_swarm: None,
            public_fn_swarm: None,
            faucet_key: (key, key_path),
            mnemonic_file,
        }
    }

    fn setup_vfn_swarm(&mut self) {
        self.vfn_swarm = Some(
            LibraSwarm::configure_fn_swarm(
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::ValidatorFullnode,
            )
            .unwrap(),
        );
    }

    fn setup_public_fn_swarm(&mut self, num_nodes: usize) {
        self.public_fn_swarm = Some(
            LibraSwarm::configure_fn_swarm(
                None,
                None,
                &self.validator_swarm.config,
                FullnodeType::PublicFullnode(num_nodes),
            )
            .unwrap(),
        )
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
            &self.faucet_key.1,
            false,
            /* faucet server */ None,
            Some(mnemonic_file_path),
            waypoint.unwrap_or_else(|| self.validator_swarm.config.waypoint),
        )
        .unwrap()
    }

    fn get_client(
        &self,
        swarm: &LibraSwarm,
        node_index: usize,
        waypoint: Option<Waypoint>,
    ) -> ClientProxy {
        let port = swarm.get_client_port(node_index);
        self.get_json_rpc_client(port, waypoint)
    }

    fn get_validator_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        self.get_client(&self.validator_swarm, node_index, waypoint)
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

    fn get_vfn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        self.get_client(
            self.vfn_swarm
                .as_ref()
                .expect("Vfn swarm is not initialized"),
            node_index,
            waypoint,
        )
    }

    fn get_public_fn_client(&self, node_index: usize, waypoint: Option<Waypoint>) -> ClientProxy {
        self.get_client(
            self.public_fn_swarm
                .as_ref()
                .expect("Public fn swarm is not initialized"),
            node_index,
            waypoint,
        )
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
    env.validator_swarm.launch();
    let ac_client = env.get_validator_client(client_port_index, None);
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
    test_smoke_script(swarm.get_validator_client(0, None));
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
    client_proxy
        .mint_coins(&["mintb", "1", "10", "LBR"], true)
        .unwrap();
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
        vec![(31.0, "LBR".to_string())],
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
        .mint_coins(&["mintb", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["t", "0", "1", "1", "LBR"], false)
        .unwrap();
    let events = debug_client.get_events().expect("Failed to get events");
    let txn_node = format!("txn::{}::{}", treasury_compliance_account_address(), 1);
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
        .mint_coins(&["mb", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    let peer_to_restart = 0;
    // restart node
    env.validator_swarm.kill_node(peer_to_restart);
    assert!(env.validator_swarm.add_node(peer_to_restart, false).is_ok());
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
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
        vec![(30.0, "LBR".to_string())],
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
        .mint_coins(&["mb", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
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
    assert!(env.validator_swarm.add_node(peer_to_stop, false).is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_client(0, None);
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
        vec![(20.0, "LBR".to_string())],
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
        vec![(30.0, "LBR".to_string())],
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
        .mint_coins(&["mb", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
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
    assert!(env.validator_swarm.add_node(peer_to_stop, false).is_ok());
    // create the client for the restarted node
    let accounts = client_proxy_1.copy_all_accounts();
    let mut client_proxy_0 = env.get_validator_client(0, None);
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
        vec![(20.0, "LBR".to_string())],
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
        vec![(30.0, "LBR".to_string())],
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
        .mint_coins(&["mb", "1", "10", "LBR"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "LBR"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));

    // Test single chunk sync, chunk_size = 5
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(90.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(20.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "LBR"], true)
        .unwrap();

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart, false).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(89.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(21.0, "LBR".to_string())],
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
        vec![(21.0, "LBR".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap()
    ));
    for _ in 0..10 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1", "LBR"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart, false).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(79.0, "LBR".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap()
    ));
    assert!(compare_balances(
        vec![(31.0, "LBR".to_string())],
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
    let (receiver_address, receiver_auth_key) = client_proxy
        .get_account_address_from_parameter(
            "1bfb3b36384dabd29e38b4a0eafd9797b75141bb007cea7943f8a4714d3d784a",
        )
        .unwrap();
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
    // mint to the recipient address
    client_proxy
        .mint_coins(
            &[
                "mintb",
                &format!("{}", receiver_auth_key.unwrap()),
                "1",
                "LBR",
            ],
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
            gas_currency: p_gas_currency,
            max_gas_amount: p_max_gas_amount,
            script,
            ..
        } => {
            assert_eq!(p_sender, sender_address.to_string());
            assert_eq!(p_sequence_number, sequence_number);
            assert_eq!(p_gas_unit_price, gas_unit_price);
            assert_eq!(p_gas_currency, currency_code.to_string());
            assert_eq!(p_max_gas_amount, max_gas_amount);
            match script {
                ScriptView::PeerToPeer {
                    receiver: p_receiver,
                    amount: p_amount,
                    currency: p_currency,
                    metadata,
                    metadata_signature,
                } => {
                    assert_eq!(p_receiver, receiver_address.to_string());
                    assert_eq!(p_amount, amount);
                    assert_eq!(p_currency, currency_code.to_string());
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
    env.setup_vfn_swarm();
    env.setup_public_fn_swarm(2);
    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.public_fn_swarm.as_mut().unwrap().launch();

    // execute smoke script
    test_smoke_script(env.get_validator_client(0, None));

    // read state from full node client
    let mut validator_ac_client = env.get_validator_client(1, None);
    let mut full_node_client = env.get_vfn_client(1, None);
    let mut full_node_client_2 = env.get_public_fn_client(0, None);

    // ensure the client has up-to-date sequence number after test_smoke_script(3 minting)
    let sender_account = treasury_compliance_account_address();
    full_node_client
        .wait_for_transaction(sender_account, 3)
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
    let mut client_proxy_0 = env.get_validator_client(0, None);
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
        .wait_for_transaction(treasury_compliance_account_address(), 1)
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
    // Wait for it catches up, mint1 + mint2 => seq == 2
    client_proxy_0
        .wait_for_transaction(treasury_compliance_account_address(), 2)
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
    let mut client_with_waypoint = env.get_validator_client(0, Some(genesis_waypoint));
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
    client_with_waypoint = env.get_validator_client(1, Some(epoch_1_waypoint));
    client_with_waypoint.test_trusted_connection().unwrap();
    assert_eq!(
        client_with_waypoint.latest_epoch_change_li().unwrap(),
        epoch_1_li
    );

    // Verify that a client with the wrong waypoint is not going to be able to connect to the chain.
    let bad_li = LedgerInfo::mock_genesis(None);
    let bad_waypoint = Waypoint::new_epoch_boundary(&bad_li).unwrap();
    let mut client_with_bad_waypoint = env.get_validator_client(1, Some(bad_waypoint));
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
#[ignore]
fn test_key_manager_consensus_rotation() {
    // Create and launch a local validator swarm of 2 nodes.
    let mut env = TestEnvironment::new(2);
    env.validator_swarm.launch();

    // Create a node config for the key manager by extracting the first node config in the swarm.
    // TODO(joshlind): see if we can refactor TestEnvironment to clean this up.
    let node_config_path = env.validator_swarm.config.config_files.get(0).unwrap();
    let node_config = NodeConfig::load(&node_config_path).unwrap();
    let json_rpc_endpoint = format!("http://127.0.0.1:{}", node_config.rpc.address.port());

    let mut key_manager_config = KeyManagerConfig::default();
    key_manager_config.json_rpc_endpoint = json_rpc_endpoint.clone();
    key_manager_config.rotation_period_secs = 10;
    key_manager_config.sleep_period_secs = 10;
    let mut on_disk_storage_config = OnDiskStorageConfig::default();
    on_disk_storage_config.path =
        node_config_path.with_file_name(on_disk_storage_config.path.clone());
    key_manager_config.secure_backend = SecureBackend::OnDiskStorage(on_disk_storage_config);

    // Save the key manager config to disk
    let key_manager_config_path = node_config_path.with_file_name("key_manager.config.toml");
    key_manager_config.save(&key_manager_config_path).unwrap();

    // Bootstrap secure storage by initializing the keys required by the key manager.
    // TODO(joshlind): set these keys using config manager when initialization is supported.
    let mut storage: Storage = (&key_manager_config.secure_backend).try_into().unwrap();
    let operator_private = node_config
        .test
        .unwrap()
        .operator_keypair
        .unwrap()
        .take_private()
        .unwrap();
    let operator_account =
        libra_types::account_address::from_public_key(&operator_private.public_key());
    storage
        .set(
            OPERATOR_ACCOUNT,
            Value::String(operator_account.to_string()),
        )
        .unwrap();
    storage
        .set(OPERATOR_KEY, Value::Ed25519PrivateKey(operator_private))
        .unwrap();

    // Create a json-rpc connection to the blockchain and verify storage matches the on-chain state.
    let libra_interface = JsonRpcLibraInterface::new(json_rpc_endpoint);
    let account = node_config.validator_network.unwrap().peer_id();
    let current_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&current_consensus, validator_info.consensus_public_key());

    // Spawn the key manager and sleep until a rotation occurs.
    // TODO(joshlind): support a dedicated key manager log (instead of just printing on failure).
    let mut command = Command::new(workspace_builder::get_bin(KEY_MANAGER_BIN));
    command
        .current_dir(workspace_builder::workspace_root())
        .arg(key_manager_config_path)
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
