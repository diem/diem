// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{smoke_test_environment::SmokeTestEnvironment, test_utils::compare_balances};
use diem_types::account_config::{testnet_dd_account_address, treasury_compliance_account_address};

#[test]
fn test_full_node_basic_flow() {
    let mut env = SmokeTestEnvironment::new(4);
    env.setup_vfn_swarm();
    env.setup_pfn_swarm(2);

    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.pfn_swarm.as_mut().unwrap().launch();

    // read state from full node client
    let mut validator_client = env.get_validator_client(0, None);
    let mut vfn_client = env.get_vfn_client(1, None);
    let mut pfn_client = env.get_pfn_client(0, None);

    // mint from full node and check both validator and full node have correct balance
    let account = validator_client.create_next_account(false).unwrap().address;
    vfn_client.create_next_account(false).unwrap();
    pfn_client.create_next_account(false).unwrap();

    let sender_account = testnet_dd_account_address();
    let creation_account = treasury_compliance_account_address();
    let sequence_reset = format!("sequence {} true", sender_account);
    let creation_sequence_reset = format!("sequence {} true", creation_account);
    let sequence_reset_command: Vec<_> = sequence_reset.split(' ').collect();
    let creation_sequence_reset_command: Vec<_> = creation_sequence_reset.split(' ').collect();

    vfn_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    vfn_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    vfn_client
        .mint_coins(&["mintb", "0", "10", "XUS"], true)
        .expect("Fail to mint!");
    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        vfn_client.get_balances(&["b", "0"]).unwrap(),
    ));

    let sequence = vfn_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    validator_client
        .wait_for_transaction(sender_account, sequence - 1)
        .unwrap();
    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        validator_client.get_balances(&["b", "0"]).unwrap(),
    ));

    // reset sequence number for sender account
    validator_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    vfn_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    validator_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    vfn_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();

    // mint from validator and check both nodes have correct balance
    validator_client.create_next_account(false).unwrap();
    vfn_client.create_next_account(false).unwrap();
    pfn_client.create_next_account(false).unwrap();

    validator_client
        .mint_coins(&["mintb", "1", "10", "XUS"], true)
        .unwrap();
    let sequence = validator_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    vfn_client
        .wait_for_transaction(sender_account, sequence - 1)
        .unwrap();

    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        validator_client.get_balances(&["b", "1"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(10.0, "XUS".to_string())],
        vfn_client.get_balances(&["b", "1"]).unwrap(),
    ));

    // minting again on validator doesn't cause error since client sequence has been updated
    validator_client
        .mint_coins(&["mintb", "1", "10", "XUS"], true)
        .unwrap();

    // test transferring balance from 0 to 1 through full node proxy
    vfn_client
        .transfer_coins(&["tb", "0", "1", "10", "XUS"], true)
        .unwrap();

    assert!(compare_balances(
        vec![(0.0, "XUS".to_string())],
        vfn_client.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "XUS".to_string())],
        validator_client.get_balances(&["b", "1"]).unwrap(),
    ));

    let sequence = validator_client
        .get_sequence_number(&["sequence", &format!("{}", account), "true"])
        .unwrap();
    pfn_client
        .wait_for_transaction(account, sequence - 1)
        .unwrap();
    assert!(compare_balances(
        vec![(0.0, "XUS".to_string())],
        pfn_client.get_balances(&["b", "0"]).unwrap(),
    ));
    assert!(compare_balances(
        vec![(30.0, "XUS".to_string())],
        pfn_client.get_balances(&["b", "1"]).unwrap(),
    ));
}

