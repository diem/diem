// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(unused_mut)]
use cli::client_proxy::ClientProxy;
use libra_swarm::swarm::LibraSwarm;
use num_traits::cast::FromPrimitive;
use rust_decimal::Decimal;

fn setup_swarm_and_client_proxy(
    num_nodes: usize,
    client_port_index: usize,
) -> (LibraSwarm, ClientProxy) {
    ::logger::init_for_e2e_testing();

    let (faucet_account_keypair, faucet_key_file_path, _temp_dir) =
        generate_keypair::load_faucet_key_or_create_default(None);

    let swarm = LibraSwarm::launch_swarm(num_nodes, false, faucet_account_keypair, true);
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

fn test_smoke_script(mut client_proxy: ClientProxy) {
    client_proxy.create_next_account().unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    client_proxy.create_next_account().unwrap();
    client_proxy.mint_coins(&["mintb", "1", "1"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "3"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(7.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(4.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
    client_proxy.create_next_account().unwrap();
    client_proxy
        .mint_coins(&["mintb", "2", "15"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(15.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "2"]).unwrap())
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
    client_proxy.create_next_account().unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100"], true)
        .unwrap();
    client_proxy.create_next_account().unwrap();
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
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(21.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
}

#[test]
fn test_basic_fault_tolerance() {
    // A configuration with 4 validators should tolerate single node failure.
    let (mut swarm, mut client_proxy) = setup_swarm_and_client_proxy(4, 1);
    let validator_ports = swarm.get_validators_public_ports();
    // kill the first validator
    swarm.kill_node(*validator_ports.get(0).unwrap());

    // run the script for the smoke test by submitting requests to the second validator
    test_smoke_script(client_proxy);
}

#[test]
fn test_basic_restartability() {
    let (mut swarm, mut client_proxy) = setup_swarm_and_client_proxy(4, 0);
    client_proxy.create_next_account().unwrap();
    client_proxy.create_next_account().unwrap();
    client_proxy.mint_coins(&["mb", "0", "100"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
    let port = swarm.get_validators_public_ports()[0];
    // restart node
    swarm.kill_node(port);
    assert!(swarm.add_node(port, false));
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(80.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(20.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
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
    client_proxy.create_next_account().unwrap();
    client_proxy.create_next_account().unwrap();
    client_proxy.mint_coins(&["mb", "0", "100"], true).unwrap();
    client_proxy
        .transfer_coins(&["tb", "0", "1", "10"], true)
        .unwrap();
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
    let node_to_restart = *swarm.get_validators_public_ports().get(0).unwrap();

    swarm.kill_node(node_to_restart);
    // All these are executed while one node is down
    assert_eq!(
        Decimal::from_f64(90.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(10.0),
        Decimal::from_f64(client_proxy.get_balance(&["b", "1"]).unwrap())
    );
    for _ in 0..5 {
        client_proxy
            .transfer_coins(&["tb", "0", "1", "1"], true)
            .unwrap();
    }

    // Reconnect and synchronize the state
    assert!(swarm.add_node(node_to_restart, false));

    // Wait for all the nodes to catch up
    swarm.wait_for_all_nodes_to_catchup();

    // Connect to the newly recovered node and verify its state
    let tmp_mnemonic_file = tempfile::NamedTempFile::new().unwrap();
    let mut client_proxy2 = ClientProxy::new(
        "localhost",
        node_to_restart.to_string().as_str(),
        &swarm.get_trusted_peers_config_path(),
        "",
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
        Decimal::from_f64(client_proxy2.get_balance(&["b", "0"]).unwrap())
    );
    assert_eq!(
        Decimal::from_f64(15.0),
        Decimal::from_f64(client_proxy2.get_balance(&["b", "1"]).unwrap())
    );
}
