// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    test_utils::{diem_swarm_utils::get_diem_debugger, setup_swarm_and_client_proxy},
    workspace_builder,
};

#[test]
fn test_replay_tooling() {
    let (env, mut client) = setup_swarm_and_client_proxy(1, 0);
    let json_debugger = get_diem_debugger(&env.validator_swarm, 0);

    client.create_next_account(false).unwrap();
    client.create_next_account(false).unwrap();
    client
        .mint_coins(&["mintb", "0", "100", "XUS"], true)
        .unwrap();

    client
        .mint_coins(&["mintb", "1", "100", "XUS"], true)
        .unwrap();

    client
        .transfer_coins(&["tb", "0", "1", "3", "XUS"], true)
        .unwrap();

    let txn = client
        .get_committed_txn_by_acc_seq(&["txn_acc_seq", "0", "0", "false"])
        .unwrap()
        .unwrap();

    let replay_result = json_debugger
        .execute_past_transactions(txn.version, 1)
        .unwrap()
        .pop()
        .unwrap();

    let (account, _) = client.get_account_address_from_parameter("0").unwrap();
    let script_path = workspace_builder::workspace_root()
        .join("language/diem-tools/transaction-replay/examples/account_exists.move");

    let bisect_result = json_debugger
        .bisect_transactions_by_script(script_path.to_str().unwrap(), account, 0, txn.version, None)
        .unwrap()
        .unwrap();

    let account_creation_txn = client
        .get_committed_txn_by_acc_seq(&[
            "txn_acc_seq",
            "0000000000000000000000000b1e55ed",
            "0",
            "false",
        ])
        .unwrap()
        .unwrap();

    assert_eq!(account_creation_txn.version + 1, bisect_result);
    assert_eq!(replay_result.gas_used(), txn.gas_used);
    assert_eq!("executed", txn.vm_status.unwrap().r#type);
}
