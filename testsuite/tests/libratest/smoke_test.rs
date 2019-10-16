// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_mut)]
use cli::{
    client_proxy::ClientProxy, AccountAddress, CryptoHash, TransactionArgument, TransactionPayload,
};
use config::config::{NodeConfig, RoleType};
use crypto::{ed25519::*, test_utils::KeyPair, SigningKey};
use libra_swarm::{swarm::LibraSwarm, utils};
use libra_tools::tempdir::TempPath;
use logger::prelude::*;
use num_traits::cast::FromPrimitive;
use rust_decimal::Decimal;
use std::fs;
use std::str::FromStr;

struct TestEnvironment {
    validator_swarm: LibraSwarm,
    full_node_swarm: Option<LibraSwarm>,
    faucet_key: (
        KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
        String,
        Option<TempPath>,
    ),
    mnemonic_file: TempPath,
}

impl TestEnvironment {
    fn new(num_validators: usize) -> Self {
        ::logger::init_for_e2e_testing();
        let faucet_key = generate_keypair::load_faucet_key_or_create_default(None);
        let validator_swarm = LibraSwarm::configure_swarm(
            num_validators,
            RoleType::Validator,
            faucet_key.0.clone(),
            None,
            None,
            None,
        )
        .unwrap();

        let mnemonic_file = libra_tools::tempdir::TempPath::new();
        mnemonic_file
            .create_as_file()
            .expect("could not create temporary mnemonic_file_path");
        Self {
            validator_swarm,
            full_node_swarm: None,
            faucet_key,
            mnemonic_file,
        }
    }

