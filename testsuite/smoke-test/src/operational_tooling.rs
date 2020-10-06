// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{smoke_test_environment::SmokeTestEnvironment, test_utils::load_backend_storage};
use libra_config::config::NodeConfig;
use libra_global_constants::{
    CONSENSUS_KEY, OPERATOR_ACCOUNT, OPERATOR_KEY, OWNER_ACCOUNT, VALIDATOR_NETWORK_ADDRESS_KEYS,
    VALIDATOR_NETWORK_KEY,
};
use libra_management::storage::to_x25519;
use libra_network_address::NetworkAddress;
use libra_operational_tool::test_helper::OperationalTool;
use libra_secure_json_rpc::VMStatusView;
use libra_secure_storage::{CryptoStorage, KVStorage, Storage};
use libra_types::{
    account_address::AccountAddress, chain_id::ChainId,
    transaction::authenticator::AuthenticationKey,
};
use std::{
    convert::{TryFrom, TryInto},
    fs,
    str::FromStr,
};

#[test]
fn test_account_resource() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let storage: Storage = (&backend).try_into().unwrap();

    // Fetch the owner account resource
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let account_resource = op_tool.account_resource(owner_account).unwrap();
    assert_eq!(owner_account, account_resource.account);
    assert_eq!(0, account_resource.sequence_number);

    // Fetch the operator account resource
    let operator_account = storage
        .get::<AccountAddress>(OPERATOR_ACCOUNT)
        .unwrap()
        .value;
    let account_resource = op_tool.account_resource(operator_account).unwrap();
    assert_eq!(operator_account, account_resource.account);
    assert_eq!(0, account_resource.sequence_number);

    // Verify operator key
    let on_chain_operator_key = hex::decode(account_resource.authentication_key).unwrap();
    let operator_key = storage.get_public_key(OPERATOR_KEY).unwrap().public_key;
    assert_eq!(
        AuthenticationKey::ed25519(&operator_key),
        AuthenticationKey::try_from(on_chain_operator_key).unwrap()
    );
}

#[test]
fn test_consensus_key_rotation() {
    let mut swarm = SmokeTestEnvironment::new(5);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);

    // Rotate the consensus key
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_consensus_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);

    // Verify that the validator set info contains the new consensus key
    let info_consensus_key = op_tool
        .validator_set(Some(validator_account), &backend)
        .unwrap()[0]
        .consensus_public_key
        .clone();
    assert_eq!(new_consensus_key, info_consensus_key);

    // Rotate the consensus key in storage manually and perform another rotation using the op_tool.
    // Here, we expected the op_tool to see that the consensus key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let mut storage: Storage = (&backend).try_into().unwrap();
    let rotated_consensus_key = storage.rotate_key(CONSENSUS_KEY).unwrap();
    let (_txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend).unwrap();
    assert_eq!(rotated_consensus_key, new_consensus_key);
}

#[test]
fn test_extract_private_key() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config_path = swarm.validator_swarm.config.config_files.first().unwrap();
    let node_config = NodeConfig::load(node_config_path.clone()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let storage: Storage = (&backend).try_into().unwrap();

    // Extract the operator private key to file
    let key_file_path = node_config_path.with_file_name(OPERATOR_KEY);
    let _ = op_tool
        .extract_private_key(OPERATOR_KEY, key_file_path.to_str().unwrap(), &backend)
        .unwrap();

    // Verify the operator private key has been written correctly
    let file_contents = fs::read(key_file_path).unwrap();
    let key_from_file = lcs::from_bytes(&file_contents).unwrap();
    let key_from_storage = storage.export_private_key(OPERATOR_KEY).unwrap();
    assert_eq!(key_from_storage, key_from_file);
}

#[test]
fn test_extract_public_key() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config_path = swarm.validator_swarm.config.config_files.first().unwrap();
    let node_config = NodeConfig::load(node_config_path.clone()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let storage: Storage = (&backend).try_into().unwrap();

    // Extract the operator key to file
    let key_file_path = node_config_path.with_file_name(OPERATOR_KEY);
    let _ = op_tool
        .extract_public_key(OPERATOR_KEY, key_file_path.to_str().unwrap(), &backend)
        .unwrap();

    // Verify the operator key has been written correctly
    let file_contents = fs::read(key_file_path).unwrap();
    let key_from_file = lcs::from_bytes(&file_contents).unwrap();
    let key_from_storage = storage.get_public_key(OPERATOR_KEY).unwrap().public_key;
    assert_eq!(key_from_storage, key_from_file);
}

