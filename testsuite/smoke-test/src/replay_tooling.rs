// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{test_utils::setup_swarm_and_client_proxy, workspace_builder};
use libra_config::config::NodeConfig;
use libra_json_rpc::views::VMStatusView as JsonVMStatusView;
use libra_transaction_replay::LibraDebugger;

#[test]
fn test_replay_tooling() {
    let (swarm, mut client_proxy) = setup_swarm_and_client_proxy(1, 0);
    let validator_config = NodeConfig::load(&swarm.validator_swarm.config.config_files[0]).unwrap();
    let swarm_rpc_endpoint = format!("http://localhost:{}", validator_config.rpc.address.port());
    let json_debugger = LibraDebugger::json_rpc(swarm_rpc_endpoint.as_str()).unwrap();

    client_proxy.create_next_account(false).unwrap();
    client_proxy.create_next_account(false).unwrap();
    client_proxy
        .mint_coins(&["mintb", "0", "100", "Coin1"], true)
        .unwrap();

    client_proxy
        .mint_coins(&["mintb", "1", "100", "Coin1"], true)
        .unwrap();

    client_proxy
        .transfer_coins(&["tb", "0", "1", "3", "Coin1"], true)
        .unwrap();

    let txn = client_proxy
        .get_committed_txn_by_acc_seq(&["txn_acc_seq", "0", "0", "false"])
        .unwrap()
        .unwrap();

    let replay_result = json_debugger
        .execute_past_transactions(txn.version, 1)
        .unwrap()
        .pop()
        .unwrap();

    let (account, _) = client_proxy
        .get_account_address_from_parameter("0")
        .unwrap();
    let script_path = workspace_builder::workspace_root()
        .join("language/libra-tools/transaction-replay/examples/account_exists.move");

    let bisect_result = json_debugger
        .bisect_transactions_by_script(script_path.to_str().unwrap(), account, 0, txn.version)
        .unwrap()
        .unwrap();

    let account_creation_txn = client_proxy
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
    assert_eq!(
        JsonVMStatusView::from(&replay_result.status().status().unwrap()),
        txn.vm_status
    );
}