    fn setup_full_node_swarm(&mut self, num_full_nodes: usize) {
        self.full_node_swarm = Some(
            LibraSwarm::configure_swarm(
                num_full_nodes,
                RoleType::FullNode,
                self.faucet_key.0.clone(),
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
        let mut swarm = match role {
            RoleType::Validator => &mut self.validator_swarm,
            RoleType::FullNode => self.full_node_swarm.as_mut().unwrap(),
        };
        let num_attempts = 5;
        for _ in 0..num_attempts {
            match swarm.launch_attempt(false) {
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

    fn get_ac_client(&self, port: u16) -> ClientProxy {
        let config = NodeConfig::load(&self.validator_swarm.config.configs[0]).unwrap();
        let validator_set_file = self
            .validator_swarm
            .dir
            .as_ref()
            .join("0")
            .join(&config.consensus.consensus_peers_file);
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
            "localhost",
            port,
            validator_set_file.to_str().unwrap(),
            &self.faucet_key.1,
            false,
            /* faucet server */ None,
            Some(mnemonic_file_path),
        )
        .unwrap()
    }

    fn get_validator_ac_client(&self, node_index: usize) -> ClientProxy {
        let port = self.validator_swarm.get_ac_port(node_index);
        self.get_ac_client(port)
    }

    fn get_full_node_ac_client(&self, node_index: usize) -> ClientProxy {
        match &self.full_node_swarm {
            Some(swarm) => {
                let port = swarm.get_ac_port(node_index);
                self.get_ac_client(port)
            }
            None => {
                panic!("Full Node swarm is not initialized");
            }
        }
    }
}

fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    client_port_index: usize,
) -> (TestEnvironment, ClientProxy) {
    let mut env = TestEnvironment::new(num_nodes);
    env.launch_swarm(RoleType::Validator);
    let ac_client = env.get_validator_ac_client(client_port_index);
    (env, ac_client)
}

fn test_smoke_script(mut client_proxy: ClientProxy) {
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    client_proxy.create_next_account(false).unwrap();
    client_proxy.mint_coins(&["mintb", "1", "1"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "3"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(7.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(4.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "2", "15"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(15.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "2"]).unwrap()).ok()
    );
}

#[test]
fn test_execute_custom_module_and_script() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "50"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(50.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );

    let recipient_address = client_proxy.create_next_account(false).unwrap().address;
    client_proxy.mint_coins(&["mintb", "1", "1"], true).unwrap();

    let module_path =
        utils::workspace_root().join("testsuite/tests/libratest/dev_modules/module.mvir");
    let unwrapped_module_path = module_path.to_str().unwrap();
    let module_params = &["compile", "0", unwrapped_module_path, "module"];
    let module_compiled_path = client_proxy.compile_program(module_params).unwrap();

    client_proxy
        .publish_module(&["publish", "0", &module_compiled_path[..]])
        .unwrap();

    let script_path =
        utils::workspace_root().join("testsuite/tests/libratest/dev_modules/script.mvir");
    let unwrapped_script_path = script_path.to_str().unwrap();
    let script_params = &["execute", "0", unwrapped_script_path, "script"];
    let script_compiled_path = client_proxy.compile_program(script_params).unwrap();
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

    assert_eq!(
        Decimal::from_f64(49.999_990),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(1.000_010),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
}

#[test]
fn smoke_test_single_node() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    test_smoke_script(client_proxy);
}

#[test]
fn smoke_test_multi_node() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
    test_smoke_script(client_proxy);
}

#[test]
fn test_concurrent_transfers_single_node() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100"], true)
        .unwrap();
    client_proxy.create_next_account(false).unwrap();
    for _ in 0..20 {
        client_proxy
            .transfer_coins(&["t", "0", "1", "1"], false)
            .unwrap();
    }
    client_proxy
        .transfer_coins(&["tb", "0", "1", "1"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(79.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(21.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
}

#[test]
fn test_basic_fault_tolerance() {
    // A configuration with 4 validators should tolerate single node failure.
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(4, 1);
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
    client_proxy.mint_coins(&["mb", "0", "100"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    let peer_to_restart = 0;
    // restart node
    env.validator_swarm.kill_node(peer_to_restart);
    assert!(env.validator_swarm.add_node(peer_to_restart, false).is_ok());
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(80.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(20.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
}

#[test]
fn test_startup_sync_state() {
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy.mint_coins(&["mb", "0", "100"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    let peer_to_stop = 0;
    env.validator_swarm.kill_node(peer_to_stop);
    let node_config = NodeConfig::load(
        env.validator_swarm
            .config
            .configs
            .get(peer_to_stop)
            .unwrap(),
    )
    .unwrap();
    // TODO Remove hardcoded path to state db
    let state_db_path = node_config.get_storage_dir().join("libradb");
    // Verify that state_db_path exists and
    // we are not deleting a non-existent directory
    assert!(state_db_path.as_path().exists());
    // Delete the state db to simulate state db lagging
    // behind consensus db and forcing a state sync
    // during a node startup
    fs::remove_dir_all(state_db_path).unwrap();
    assert!(env.validator_swarm.add_node(peer_to_stop, false).is_ok());
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(80.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(20.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
}

#[test]
fn test_basic_state_synchronization() {
    //
    // - Start a swarm of 5 nodes (3 nodes forming a QC).
    // - Kill one node and continue submitting transactions to the others.
    // - Restart the node
    // - Wait for all the nodes to catch up
    // - Verify that the restarted node has synced up with the submitted transactions.
    let (mut env, mut client_proxy) = setup_swarm_and_client_proxy(5, 1);
    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy.mint_coins(&["mb", "0", "100"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    let node_to_restart = 0;
    env.validator_swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&client_proxy.get_balance(&["b", "1"]).unwrap()).ok()
    );
    for _ in 0..5 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(env.validator_swarm.add_node(node_to_restart, false).is_ok());

    // Wait for all the nodes to catch up
    assert!(env.validator_swarm.wait_for_all_nodes_to_catchup());

    // Connect to the newly recovered node and verify its state
    let mut client_proxy2 = env.get_validator_ac_client(node_to_restart);
    client_proxy2.set_accounts(client_proxy.copy_all_accounts());
    assert_eq!(
        Decimal::from_f64(85.0),
        Decimal::from_str(&client_proxy2.get_balance(&["b", "0"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(15.0),
        Decimal::from_str(&client_proxy2.get_balance(&["b", "1"]).unwrap()).ok()
    );
}

#[test]
fn test_external_transaction_signer() {
    let (_swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);

    // generate key pair
    let mut seed: [u8; 32] = [0u8; 32];
    seed[..4].copy_from_slice(&[1, 2, 3, 4]);
    let key_pair = compat::generate_keypair(None);
    let private_key = key_pair.0;
    let public_key = key_pair.1;

    // create transfer parameters
    let sender_address = AccountAddress::from_public_key(&public_key);
    let receiver_address = client_proxy
        .get_account_address_from_parameter(
            "1bfb3b36384dabd29e38b4a0eafd9797b75141bb007cea7943f8a4714d3d784a",
        )
        .unwrap();
    let amount = ClientProxy::convert_to_micro_libras("1").unwrap();
    let gas_unit_price = 123;
    let max_gas_amount = 1000;

    // mint to the sender address
    client_proxy
        .mint_coins(&["mintb", &format!("{}", sender_address), "10"], true)
        .unwrap();

    // prepare transfer transaction
    let sequence_number = client_proxy
        .get_sequence_number(&["sequence", &format!("{}", sender_address)])
        .unwrap();

    let unsigned_txn = client_proxy
        .prepare_transfer_coins(
            sender_address,
            sequence_number,
            receiver_address,
            amount,
            Some(gas_unit_price),
            Some(max_gas_amount),
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
    let submitted_signed_txn = client_proxy
        .get_committed_txn_by_acc_seq(&[
            "txn_acc_seq",
            &format!("{}", sender_address),
            &sequence_number.to_string(),
            "false",
        ])
        .unwrap()
        .unwrap()
        .0;

    assert_eq!(submitted_signed_txn.sender(), sender_address);
    assert_eq!(submitted_signed_txn.sequence_number(), sequence_number);
    assert_eq!(submitted_signed_txn.gas_unit_price(), gas_unit_price);
    assert_eq!(submitted_signed_txn.max_gas_amount(), max_gas_amount);
    match submitted_signed_txn.payload() {
        TransactionPayload::Script(program) => match program.args().len() {
            2 => match (&program.args()[0], &program.args()[1]) {
                (
                    TransactionArgument::Address(arg_receiver),
                    TransactionArgument::U64(arg_amount),
                ) => {
                    assert_eq!(arg_receiver.clone(), receiver_address);
                    assert_eq!(arg_amount.clone(), amount);
                }
                _ => panic!(
                    "The first argument for payment transaction must be recipient address \
                     and the second argument must be amount."
                ),
            },
            _ => panic!("Signed transaction payload arguments must have two arguments."),
        },
        _ => panic!("Signed transaction payload expected to be of struct Script"),
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
    test_smoke_script(env.get_validator_ac_client(0));

    // read state from full node client
    let mut validator_ac_client = env.get_validator_ac_client(1);
    let mut full_node_client = env.get_full_node_ac_client(1);
    let mut full_node_client_2 = env.get_full_node_ac_client(0);
    for idx in 0..3 {
        validator_ac_client.create_next_account(false).unwrap();
        full_node_client.create_next_account(false).unwrap();
        full_node_client_2.create_next_account(false).unwrap();
        assert_eq!(
            validator_ac_client
                .get_balance(&["b", &idx.to_string()])
                .unwrap(),
            full_node_client
                .get_balance(&["b", &idx.to_string()])
                .unwrap(),
        );
    }

    let sender_account = &format!(
        "{}",
        validator_ac_client.faucet_account.clone().unwrap().address
    );
    // mint from full node and check both validator and full node have correct balance
    let account3 = validator_ac_client
        .create_next_account(false)
        .unwrap()
        .address;
    full_node_client.create_next_account(false).unwrap();
    full_node_client_2.create_next_account(false).unwrap();

    let mint_result = full_node_client.mint_coins(&["mintb", "3", "10"], true);
    assert!(mint_result.is_ok());
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&full_node_client.get_balance(&["b", "3"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&validator_ac_client.get_balance(&["b", "3"]).unwrap()).ok()
    );

    // reset sequence number for sender account
    validator_ac_client
        .get_sequence_number(&["sequence", sender_account, "true"])
        .unwrap();
    full_node_client
        .get_sequence_number(&["sequence", sender_account, "true"])
        .unwrap();

    // mint from validator and check both nodes have correct balance
    validator_ac_client.create_next_account(false).unwrap();
    full_node_client.create_next_account(false).unwrap();
    full_node_client_2.create_next_account(false).unwrap();

    validator_ac_client
        .mint_coins(&["mintb", "4", "10"], true)
        .unwrap();
    let sequence = validator_ac_client
        .get_sequence_number(&["sequence", sender_account, "true"])
        .unwrap();
    full_node_client.wait_for_transaction(
        validator_ac_client.faucet_account.clone().unwrap().address,
        sequence,
    );

    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&validator_ac_client.get_balance(&["b", "4"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_str(&full_node_client.get_balance(&["b", "4"]).unwrap()).ok()
    );

    // minting again on validator doesn't cause error since client sequence has been updated
    validator_ac_client
        .mint_coins(&["mintb", "4", "10"], true)
        .unwrap();

    // test transferring balance from 0 to 1 through full node proxy
    full_node_client
        .transfer_coins(&["tb", "3", "4", "10"], true)
        .unwrap();

    assert_eq!(
        Decimal::from_f64(0.0),
        Decimal::from_str(&full_node_client.get_balance(&["b", "3"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(30.0),
        Decimal::from_str(&validator_ac_client.get_balance(&["b", "4"]).unwrap()).ok()
    );

    let sequence = validator_ac_client
        .get_sequence_number(&["sequence", &format!("{}", account3), "true"])
        .unwrap();
    full_node_client_2.wait_for_transaction(account3, sequence);
    assert_eq!(
        Decimal::from_f64(0.0),
        Decimal::from_str(&full_node_client_2.get_balance(&["b", "3"]).unwrap()).ok()
    );
    assert_eq!(
        Decimal::from_f64(30.0),
        Decimal::from_str(&full_node_client_2.get_balance(&["b", "4"]).unwrap()).ok()
    );
}
