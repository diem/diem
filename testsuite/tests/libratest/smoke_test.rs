// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_mut)]
use cli::client_proxy::ClientProxy;
use libra_swarm::{swarm::LibraSwarm, utils};
use num_traits::cast::FromPrimitive;
use rust_decimal::Decimal;
use std::str::FromStr;

fn setup_env(
    num_nodes: usize,
    client_port_index: usize,
    template_path: Option<String>,
) -> (LibraSwarm, ClientProxy) {
    ::logger::init_for_e2e_testing();

    let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(None);

    let swarm = LibraSwarm::launch_swarm(
        num_nodes,
        false, /* disable_logging */
        faucet_account_keypair,
        true, /* tee_logs */
        None, /* config_dir */
        template_path,
    );
    let port = *swarm
        .get_validators_public_ports()
        .get(client_port_index)
        .unwrap();
    let tmp_mnemonic_file = tempfile::NamedTempFile::new().unwrap();
    let client_proxy = ClientProxy::new(
        "localhost",
        port.to_string().as_str(),
        &swarm.get_trusted_peers_config_path(),
        &faucet_key_file_path,
        false,
        /* faucet server */ None,
        Some(
            tmp_mnemonic_file
                .into_temp_path()
                .canonicalize()
                .expect("Unable to get canonical path of mnemonic_file_path")
                .to_str()
                .unwrap()
                .to_string(),
        ),
    )
    .unwrap();
    (swarm, client_proxy)
}

fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    client_port_index: usize,
) -> (LibraSwarm, ClientProxy) {
    setup_env(num_nodes, client_port_index, None)
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
    let (mut swarm, mut client_proxy) = setup_swarm_and_client_proxy(4, 1);
    let validators = swarm.get_validators_ids();
    // kill the first validator
    swarm.kill_node(validators.get(0).unwrap());

    // run the script for the smoke test by submitting requests to the second validator
    test_smoke_script(client_proxy);
}

#[test]
fn test_basic_restartability() {
    let (mut swarm, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
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
    let peer_to_restart = swarm.get_validators_ids()[0].clone();
    // restart node
    swarm.kill_node(&peer_to_restart);
    assert!(swarm.add_node(peer_to_restart, false).is_ok());
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
    let (mut swarm, mut client_proxy) = setup_swarm_and_client_proxy(5, 1);
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
    let node_to_restart = swarm.get_validators_ids().get(0).unwrap().clone();

    swarm.kill_node(&node_to_restart);
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
    assert!(swarm.add_node(node_to_restart.clone(), false).is_ok());

    // Wait for all the nodes to catch up
    swarm.wait_for_all_nodes_to_catchup();

    // Connect to the newly recovered node and verify its state
    let tmp_mnemonic_file = tempfile::NamedTempFile::new().unwrap();
    let ac_port = swarm.get_validator(&node_to_restart).unwrap().ac_port();
    let mut client_proxy2 = ClientProxy::new(
        "localhost",
        ac_port.to_string().as_str(),
        &swarm.get_trusted_peers_config_path(),
        "",
        false,
        /* faucet server */ None,
        Some(
            tmp_mnemonic_file
                .into_temp_path()
                .canonicalize()
                .expect("Unable to get canonical path of mnemonic_file_path")
                .to_str()
                .unwrap()
                .to_string(),
        ),
    )
    .unwrap();
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
fn test_full_node() {
    let (mut _swarm, mut client_proxy) = setup_env(
        1,
        0,
        Some("config/data/configs/full_node.config.toml".to_string()),
    );
    assert_eq!(
        Decimal::from_f64(1000.0),
        Decimal::from_str(
            &client_proxy
                .get_balance(&[
                    "b",
                    "0000000000000000000000000000000000000000000000000000000000000000"
                ])
                .unwrap()
        )
        .ok()
    );
    client_proxy.create_next_account(false).unwrap();
    let response = client_proxy.mint_coins(&["mint", "0", "1"], false);
    assert!(response.is_err());
}
