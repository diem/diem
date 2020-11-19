// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    operational_tooling::launch_swarm_with_op_tool_and_backend,
    test_utils::libra_swarm_utils::load_node_config,
};
use libra_config::config::SecureBackend;
use libra_network_address::NetworkAddress;
use libra_secure_json_rpc::VMStatusView;
use libra_secure_storage::{KVStorage, Storage};
use std::{convert::TryInto, str::FromStr};

#[test]
fn test_consensus_observer_mode_storage_error() {
    let num_nodes = 4;
    let (env, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(num_nodes, 0);

    // Kill safety rules storage for validator 1 to ensure it fails on the next epoch change
    let (node_config, _) = load_node_config(&env.validator_swarm, 1);
    let safety_rules_storage = match node_config.consensus.safety_rules.backend {
        SecureBackend::OnDiskStorage(config) => SecureBackend::OnDiskStorage(config),
        _ => panic!("On-disk storage is the only backend supported in smoke tests"),
    };
    let mut safety_rules_storage: Storage = (&safety_rules_storage).try_into().unwrap();
    safety_rules_storage.reset_and_clear().unwrap();

    // Force a new epoch by updating validator 0's full node address in the validator config
    let txn_ctx = op_tool
        .set_validator_config(
            None,
            Some(NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap()),
            &backend,
            false,
            false,
        )
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Rotate validator 0's operator key several different times, each requiring a new transaction
    for _ in 0..5 {
        let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, false).unwrap();
        assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());
    }

    // Verify validator 1 is still able to stay up to date with validator 0 (despite safety rules failing)
    let mut client_0 = env.get_validator_client(0, None);
    let sequence_number_0 = client_0
        .get_sequence_number(&["sequence", &txn_ctx.address.to_string()])
        .unwrap();
    let mut client_1 = env.get_validator_client(1, None);
    let sequence_number_1 = client_1
        .get_sequence_number(&["sequence", &txn_ctx.address.to_string()])
        .unwrap();
    assert_eq!(sequence_number_0, sequence_number_1);
}