#[test]
fn test_vfn_failover() {
    let mut env = SmokeTestEnvironment::new(7);
    env.setup_vfn_swarm();
    env.setup_pfn_swarm(1);

    env.validator_swarm.launch();
    env.vfn_swarm.as_mut().unwrap().launch();
    env.pfn_swarm.as_mut().unwrap().launch();

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
        .mint_coins(&["mb", "0", "100", "XUS"], true)
        .unwrap();
    vfn_0_client
        .mint_coins(&["mb", "1", "50", "XUS"], true)
        .unwrap();
    for _ in 0..8 {
        vfn_0_client
            .transfer_coins(&["t", "0", "1", "1", "XUS"], false)
            .unwrap();
    }
    vfn_0_client
        .transfer_coins(&["tb", "0", "1", "1", "XUS"], true)
        .unwrap();

    // wait for VFN 1 to catch up with creation and sender account
    vfn_1_client
        .wait_for_transaction(creation_account, 0)
        .unwrap();
    vfn_1_client
        .wait_for_transaction(sender_account, 1)
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
        .mint_coins(&["mb", "2", "100", "XUS"], true)
        .unwrap();
    vfn_1_client
        .mint_coins(&["mb", "3", "50", "XUS"], true)
        .unwrap();

    for _ in 0..6 {
        pfn_0_client.create_next_account(false).unwrap();
    }
    // wait for PFN to catch up with creation and sender account
    pfn_0_client
        .wait_for_transaction(creation_account, 2)
        .unwrap();
    pfn_0_client
        .wait_for_transaction(sender_account, 3)
        .unwrap();
    pfn_0_client
        .get_sequence_number(&sequence_reset_command)
        .unwrap();
    pfn_0_client
        .get_sequence_number(&creation_sequence_reset_command)
        .unwrap();
    pfn_0_client
        .mint_coins(&["mb", "4", "100", "XUS"], true)
        .unwrap();
    pfn_0_client
        .mint_coins(&["mb", "5", "50", "XUS"], true)
        .unwrap();

    // bring down another V
    // Transition to unfortunate case where 2(>f) validators are down
    // and submit some transactions
    env.validator_swarm.kill_node(1);
    // submit some non-blocking txns during this scenario when >f validators are down
    for _ in 0..10 {
        vfn_1_client
            .transfer_coins(&["t", "2", "3", "1", "XUS"], false)
            .unwrap();
    }

    // submit txn for vfn_0 too
    for _ in 0..5 {
        vfn_0_client
            .transfer_coins(&["t", "0", "1", "1", "XUS"], false)
            .unwrap();
    }

    // we don't know which exact VFNs each pfn client's PFN is connected to,
    // but by pigeonhole principle, we know the PFN is connected to max 2 live VFNs
    for _ in 0..7 {
        pfn_0_client
            .transfer_coins(&["t", "4", "5", "1", "XUS"], false)
            .unwrap();
    }

    // bring back one of the validators so consensus can resume
    assert!(env.validator_swarm.add_node(0).is_ok());
    // check all txns submitted so far (even those submitted during overlapping validator downtime) are committed
    let vfn_0_acct_0 = vfn_0_client.copy_all_accounts().get(0).unwrap().address;
    vfn_0_client.wait_for_transaction(vfn_0_acct_0, 13).unwrap();
    let vfn_1_acct_0 = vfn_1_client.copy_all_accounts().get(2).unwrap().address;
    vfn_1_client.wait_for_transaction(vfn_1_acct_0, 9).unwrap();
    let pfn_acct_0 = pfn_0_client.copy_all_accounts().get(4).unwrap().address;
    pfn_0_client.wait_for_transaction(pfn_acct_0, 6).unwrap();

    // submit txns to vfn of dead V
    for _ in 0..5 {
        vfn_1_client
            .transfer_coins(&["t", "2", "3", "1", "XUS"], false)
            .unwrap();
    }
    vfn_1_client
        .transfer_coins(&["tb", "2", "3", "1", "XUS"], true)
        .unwrap();

    // bring back all Vs back up
    assert!(env.validator_swarm.add_node(1).is_ok());

    // just for kicks: check regular minting still works with revived validators
    for _ in 0..5 {
        pfn_0_client
            .transfer_coins(&["t", "4", "5", "1", "XUS"], false)
            .unwrap();
    }
    pfn_0_client
        .transfer_coins(&["tb", "4", "5", "1", "XUS"], true)
        .unwrap();
}
