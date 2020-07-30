// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use cli::client_proxy::ClientProxy;
use debug_interface::NodeDebugClient;
use libra_config::config::{Identity, KeyManagerConfig, NodeConfig, SecureBackend};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, SigningKey, Uniform};
use libra_genesis_tool::config_builder::FullnodeType;
use libra_global_constants::CONSENSUS_KEY;
use libra_json_rpc::views::{ScriptView, TransactionDataView};
use libra_key_manager::{
    self,
    libra_interface::{JsonRpcLibraInterface, LibraInterface},
};
use libra_operational_tool::test_helper::OperationalTool;
use libra_secure_json_rpc::VMStatusView;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use libra_swarm::swarm::{LibraNode, LibraSwarm};
use libra_temppath::TempPath;
use libra_trace::trace::trace_node;
use libra_types::{
    account_address::AccountAddress,
    account_config::{libra_root_address, testnet_dd_account_address, COIN1_NAME},
    chain_id::ChainId,
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
    io::{self, Write},
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
    libra_root_key: (Ed25519PrivateKey, String),
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

        let key = generate_key::load_key(&validator_swarm.config.libra_root_key_path);
        let key_path = validator_swarm
            .config
            .libra_root_key_path
            .to_str()
            .expect("Unable to read faucet path")
            .to_string();

        Self {
            validator_swarm,
            vfn_swarm: None,
            public_fn_swarm: None,
            libra_root_key: (key, key_path),
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
            ChainId::test(),
            &format!("http://localhost:{}/v1", port),
            &self.libra_root_key.1,
            &self.libra_root_key.1,
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

    fn get_op_tool(&self, node_index: usize) -> OperationalTool {
        OperationalTool::new(
            format!(
                "http://127.0.0.1:{}",
                self.validator_swarm.get_client_port(node_index)
            ),
            ChainId::test(),
        )
    }
}

fn copy_file_with_sender_address(file_path: &Path, sender: AccountAddress) -> io::Result<PathBuf> {
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

#[test]
fn test_execute_custom_module_and_script() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "50", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(50.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));

    let recipient_address = client_proxy.create_next_account(false).unwrap().address;
    client_proxy
        .mint_coins(&["mintb", "1", "1", "Coin1"], true)
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
        vec![(49.999_990, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(1.000_010, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
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
    let (_account, version) = client_proxy
        .get_latest_account(&["q", &address.to_string()])
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
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "10", "Coin1"], true)
        .unwrap();
    for _ in 0..20 {
        client_proxy
            .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
            .unwrap();
    }
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(79.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(31.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_trace() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    let mut debug_client = swarm.get_validator_debug_interface_client(0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
        .unwrap();
    let events = debug_client.get_events().expect("Failed to get events");
    let txn_node = format!("txn::{}::{}", testnet_dd_account_address(), 1);
    println!("Tracing {}", txn_node);
    trace_node(&events[..], &txn_node);
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
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    let peer_to_restart = 0;
    // restart node
    env.validator_swarm.kill_node(peer_to_restart);
    assert!(env.validator_swarm.add_node(peer_to_restart, false).is_ok());
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap(),
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
        vec![(90.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_startup_sync_state_with_empty_consensus_db() {
    let (mut env, mut client_proxy_1) = setup_swarm_and_client_proxy(4, 1);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_1
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "1"]).unwrap(),
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
        vec![(90.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy_1
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy_0
        .wait_for_transaction(sender_address, 2)
        .unwrap();
    assert!(compare_balances(
        vec![(80.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "1"]).unwrap(),
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
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    client_proxy
        .mint_coins(&["mb", "1", "10", "Coin1"], true)
        .unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test single chunk sync, chunk_size = 5
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(90.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart, false).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_client(node_to_restart, None);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert!(compare_balances(
        vec![(89.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap(),
    ));

    // Test multiple chunk sync
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert!(compare_balances(
        vec![(89.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(21.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "1"]).unwrap(),
    ));
    for _ in 0..10 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
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
        vec![(79.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(31.0, "Coin1".to_string())],
        client_proxy2.get_balances(&["b", "1"]).unwrap(),
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
            &["mintb", &format!("{}", sender_auth_key), "10", "Coin1"],
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
                "Coin1",
            ],
            true,
        )
        .unwrap();

    // prepare transfer transaction
    let sequence_number = client_proxy
        .get_sequence_number(&["sequence", &format!("{}", sender_address)])
        .unwrap();

    let currency_code = COIN1_NAME;

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
    let signature = private_key.sign(&unsigned_txn);

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
    //let sender_account = treasury_compliance_account_address();
    let sender_account = testnet_dd_account_address();
    let creation_account = libra_root_address();
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
    let creation_sequence_reset = format!("sequence {} true", creation_account);
    let sequence_reset_command: Vec<_> = sequence_reset.split(' ').collect();
    let creation_sequence_reset_command: Vec<_> = creation_sequence_reset.split(' ').collect();
    full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    full_node_client
        .mint_coins(&["mintb", "3", "10", "Coin1"], true)
        .expect("Fail to mint!");

    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        full_node_client.get_balances(&["b", "3"]).unwrap(),
    ));
    let sequence = full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    validator_ac_client
        .wait_for_transaction(sender_account, sequence)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        validator_ac_client.get_balances(&["b", "3"]).unwrap(),
    ));

    // reset sequence number for sender account
    validator_ac_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    validator_ac_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    full_node_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();

    // mint from validator and check both nodes have correct balance
    validator_ac_client.create_next_account(false).unwrap();
    full_node_client.create_next_account(false).unwrap();
    full_node_client_2.create_next_account(false).unwrap();

    validator_ac_client
        .mint_coins(&["mintb", "4", "10", "Coin1"], true)
        .unwrap();
    let sequence = validator_ac_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    full_node_client
        .wait_for_transaction(sender_account, sequence)
        .unwrap();

    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        validator_ac_client.get_balances(&["b", "4"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        full_node_client.get_balances(&["b", "4"]).unwrap(),
    ));

    // minting again on validator doesn't cause error since client sequence has been updated
    validator_ac_client
        .mint_coins(&["mintb", "4", "10", "Coin1"], true)
        .unwrap();

    // test transferring balance from 0 to 1 through full node proxy
    full_node_client
        .transfer_coins(&["tb", "3", "4", "10", "Coin1"], true)
        .unwrap();

    assert!(compare_balances(
        vec![(0.0, "Coin1".to_string())],
        full_node_client.get_balances(&["b", "3"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        validator_ac_client.get_balances(&["b", "4"]).unwrap(),
    ));

    let sequence = validator_ac_client
        .get_sequence_number(&["sequence", &format!("{}", account3), "true"])
        .unwrap();
    full_node_client_2
        .wait_for_transaction(account3, sequence)
        .unwrap();
    assert!(compare_balances(
        vec![(0.0, "Coin1".to_string())],
        full_node_client_2.get_balances(&["b", "3"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "Coin1".to_string())],
        full_node_client_2.get_balances(&["b", "4"]).unwrap(),
    ));
}

#[test]
fn test_e2e_reconfiguration() {
    let (env, mut client_proxy_1) = setup_swarm_and_client_proxy(3, 1);
    let node_configs: Vec<_> = env
        .validator_swarm
        .config
        .config_files
        .iter()
        .map(|config_path| NodeConfig::load(config_path).unwrap())
        .collect();

    // the client connected to the removed validator
    let mut client_proxy_0 = env.get_validator_client(0, None);
    client_proxy_1.create_next_account(false).unwrap();
    client_proxy_0.set_accounts(client_proxy_1.copy_all_accounts());
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    // wait for the mint txn in node 0
    client_proxy_0
        .wait_for_transaction(testnet_dd_account_address(), 1)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    let peer_id = env.get_validator(0).unwrap().validator_peer_id().unwrap();
    let op_tool = env.get_op_tool(1);
    let libra_root = load_libra_root_storage(node_configs.first().unwrap());
    let context = op_tool.remove_validator(peer_id, &libra_root).unwrap();
    client_proxy_1
        .wait_for_transaction(context.address, context.sequence_number)
        .unwrap();
    // mint another 10 coins after remove node 0
    client_proxy_1
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_1.get_balances(&["b", "0"]).unwrap(),
    ));
    // client connected to removed validator can not see the update
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
    // Add the node back
    let context = op_tool.add_validator(peer_id, &libra_root).unwrap();
    client_proxy_0
        .wait_for_transaction(context.address, context.sequence_number)
        .unwrap();
    // Wait for it catches up, mint1 + mint2 => seq == 2
    client_proxy_0
        .wait_for_transaction(testnet_dd_account_address(), 2)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy_0.get_balances(&["b", "0"]).unwrap(),
    ));
}

#[test]
fn test_e2e_modify_publishing_option() {
    let (_env, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();

    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
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
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
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
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
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
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    assert!(compare_balances(
        vec![(20.0, "Coin1".to_string())],
        client_proxy.get_balances(&["b", "0"]).unwrap(),
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
fn test_vfn_failover() {
    // launch environment of 4 validator nodes and 2 full nodes
    let mut env = TestEnvironment::new(4);
    env.setup_vfn_swarm();
    env.setup_public_fn_swarm(1);
    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.public_fn_swarm.as_mut().unwrap().launch();

    // set up clients
    let mut vfn_0_client = env.get_vfn_client(0, None);
    let mut vfn_1_client = env.get_vfn_client(1, None);
    let mut pfn_0_client = env.get_public_fn_client(0, None);

    // some helpers for creation/minting
    let sender_account = testnet_dd_account_address();
    let creation_account = libra_root_address();
    let sequence_reset = format!("sequence {} true", sender_account);
    let creation_sequence_reset = format!("sequence {} true", creation_account);
    let sequence_reset_command: Vec<_> = sequence_reset.split(' ').collect();
    let creation_sequence_reset_command: Vec<_> = creation_sequence_reset.split(' ').collect();

    // Case 1:
    // submit client requests directly to VFN of dead V
    env.validator_swarm.kill_node(0);
    for _ in 0..2 {
        vfn_0_client.create_next_account(false).unwrap();
    }
    vfn_0_client
        .mint_coins(&["mb", "0", "100", "Coin1"], true)
        .unwrap();
    vfn_0_client
        .mint_coins(&["mb", "1", "50", "Coin1"], true)
        .unwrap();
    for _ in 0..20 {
        vfn_0_client
            .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
            .unwrap();
    }
    vfn_0_client
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();

    vfn_1_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    vfn_1_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    for _ in 0..4 {
        vfn_1_client.create_next_account(false).unwrap();
    }
    vfn_1_client
        .mint_coins(&["mb", "2", "100", "Coin1"], true)
        .unwrap();
    vfn_1_client
        .mint_coins(&["mb", "3", "50", "Coin1"], true)
        .unwrap();

    for _ in 0..6 {
        pfn_0_client.create_next_account(false).unwrap();
    }
    // wait for PFN to catch up with creation and sender account
    pfn_0_client
        .wait_for_transaction(creation_account, 5)
        .unwrap();
    pfn_0_client
        .wait_for_transaction(sender_account, 4)
        .unwrap();
    pfn_0_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    pfn_0_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    pfn_0_client
        .mint_coins(&["mb", "4", "100", "Coin1"], true)
        .unwrap();
    pfn_0_client
        .mint_coins(&["mb", "5", "50", "Coin1"], true)
        .unwrap();

    // bring down another V
    // Transition to unfortunate case where 2(>f) validators are down
    // and submit some transactions
    env.validator_swarm.kill_node(1);
    // submit some non-blocking txns during this scenario when >f validators are down
    for _ in 0..10 {
        vfn_1_client
            .transfer_coins(&["t", "2", "3", "1", "Coin1"], false)
            .unwrap();
    }

    // submit txn for vfn_0 too
    for _ in 0..5 {
        vfn_0_client
            .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
            .unwrap();
    }

    // we don't know which exact VFNs each pfn client's PFN is connected to,
    // but by pigeonhole principle, we know the PFN is connected to max 2 live VFNs
    for _ in 0..7 {
        pfn_0_client
            .transfer_coins(&["t", "4", "5", "1", "Coin1"], false)
            .unwrap();
    }

    // bring back one of the validators so consensus can resume
    assert!(env.validator_swarm.add_node(0, false).is_ok());
    // check all txns submitted so far (even those submitted during overlapping validator downtime) are committed
    let vfn_0_acct_0 = vfn_0_client.copy_all_accounts().get(0).unwrap().address;
    vfn_0_client.wait_for_transaction(vfn_0_acct_0, 26).unwrap();
    let vfn_1_acct_0 = vfn_1_client.copy_all_accounts().get(2).unwrap().address;
    vfn_1_client.wait_for_transaction(vfn_1_acct_0, 10).unwrap();
    let pfn_acct_0 = pfn_0_client.copy_all_accounts().get(4).unwrap().address;
    pfn_0_client.wait_for_transaction(pfn_acct_0, 7).unwrap();

    // submit txns to vfn of dead V
    for _ in 0..20 {
        vfn_1_client
            .transfer_coins(&["t", "2", "3", "1", "Coin1"], false)
            .unwrap();
    }
    vfn_1_client
        .transfer_coins(&["tb", "2", "3", "1", "Coin1"], true)
        .unwrap();

    // bring back all Vs back up
    assert!(env.validator_swarm.add_node(1, false).is_ok());

    // just for kicks: check regular minting still works with revived validators
    for _ in 0..20 {
        pfn_0_client
            .transfer_coins(&["t", "4", "5", "1", "Coin1"], false)
            .unwrap();
    }
    pfn_0_client
        .transfer_coins(&["tb", "4", "5", "1", "Coin1"], true)
        .unwrap();
}

#[test]
fn test_malformed_script() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
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
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
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

    // Load validator's on disk storage and update key manager secure backend in config
    let mut storage: Storage = if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        let storage_backend = storage_identity.backend.clone();
        key_manager_config.secure_backend = storage_backend.clone();
        (&storage_backend).try_into().unwrap()
    } else {
        panic!("Couldn't load identity from storage");
    };

    // Save the key manager config to disk
    let key_manager_config_path = node_config_path.with_file_name("key_manager.yaml");
    key_manager_config.save(&key_manager_config_path).unwrap();

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

    // Submit a reconfiguration so that the key rotation will be performed on-chain
    // let libra = get_libra_interface(&node_config);
    // let time_service = RealTimeService::new();
    // let libra_root_key = env.libra_root_key.0;
    // submit_new_reconfig(&libra, &time_service, &libra_root_key).unwrap();
    // sleep(Duration::from_secs(2));

    // Verify the consensus key has been rotated in secure storage and on-chain.
    let rotated_consensus = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let validator_info = libra_interface.retrieve_validator_info(account).unwrap();
    assert_eq!(&rotated_consensus, validator_info.consensus_public_key());
    assert_ne!(current_consensus, rotated_consensus);

    // Cause a failure (i.e., wipe storage) and verify the key manager exits with an error status.
    storage.reset_and_clear().unwrap();
    let output = key_manager.wait_with_output().unwrap();
    if output.status.success() {
        panic!(
            "Key manager did not return an error as expected! Printing key manager output: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

/// Loads the validator's storage backend from the given node config
fn load_backend_storage(node_config: &&NodeConfig) -> SecureBackend {
    if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        storage_identity.backend.clone()
    } else {
        panic!("Couldn't load identity from storage");
    }
}

fn load_libra_root_storage(node_config: &NodeConfig) -> SecureBackend {
    if let Identity::FromStorage(storage_identity) =
        &node_config.validator_network.as_ref().unwrap().identity
    {
        match storage_identity.backend.clone() {
            SecureBackend::OnDiskStorage(mut config) => {
                config.namespace = Some("libra_root".to_string());
                SecureBackend::OnDiskStorage(config)
            }
            _ => unimplemented!("only support on-disk storage in smoke tests"),
        }
    } else {
        panic!("Couldn't load identity from storage");
    }
}

#[test]
fn test_consensus_key_rotation() {
    let mut swarm = TestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the consensus key
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_consensus_key = op_tool
        .validator_config(validator_account)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);

    // Verify that the validator set info contains the new consensus key
    let info_consensus_key = op_tool.validator_set(validator_account).unwrap()[0]
        .consensus_public_key
        .clone();
    assert_eq!(new_consensus_key, info_consensus_key)
}

#[test]
fn test_operator_key_rotation() {
    let mut swarm = TestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = OperationalTool::new(
        format!("http://127.0.0.1:{}", node_config.rpc.address.port()),
        ChainId::test(),
    );

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the transaction was executed correctly
    let result = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    let vm_status = result.unwrap();
    assert_eq!(VMStatusView::Executed, vm_status);

    // Rotate the consensus key to verify the operator key has been updated
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_consensus_key = op_tool
        .validator_config(validator_account)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);
}

#[test]
fn test_network_key_rotation() {
    let mut swarm = TestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&&node_config);

    // Rotate the validator network key
    let (txn_ctx, new_network_key) = op_tool.rotate_validator_network_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that config has been loaded correctly with new key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_network_key = op_tool
        .validator_config(validator_account)
        .unwrap()
        .validator_network_key;
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key =
        op_tool.validator_set(validator_account).unwrap()[0].validator_network_key;
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    swarm.validator_swarm.kill_node(0);
    swarm.validator_swarm.add_node(0, false).unwrap();
}

#[test]
fn test_stop_consensus() {
    let mut env = TestEnvironment::new(4);
    println!("1. set stop_consensus = true for the first node and check it can sync to others");
    let config_path = env.validator_swarm.config.config_files.first().unwrap();
    let mut node_config = NodeConfig::load(config_path).unwrap();
    node_config.consensus.stop_consensus = true;
    node_config.save(config_path).unwrap();
    env.validator_swarm.launch();
    // 1. test the stopped node still syncs the transaction
    let mut client_proxy = env.get_validator_client(0, None);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10", "Coin1"], true)
        .unwrap();
    println!("2. set stop_consensus = true for all nodes and restart");
    for (i, config_path) in env
        .validator_swarm
        .config
        .config_files
        .clone()
        .iter()
        .enumerate()
    {
        let mut node_config = NodeConfig::load(config_path).unwrap();
        node_config.consensus.stop_consensus = true;
        node_config.save(config_path).unwrap();
        env.validator_swarm.kill_node(i);
        env.validator_swarm.add_node(i, false).unwrap();
    }
    println!("3. delete one node's db and test they can still sync when stop_consensus is true for every nodes");
    env.validator_swarm.kill_node(0);
    fs::remove_dir_all(node_config.storage.dir()).unwrap();
    env.validator_swarm.add_node(0, false).unwrap();
    println!("4. verify all nodes are at the same round and no progress being made in 5 sec");
    env.validator_swarm.wait_for_all_nodes_to_catchup();
    let mut known_round = None;
    for i in 0..5 {
        let last_committed_round_str = "libra_consensus_last_committed_round{}";
        for (index, node) in &mut env.validator_swarm.nodes {
            if let Some(round) = node.get_metric(last_committed_round_str) {
                match known_round {
                    Some(r) if r != round => panic!(
                        "round not equal, last known: {}, node {} is {}",
                        r, index, round
                    ),
                    None => known_round = Some(round),
                    _ => continue,
                }
            } else {
                panic!("unable to get round from node {}", index);
            }
        }
        println!(
            "The last know round after {} sec is {}",
            i,
            known_round.unwrap()
        );
        sleep(Duration::from_secs(1));
    }
}