#[test]
fn test_network_key_rotation() {
    let num_nodes = 5;
    let mut swarm = SmokeTestEnvironment::new(num_nodes);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);

    // Rotate the validator network key
    let (txn_ctx, new_network_key) = op_tool.rotate_validator_network_key(&backend).unwrap();
    wait_for_transaction_on_all_nodes(
        &swarm,
        num_nodes,
        txn_ctx.address,
        txn_ctx.sequence_number + 1,
    );

    // Verify that config has been loaded correctly with new key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_network_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool
        .validator_set(Some(validator_account), &backend)
        .unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    swarm.validator_swarm.kill_node(0);
    swarm.validator_swarm.add_node(0).unwrap();
}

#[test]
fn test_network_key_rotation_recovery() {
    let num_nodes = 5;
    let mut swarm = SmokeTestEnvironment::new(num_nodes);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);

    // Rotate the network key in storage manually and perform a key rotation using the op_tool.
    // Here, we expected the op_tool to see that the network key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let mut storage: Storage = (&backend).try_into().unwrap();
    let rotated_network_key = storage.rotate_key(VALIDATOR_NETWORK_KEY).unwrap();
    let (txn_ctx, new_network_key) = op_tool.rotate_validator_network_key(&backend).unwrap();
    assert_eq!(new_network_key, to_x25519(rotated_network_key).unwrap());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(
        &swarm,
        num_nodes,
        txn_ctx.address,
        txn_ctx.sequence_number + 1,
    );

    // Verify that config has been loaded correctly with new key
    let validator_account = node_config.validator_network.as_ref().unwrap().peer_id();
    let config_network_key = op_tool
        .validator_config(validator_account, &backend)
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool
        .validator_set(Some(validator_account), &backend)
        .unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    swarm.validator_swarm.kill_node(0);
    swarm.validator_swarm.add_node(0).unwrap();
}

#[test]
fn test_operator_key_rotation() {
    let mut swarm = SmokeTestEnvironment::new(5);
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
    let backend = load_backend_storage(&node_config);

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
        .validator_config(validator_account, &backend)
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);
}

#[test]
fn test_operator_key_rotation_recovery() {
    let mut swarm = SmokeTestEnvironment::new(5);
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
    let backend = load_backend_storage(&node_config);

    // Rotate the operator key
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend).unwrap();
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

    // Verify that the operator key was updated on-chain
    let mut storage: Storage = (&backend).try_into().unwrap();
    let operator_account = storage
        .get::<AccountAddress>(OPERATOR_ACCOUNT)
        .unwrap()
        .value;
    let account_resource = op_tool.account_resource(operator_account).unwrap();
    let on_chain_operator_key = hex::decode(account_resource.authentication_key).unwrap();
    assert_eq!(
        AuthenticationKey::ed25519(&new_operator_key),
        AuthenticationKey::try_from(on_chain_operator_key).unwrap()
    );

    // Rotate the operator key in storage manually and perform another rotation using the op tool.
    // Here, we expected the op_tool to see that the operator key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let rotated_operator_key = storage.rotate_key(OPERATOR_KEY).unwrap();
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend).unwrap();
    assert_eq!(rotated_operator_key, new_operator_key);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Verify that the transaction was executed correctly
    let result = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    let vm_status = result.unwrap();
    assert_eq!(VMStatusView::Executed, vm_status);

    // Verify that the operator key was updated on-chain
    let account_resource = op_tool.account_resource(operator_account).unwrap();
    let on_chain_operator_key = hex::decode(account_resource.authentication_key).unwrap();
    assert_eq!(
        AuthenticationKey::ed25519(&new_operator_key),
        AuthenticationKey::try_from(on_chain_operator_key).unwrap()
    );
}

#[test]
fn test_print_account() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let storage: Storage = (&backend).try_into().unwrap();

    // Print the owner account
    let op_tool_owner_account = op_tool.print_account(OWNER_ACCOUNT, &backend).unwrap();
    let storage_owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    assert_eq!(storage_owner_account, op_tool_owner_account);

    // Print the operator account
    let op_tool_operator_account = op_tool.print_account(OPERATOR_ACCOUNT, &backend).unwrap();
    let storage_operator_account = storage
        .get::<AccountAddress>(OPERATOR_ACCOUNT)
        .unwrap()
        .value;
    assert_eq!(storage_operator_account, op_tool_operator_account);
}

