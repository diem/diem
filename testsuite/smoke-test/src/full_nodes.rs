// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{compare_balances, test_smoke_script},
};
use libra_types::account_config::{
    testnet_dd_account_address, treasury_compliance_account_address,
};

#[test]
fn test_full_node_basic_flow() {
    // launch environment of 4 validator nodes and 2 full nodes
    let mut env = SmokeTestEnvironment::new(4);
    env.setup_vfn_swarm();
    env.setup_pfn_swarm(2);
    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.public_fn_swarm.as_mut().unwrap().launch();

    // execute smoke script
    test_smoke_script(env.get_validator_client(0, None));

    // read state from full node client
    let mut validator_ac_client = env.get_validator_client(1, None);
    let mut full_node_client = env.get_vfn_client(1, None);
    let mut full_node_client_2 = env.get_pfn_client(0, None);

    // ensure the client has up-to-date sequence number after test_smoke_script(3 minting)
    let sender_account = testnet_dd_account_address();
    let creation_account = treasury_compliance_account_address();
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
fn test_vfn_failover() {
    // launch environment of 6 validator nodes and 2 full nodes
    let mut env = SmokeTestEnvironment::new(6);
    env.setup_vfn_swarm();
    env.setup_pfn_swarm(1);
    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.public_fn_swarm.as_mut().unwrap().launch();

    // set up clients
    let mut vfn_0_client = env.get_vfn_client(0, None);
    let mut vfn_1_client = env.get_vfn_client(1, None);
    let mut pfn_0_client = env.get_pfn_client(0, None);

    // some helpers for creation/minting
    let sender_account = testnet_dd_account_address();
    let creation_account = treasury_compliance_account_address();
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
    for _ in 0..8 {
        vfn_0_client
            .transfer_coins(&["t", "0", "1", "1", "Coin1"], false)
            .unwrap();
    }
    vfn_0_client
        .transfer_coins(&["tb", "0", "1", "1", "Coin1"], true)
        .unwrap();

    // wait for VFN 1 to catch up with creation and sender account
    vfn_1_client
        .wait_for_transaction(creation_account, 1)
        .unwrap();
    vfn_1_client
        .wait_for_transaction(sender_account, 2)
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
        .wait_for_transaction(creation_account, 3)
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
    assert!(env.validator_swarm.add_node(0).is_ok());
    // check all txns submitted so far (even those submitted during overlapping validator downtime) are committed
    let vfn_0_acct_0 = vfn_0_client.copy_all_accounts().get(0).unwrap().address;
    vfn_0_client.wait_for_transaction(vfn_0_acct_0, 14).unwrap();
    let vfn_1_acct_0 = vfn_1_client.copy_all_accounts().get(2).unwrap().address;
    vfn_1_client.wait_for_transaction(vfn_1_acct_0, 10).unwrap();
    let pfn_acct_0 = pfn_0_client.copy_all_accounts().get(4).unwrap().address;
    pfn_0_client.wait_for_transaction(pfn_acct_0, 7).unwrap();

    // submit txns to vfn of dead V
    for _ in 0..5 {
        vfn_1_client
            .transfer_coins(&["t", "2", "3", "1", "Coin1"], false)
            .unwrap();
    }
    vfn_1_client
        .transfer_coins(&["tb", "2", "3", "1", "Coin1"], true)
        .unwrap();

    // bring back all Vs back up
    assert!(env.validator_swarm.add_node(1).is_ok());

    // just for kicks: check regular minting still works with revived validators
    for _ in 0..5 {
        pfn_0_client
            .transfer_coins(&["t", "4", "5", "1", "Coin1"], false)
            .unwrap();
    }
    pfn_0_client
        .transfer_coins(&["tb", "4", "5", "1", "Coin1"], true)
        .unwrap();
}