#[test]
fn test_validate_transaction() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);

    // Validate an unknown transaction and verify no VM state found
    let operator_account = op_tool.print_account(OPERATOR_ACCOUNT, &backend).unwrap();
    assert_eq!(
        None,
        op_tool
            .validate_transaction(operator_account, 1000)
            .unwrap()
    );

    // Submit a transaction (rotate the operator key) and validate the transaction execution
    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend).unwrap();
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(operator_account, txn_ctx.sequence_number + 1)
        .unwrap();

    let result = op_tool
        .validate_transaction(operator_account, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, result.unwrap());
}

#[test]
fn test_validator_config() {
    let mut swarm = SmokeTestEnvironment::new(1);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let mut storage: Storage = (&backend).try_into().unwrap();

    // Fetch the initial validator config for this operator's owner
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let consensus_key = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let original_validator_config = op_tool.validator_config(owner_account, &backend).unwrap();
    assert_eq!(
        consensus_key,
        original_validator_config.consensus_public_key
    );

    // Rotate the consensus key locally and update the validator network address using the config
    let new_consensus_key = storage.rotate_key(CONSENSUS_KEY).unwrap();
    let new_network_address = NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
    let txn_ctx = op_tool
        .set_validator_config(Some(new_network_address.clone()), None, &backend)
        .unwrap();

    // Wait for the transaction to be executed
    let mut client = swarm.get_validator_client(0, None);
    client
        .wait_for_transaction(txn_ctx.address, txn_ctx.sequence_number + 1)
        .unwrap();

    // Re-fetch the validator config and verify the changes
    let new_validator_config = op_tool.validator_config(owner_account, &backend).unwrap();
    assert_eq!(new_consensus_key, new_validator_config.consensus_public_key);
    assert!(new_validator_config
        .validator_network_address
        .to_string()
        .contains(&new_network_address.to_string()));
    assert_eq!(original_validator_config.name, new_validator_config.name);
    assert_eq!(
        original_validator_config.fullnode_network_address,
        new_validator_config.fullnode_network_address
    );
}

#[test]
fn test_validator_set() {
    let mut swarm = SmokeTestEnvironment::new(3);
    swarm.validator_swarm.launch();

    // Load a node config
    let node_config =
        NodeConfig::load(swarm.validator_swarm.config.config_files.first().unwrap()).unwrap();

    // Connect the operator tool to the first node's JSON RPC API
    let op_tool = swarm.get_op_tool(0);

    // Load validator's on disk storage
    let backend = load_backend_storage(&node_config);
    let mut storage: Storage = (&backend).try_into().unwrap();

    // Fetch the validator config and validator info for this operator's owner
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let validator_config = op_tool.validator_config(owner_account, &backend).unwrap();
    let validator_set_infos = op_tool
        .validator_set(Some(owner_account), &backend)
        .unwrap();
    assert_eq!(1, validator_set_infos.len());

    // Compare the validator config and the validator info
    let validator_info = validator_set_infos.first().unwrap();
    assert_eq!(validator_info.account_address, owner_account);
    assert_eq!(validator_info.name, validator_config.name);
    assert_eq!(
        validator_info.consensus_public_key,
        validator_config.consensus_public_key
    );
    assert_eq!(
        validator_info.validator_network_address,
        validator_config.validator_network_address
    );
    assert_eq!(
        validator_info.fullnode_network_address,
        validator_config.fullnode_network_address
    );

    // Fetch the entire validator set and check this account is included
    let validator_set_infos = op_tool.validator_set(None, &backend).unwrap();
    assert_eq!(3, validator_set_infos.len());
    let _ = validator_set_infos
        .iter()
        .find(|info| info.account_address == owner_account)
        .unwrap();

    // Overwrite the shared network encryption key in storage and verify that the
    // validator set can still be retrieved (but unable to decrypt the validator
    // network address)
    let _ = storage
        .set(VALIDATOR_NETWORK_ADDRESS_KEYS, "random string")
        .unwrap();
    let validator_set_infos = op_tool.validator_set(None, &backend).unwrap();
    assert_eq!(3, validator_set_infos.len());

    let validator_info = validator_set_infos
        .iter()
        .find(|info| info.account_address == owner_account)
        .unwrap();
    assert_eq!(
        validator_info.fullnode_network_address,
        validator_config.fullnode_network_address
    );
    assert_ne!(
        validator_info.validator_network_address,
        validator_config.validator_network_address
    );
}

fn wait_for_transaction_on_all_nodes(
    swarm: &SmokeTestEnvironment,
    num_nodes: usize,
    account: AccountAddress,
    sequence_number: u64,
) {
    for i in 0..num_nodes {
        let mut client = swarm.get_validator_client(i, None);
        client
            .wait_for_transaction(account, sequence_number)
            .unwrap();
    }
}
