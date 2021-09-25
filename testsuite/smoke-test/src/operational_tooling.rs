// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::new_local_swarm,
    test_utils::{
        diem_swarm_utils::{create_root_storage, load_validators_backend_storage},
        write_key_to_file_bcs_format, write_key_to_file_hex_format,
    },
};
use anyhow::{bail, Result};
use diem_config::{
    config::{PeerRole, SecureBackend},
    network_id::NetworkId,
};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    x25519, HashValue, PrivateKey, Uniform, ValidCryptoMaterialStringExt,
};
use diem_global_constants::{
    CONSENSUS_KEY, FULLNODE_NETWORK_KEY, GENESIS_WAYPOINT, OPERATOR_ACCOUNT, OPERATOR_KEY,
    OWNER_ACCOUNT, OWNER_KEY, VALIDATOR_NETWORK_ADDRESS_KEYS, VALIDATOR_NETWORK_KEY, WAYPOINT,
};
use diem_key_manager::diem_interface::{DiemInterface, JsonRpcDiemInterface};
use diem_management::storage::to_x25519;
use diem_operational_tool::{
    keys::{EncodingType, KeyType},
    test_helper::OperationalTool,
};
use diem_sdk::client::{views::VMStatusView, BlockingClient};
use diem_secure_storage::{CryptoStorage, KVStorage, Storage};
use diem_temppath::TempPath;
use diem_types::{
    account_address::{from_identity_public_key, AccountAddress},
    block_info::BlockInfo,
    ledger_info::LedgerInfo,
    network_address::NetworkAddress,
    transaction::authenticator::AuthenticationKey,
    waypoint::Waypoint,
};
use forge::{LocalNode, LocalSwarm, Node, NodeExt, SwarmExt};
use rand::rngs::OsRng;
use std::{
    collections::BTreeSet,
    convert::{TryFrom, TryInto},
    fs,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

#[test]
fn test_account_resource() {
    let (_env, op_tool, _, storage) = launch_swarm_with_op_tool_and_backend(1);

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
fn test_auto_validate_options() {
    let (swarm, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(1);

    // Rotate the operator key with a really low timeout to prevent validation
    let (txn_ctx, _) = op_tool
        .rotate_operator_key_with_custom_validation(&backend, false, Some(1), Some(0))
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Now wait for transaction execution
    let client = swarm.validators().next().unwrap().json_rpc_client();
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the transaction was executed correctly
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Rotate the operator key with a custom timeout of 1 minute and a a custom sleep interval
    let (txn_ctx, _) = op_tool
        .rotate_operator_key_with_custom_validation(&backend, false, Some(2), Some(60))
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());
}

#[test]
fn test_consensus_key_rotation() {
    let (_env, op_tool, backend, mut storage) = launch_swarm_with_op_tool_and_backend(1);

    // Rotate the consensus key
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend, false).unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_consensus_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);

    // Verify that the validator set info contains the new consensus key
    let info_consensus_key = op_tool
        .validator_set(Some(validator_account), Some(&backend))
        .unwrap()[0]
        .consensus_public_key
        .clone();
    assert_eq!(new_consensus_key, info_consensus_key);

    // Rotate the consensus key in storage manually and perform another rotation using the op_tool.
    // Here, we expected the op_tool to see that the consensus key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let rotated_consensus_key = storage.rotate_key(CONSENSUS_KEY).unwrap();
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend, true).unwrap();
    assert!(txn_ctx.execution_result.is_none());
    assert_eq!(rotated_consensus_key, new_consensus_key);
}

#[test]
fn test_create_operator_hex_file() {
    create_operator_with_file_writer(write_key_to_file_hex_format);
}

#[test]
fn test_create_operator_bcs_file() {
    create_operator_with_file_writer(write_key_to_file_bcs_format);
}

#[test]
fn test_create_validator_hex_file() {
    create_validator_with_file_writer(write_key_to_file_hex_format);
}

#[test]
fn test_create_validator_bcs_file() {
    create_validator_with_file_writer(write_key_to_file_bcs_format);
}

#[test]
fn test_disable_address_validation() {
    let num_nodes = 1;
    let (_env, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Try to set the validator config with a bad address and verify failure
    let bad_network_address = NetworkAddress::from_str("/dns4/127.0.0.1/tcp/1234").unwrap();
    op_tool
        .set_validator_config(
            Some(bad_network_address.clone()),
            None,
            &backend,
            false,
            false,
        )
        .unwrap_err();

    // Now disable address verification to set the validator config with a bad network address
    let txn_ctx = op_tool
        .set_validator_config(Some(bad_network_address), None, &backend, false, true)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Rotate the consensus key and verify that it isn't blocked by a bad network address
    let _ = op_tool.rotate_consensus_key(&backend, false).unwrap();

    // Rotate the validator network key and verify that it isn't blocked by a bad network address
    let _ = op_tool
        .rotate_validator_network_key(&backend, false)
        .unwrap();

    // Rotate the fullnode network key and verify that it isn't blocked by a bad network address
    let _ = op_tool
        .rotate_fullnode_network_key(&backend, false)
        .unwrap();

    // Rotate the operator key and verify that it isn't blocked by a bad network address
    let _ = op_tool.rotate_operator_key(&backend, false).unwrap();

    // Update the validator network address with a valid address
    let new_network_address = NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
    let _ = op_tool
        .set_validator_config(Some(new_network_address), None, &backend, false, false)
        .unwrap();
}

#[test]
fn test_set_operator_and_add_new_validator() {
    let num_nodes = 3;
    let (mut swarm, op_tool, _, _) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Create new validator and validator operator keys and accounts
    let (validator_key, validator_account) = create_new_test_account();
    let (operator_key, operator_account) = create_new_test_account();

    // Write the validator key to a file and create the validator account
    let validator_key_path = write_key_to_file(
        &validator_key.public_key(),
        swarm.validators().next().unwrap(),
        write_key_to_file_hex_format,
    );
    let diem_backend = create_root_storage(&mut swarm);
    let val_human_name = "new_validator";
    let (txn_ctx, _) = op_tool
        .create_validator(
            val_human_name,
            validator_key_path.to_str().unwrap(),
            &diem_backend,
            false,
        )
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    let validator = swarm.validators().next().unwrap();
    // Write the operator key to a file and create the operator account
    let operator_key_path = write_key_to_file(
        &operator_key.public_key(),
        validator,
        write_key_to_file_bcs_format,
    );
    let op_human_name = "new_operator";
    let (txn_ctx, _) = op_tool
        .create_validator_operator(
            op_human_name,
            operator_key_path.to_str().unwrap(),
            &diem_backend,
            true,
        )
        .unwrap();

    // Wait for transaction execution
    let client = validator.json_rpc_client();
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the transaction was executed
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Overwrite the keys in storage to execute the command from the new validator's perspective
    let backend = load_validators_backend_storage(validator);
    let mut storage: Storage = (&backend).try_into().unwrap();
    storage.set(OWNER_ACCOUNT, validator_account).unwrap();
    storage
        .import_private_key(OWNER_KEY, validator_key)
        .unwrap();

    // Verify no validator operator
    let diem_json_rpc = JsonRpcDiemInterface::new(validator.json_rpc_endpoint().to_string());
    let account_state = diem_json_rpc
        .retrieve_account_state(validator_account)
        .unwrap();
    let val_config_resource = account_state
        .get_validator_config_resource()
        .unwrap()
        .unwrap();
    assert!(val_config_resource.delegated_account.is_none());
    assert!(val_config_resource.validator_config.is_none());

    // Set the validator operator
    let txn_ctx = op_tool
        .set_validator_operator(op_human_name, operator_account, &backend, true)
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Wait for transaction execution
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify the operator has been set correctly
    let account_state = diem_json_rpc
        .retrieve_account_state(validator_account)
        .unwrap();
    let val_config_resource = account_state
        .get_validator_config_resource()
        .unwrap()
        .unwrap();
    assert_eq!(
        operator_account,
        val_config_resource.delegated_account.unwrap()
    );
    assert!(val_config_resource.validator_config.is_none());

    // Overwrite the keys in storage to execute the command from the new operator's perspective
    storage.set(OPERATOR_ACCOUNT, operator_account).unwrap();
    storage
        .import_private_key(OPERATOR_KEY, operator_key)
        .unwrap();

    // Set the validator config
    let network_address = Some(NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap());
    let txn_ctx = op_tool
        .set_validator_config(
            network_address.clone(),
            network_address,
            &backend,
            true,
            false,
        )
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Wait for transaction execution
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Check the validator set size
    let validator_set_infos = op_tool.validator_set(None, Some(&backend)).unwrap();
    assert_eq!(num_nodes, validator_set_infos.len());
    assert!(!validator_set_infos
        .iter()
        .any(|info| info.account_address == validator_account));

    // Add the validator to the validator set
    let txn_ctx = op_tool
        .add_validator(validator_account, &diem_backend, true)
        .unwrap();

    // Wait for transaction execution
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();
    // Verify that the transaction wasn't executed
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Check the new validator has been added to the set
    let validator_set_infos = op_tool.validator_set(None, Some(&backend)).unwrap();
    assert_eq!(num_nodes + 1, validator_set_infos.len());
    let validator_info = validator_set_infos
        .iter()
        .find(|info| info.account_address == validator_account)
        .unwrap();
    assert_eq!(validator_account, validator_info.account_address);
    assert_eq!(val_human_name, validator_info.name);

    // Try and add the same validator again and watch it fail
    let txn_ctx = op_tool
        .add_validator(validator_account, &diem_backend, false)
        .unwrap();
    let status = txn_ctx.execution_result.unwrap();

    // Make sure this was the error that we were expecting and that the error is explained
    // correctly.
    match status {
        VMStatusView::MoveAbort { explanation, .. } => {
            assert!(
                explanation.is_some(),
                "Move abort explanation for known abort code not found"
            );
            let explanation = explanation.unwrap();
            assert_eq!(explanation.category, "INVALID_ARGUMENT");
            assert_eq!(explanation.reason, "EALREADY_A_VALIDATOR");
        }
        _ => panic!("Unexpected VM status when trying to double-add validator"),
    }
}

#[test]
fn test_extract_private_key() {
    let (swarm, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

    // Extract the operator private key to file
    let node_config_path = swarm.validators().next().unwrap().config_path();
    let key_file_path = node_config_path.with_file_name(OPERATOR_KEY);
    let _ = op_tool
        .extract_private_key(
            OPERATOR_KEY,
            key_file_path.to_str().unwrap(),
            KeyType::Ed25519,
            EncodingType::BCS,
            &backend,
        )
        .unwrap();

    // Verify the operator private key has been written correctly
    let file_contents = fs::read(key_file_path).unwrap();
    let key_from_file = bcs::from_bytes(&file_contents).unwrap();
    let key_from_storage = storage.export_private_key(OPERATOR_KEY).unwrap();
    assert_eq!(key_from_storage, key_from_file);
}

#[test]
fn test_extract_public_key() {
    let (swarm, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

    // Extract the operator public key to file
    let node_config_path = swarm.validators().next().unwrap().config_path();
    let key_file_path = node_config_path.with_file_name(OPERATOR_KEY);
    let _ = op_tool
        .extract_public_key(
            OPERATOR_KEY,
            key_file_path.to_str().unwrap(),
            KeyType::Ed25519,
            EncodingType::BCS,
            &backend,
        )
        .unwrap();

    // Verify the operator key has been written correctly
    let file_contents = fs::read(key_file_path).unwrap();
    let key_from_file = bcs::from_bytes(&file_contents).unwrap();
    let key_from_storage = storage.get_public_key(OPERATOR_KEY).unwrap().public_key;
    assert_eq!(key_from_storage, key_from_file);
}

#[test]
fn test_extract_peer_from_storage() {
    let (swarm, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(1);

    // Check Validator Network Key
    let config = swarm.validators().next().unwrap().config().clone();
    let map = op_tool
        .extract_peer_from_storage(VALIDATOR_NETWORK_KEY, &backend)
        .unwrap();
    let network_config = config.validator_network.unwrap();
    let expected_peer_id = network_config.peer_id();
    let expected_public_key = network_config.identity_key().public_key();
    let (peer_id, peer) = map.iter().next().unwrap();
    assert_eq!(expected_public_key, *peer.keys.iter().next().unwrap());
    assert_eq!(expected_peer_id, *peer_id);

    // Check FullNode Network Key
    let map = op_tool
        .extract_peer_from_storage(FULLNODE_NETWORK_KEY, &backend)
        .unwrap();
    let network_config = config
        .full_node_networks
        .iter()
        .find(|network| network.network_id == NetworkId::Public)
        .unwrap();
    let expected_peer_id = network_config.peer_id();
    let expected_public_key = network_config.identity_key().public_key();
    let (peer_id, peer) = map.iter().next().unwrap();
    assert_eq!(expected_public_key, *peer.keys.iter().next().unwrap());
    assert_eq!(expected_peer_id, *peer_id);
}

#[test]
fn test_extract_peer_from_file() {
    let op_tool = OperationalTool::test();
    let path = TempPath::new();
    path.create_as_file().unwrap();
    let key = op_tool
        .generate_key(KeyType::X25519, path.as_ref(), EncodingType::Hex)
        .unwrap();

    let peer = op_tool
        .extract_peer_from_file(path.as_ref(), EncodingType::Hex)
        .unwrap();
    assert_eq!(1, peer.len());
    let (peer_id, peer) = peer.iter().next().unwrap();
    let public_key = key.public_key();
    assert_eq!(public_key, *peer.keys.iter().next().unwrap());
    assert_eq!(from_identity_public_key(public_key), *peer_id);
}

#[test]
fn test_extract_peers_from_keys() {
    let op_tool = OperationalTool::test();
    let output_path = TempPath::new();
    output_path.create_as_file().unwrap();

    let mut keys = BTreeSet::new();
    for _ in 1..10 {
        let key_path = TempPath::new();
        key_path.create_as_file().unwrap();
        keys.insert(
            op_tool
                .generate_key(KeyType::X25519, key_path.as_ref(), EncodingType::Hex)
                .unwrap()
                .public_key(),
        );
    }
    let peers = op_tool
        .extract_peers_from_keys(keys.clone(), output_path.as_ref())
        .unwrap();
    assert_eq!(keys.len(), peers.len());
    for key in keys {
        let address = from_identity_public_key(key);
        let peer = peers.get(&address).unwrap();
        let keys = &peer.keys;

        assert_eq!(1, keys.len());
        assert!(keys.contains(&key));
        assert_eq!(PeerRole::Downstream, peer.role);
        assert!(peer.addresses.is_empty());
    }
}

#[test]
fn test_generate_key() {
    let op_tool = OperationalTool::test();
    let path = TempPath::new();
    path.create_as_file().unwrap();

    // Base64
    let expected_key = op_tool
        .generate_key(KeyType::X25519, path.as_ref(), EncodingType::Base64)
        .unwrap();
    assert_eq!(
        expected_key,
        x25519::PrivateKey::try_from(
            base64::decode(fs::read(path.as_ref()).unwrap())
                .unwrap()
                .as_slice()
        )
        .unwrap(),
    );

    // Hex
    let expected_key = op_tool
        .generate_key(KeyType::X25519, path.as_ref(), EncodingType::Hex)
        .unwrap();
    assert_eq!(
        expected_key,
        x25519::PrivateKey::from_encoded_string(
            &String::from_utf8(fs::read(path.as_ref()).unwrap()).unwrap()
        )
        .unwrap()
    );

    // BCS
    let expected_key = op_tool
        .generate_key(KeyType::X25519, path.as_ref(), EncodingType::BCS)
        .unwrap();
    assert_eq!(
        expected_key,
        bcs::from_bytes(&fs::read(path.as_ref()).unwrap()).unwrap()
    );
}

#[test]
fn test_insert_waypoint() {
    let (_env, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

    // Get the current waypoint from storage
    let current_waypoint: Waypoint = storage.get(WAYPOINT).unwrap().value;

    // Insert a new waypoint and genesis waypoint into storage
    let inserted_waypoint =
        Waypoint::new_any(&LedgerInfo::new(BlockInfo::empty(), HashValue::zero()));
    assert_ne!(current_waypoint, inserted_waypoint);
    op_tool
        .insert_waypoint(inserted_waypoint, &backend, true)
        .unwrap();

    // Verify the waypoint has changed in storage and that genesis waypoint is now set
    assert_eq!(inserted_waypoint, storage.get(WAYPOINT).unwrap().value);
    assert_eq!(
        inserted_waypoint,
        storage.get(GENESIS_WAYPOINT).unwrap().value
    );

    // Insert the old waypoint into storage, but skip the genesis waypoint
    op_tool
        .insert_waypoint(current_waypoint, &backend, false)
        .unwrap();
    assert_eq!(current_waypoint, storage.get(WAYPOINT).unwrap().value);
    assert_eq!(
        inserted_waypoint,
        storage.get(GENESIS_WAYPOINT).unwrap().value
    );
}

#[test]
fn test_fullnode_network_key_rotation() {
    let num_nodes = 1;
    let (env, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Rotate the full node network key
    let (txn_ctx, new_network_key) = op_tool.rotate_fullnode_network_key(&backend, true).unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Wait for transaction execution
    let client = env.validators().next().unwrap().json_rpc_client();
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the config has been loaded correctly with new key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_network_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .fullnode_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool
        .validator_set(Some(validator_account), Some(&backend))
        .unwrap()[0]
        .fullnode_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);
}

#[test]
fn test_network_key_rotation() {
    let num_nodes = 4;
    let (mut swarm, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Rotate the validator network key
    let (txn_ctx, new_network_key) = op_tool
        .rotate_validator_network_key(&backend, true)
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(&swarm, txn_ctx.address, txn_ctx.sequence_number);

    // Verify that config has been loaded correctly with new key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_network_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool
        .validator_set(Some(validator_account), Some(&backend))
        .unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    let validator = swarm.validators_mut().next().unwrap();
    validator.stop();
    validator.start().unwrap();
    swarm
        .wait_for_connectivity(Instant::now() + Duration::from_secs(60))
        .unwrap();
}

#[test]
fn test_network_key_rotation_recovery() {
    let num_nodes = 4;
    let (mut swarm, op_tool, backend, mut storage) =
        launch_swarm_with_op_tool_and_backend(num_nodes);

    // Rotate the network key in storage manually and perform a key rotation using the op_tool.
    // Here, we expected the op_tool to see that the network key in storage doesn't match the one
    // on-chain, and thus it should simply forward a transaction to the blockchain.
    let rotated_network_key = storage.rotate_key(VALIDATOR_NETWORK_KEY).unwrap();
    let (txn_ctx, new_network_key) = op_tool
        .rotate_validator_network_key(&backend, true)
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());
    assert_eq!(new_network_key, to_x25519(rotated_network_key).unwrap());

    // Ensure all nodes have received the transaction
    wait_for_transaction_on_all_nodes(&swarm, txn_ctx.address, txn_ctx.sequence_number);

    // Verify that config has been loaded correctly with new key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_network_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, config_network_key);

    // Verify that the validator set info contains the new network key
    let info_network_key = op_tool
        .validator_set(Some(validator_account), Some(&backend))
        .unwrap()[0]
        .validator_network_address
        .find_noise_proto()
        .unwrap();
    assert_eq!(new_network_key, info_network_key);

    // Restart validator
    // At this point, the `add_node` call ensures connectivity to all nodes
    let validator = swarm.validators_mut().next().unwrap();
    validator.stop();
    validator.start().unwrap();
    swarm
        .wait_for_connectivity(Instant::now() + Duration::from_secs(60))
        .unwrap();
}

#[test]
fn test_operator_key_rotation() {
    let (env, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, true).unwrap();
    assert!(txn_ctx.execution_result.is_none());

    // Wait for transaction execution
    let client = env.validators().next().unwrap().json_rpc_client();
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the transaction was executed correctly
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Rotate the consensus key to verify the operator key has been updated
    let (txn_ctx, new_consensus_key) = op_tool.rotate_consensus_key(&backend, false).unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify that the config has been updated correctly with the new consensus key
    let validator_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let config_consensus_key = op_tool
        .validator_config(validator_account, Some(&backend))
        .unwrap()
        .consensus_public_key;
    assert_eq!(new_consensus_key, config_consensus_key);
}

#[test]
fn test_operator_key_rotation_recovery() {
    let (env, op_tool, backend, mut storage) = launch_swarm_with_op_tool_and_backend(1);

    // Rotate the operator key
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend, false).unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify that the transaction was executed correctly
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify that the operator key was updated on-chain
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
    let (txn_ctx, new_operator_key) = op_tool.rotate_operator_key(&backend, true).unwrap();
    assert!(txn_ctx.execution_result.is_none());
    assert_eq!(rotated_operator_key, new_operator_key);

    // Wait for transaction execution
    let client = env.validators().next().unwrap().json_rpc_client();
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the transaction was executed correctly
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

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
    let (_env, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

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
fn test_print_key() {
    let (_env, op_tool, backend, storage) = launch_swarm_with_op_tool_and_backend(1);

    // Print the operator key
    let op_tool_operator_key = op_tool.print_key(OPERATOR_KEY, &backend).unwrap();
    let storage_operator_key = storage.get_public_key(OPERATOR_KEY).unwrap().public_key;
    assert_eq!(storage_operator_key, op_tool_operator_key);

    // Print the consensus key
    let op_tool_consensus_key = op_tool.print_key(CONSENSUS_KEY, &backend).unwrap();
    let storage_consensus_key = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    assert_eq!(storage_consensus_key, op_tool_consensus_key);
}

#[test]
fn test_print_waypoints() {
    let (_env, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(1);

    // Insert a new waypoint and genesis waypoint into storage
    let inserted_waypoint =
        Waypoint::new_any(&LedgerInfo::new(BlockInfo::empty(), HashValue::zero()));
    op_tool
        .insert_waypoint(inserted_waypoint, &backend, true)
        .unwrap();

    // Print the waypoint
    let waypoint = op_tool.print_waypoint(WAYPOINT, &backend).unwrap();
    assert_eq!(inserted_waypoint, waypoint);

    // Print the gensis waypoint
    let genesis_waypoint = op_tool.print_waypoint(GENESIS_WAYPOINT, &backend).unwrap();
    assert_eq!(inserted_waypoint, genesis_waypoint);
}

#[test]
fn test_validate_transaction() {
    let (swarm, op_tool, backend, _) = launch_swarm_with_op_tool_and_backend(1);

    // Validate an unknown transaction and verify no VM state found
    let operator_account = op_tool.print_account(OPERATOR_ACCOUNT, &backend).unwrap();
    assert_eq!(
        None,
        op_tool
            .validate_transaction(operator_account, 1000)
            .unwrap()
            .execution_result
    );

    // Submit a transaction (rotate the operator key) and validate the transaction execution
    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, true).unwrap();
    let client = swarm.validators().next().unwrap().json_rpc_client();
    wait_for_account_sequence_number(&client, operator_account, txn_ctx.sequence_number).unwrap();

    let result = op_tool
        .validate_transaction(operator_account, txn_ctx.sequence_number)
        .unwrap()
        .execution_result;
    assert_eq!(VMStatusView::Executed, result.unwrap());

    // Submit a transaction with auto validation (rotate the operator key) and compare results
    let (txn_ctx, _) = op_tool.rotate_operator_key(&backend, false).unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    let result = op_tool
        .validate_transaction(operator_account, txn_ctx.sequence_number)
        .unwrap()
        .execution_result;
    assert_eq!(VMStatusView::Executed, result.unwrap());
}

#[test]
fn test_validator_config() {
    let (_env, op_tool, backend, mut storage) = launch_swarm_with_op_tool_and_backend(1);

    // Fetch the initial validator config for this operator's owner
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let consensus_key = storage.get_public_key(CONSENSUS_KEY).unwrap().public_key;
    let original_validator_config = op_tool
        .validator_config(owner_account, Some(&backend))
        .unwrap();
    assert_eq!(
        consensus_key,
        original_validator_config.consensus_public_key
    );

    // Rotate the consensus key locally and update the validator network address using the config
    let new_consensus_key = storage.rotate_key(CONSENSUS_KEY).unwrap();
    let new_network_address = NetworkAddress::from_str("/ip4/10.0.0.16/tcp/80").unwrap();
    let txn_ctx = op_tool
        .set_validator_config(
            Some(new_network_address.clone()),
            None,
            &backend,
            false,
            false,
        )
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Re-fetch the validator config and verify the changes
    let new_validator_config = op_tool
        .validator_config(owner_account, Some(&backend))
        .unwrap();
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
fn test_validator_decryption() {
    let (_env, op_tool, backend, mut storage) = launch_swarm_with_op_tool_and_backend(1);

    // Fetch the validator config and validator info for this operator's owner
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let validator_config = op_tool
        .validator_config(owner_account, Some(&backend))
        .unwrap();
    let validator_set_infos = op_tool
        .validator_set(Some(owner_account), Some(&backend))
        .unwrap();
    assert_eq!(1, validator_set_infos.len());

    // Ensure the validator network addresses were decrypted successfully
    let failed_decryption_address = NetworkAddress::from_str("/dns4/could-not-decrypt").unwrap();
    let config_network_address = validator_config.validator_network_address;
    let info_network_address = validator_set_infos[0].validator_network_address.clone();
    assert_eq!(config_network_address, info_network_address,);
    assert_ne!(failed_decryption_address, config_network_address);

    // Corrupt the network address encryption key in storage
    storage
        .set(VALIDATOR_NETWORK_ADDRESS_KEYS, "INVALID KEY")
        .unwrap();

    // Verify that any failure to decrypt the address will still produce a result
    for backend in &[Some(&backend), None] {
        // Fetch the validator config and validator info for this operator's owner
        let validator_config = op_tool.validator_config(owner_account, *backend).unwrap();
        let validator_set_infos = op_tool
            .validator_set(Some(owner_account), *backend)
            .unwrap();

        // Ensure the validator network addresses failed to decrypt, but everything else was fetched
        let config_network_address = validator_config.validator_network_address;
        let info_network_address = validator_set_infos[0].validator_network_address.clone();
        assert_eq!(config_network_address, info_network_address);
        assert_eq!(failed_decryption_address, config_network_address);
    }
}

#[test]
fn test_validator_set() {
    let num_nodes = 4;
    let (_env, op_tool, backend, mut storage) = launch_swarm_with_op_tool_and_backend(num_nodes);

    // Fetch the validator config and validator info for this operator's owner
    let owner_account = storage.get::<AccountAddress>(OWNER_ACCOUNT).unwrap().value;
    let validator_config = op_tool
        .validator_config(owner_account, Some(&backend))
        .unwrap();
    let validator_set_infos = op_tool
        .validator_set(Some(owner_account), Some(&backend))
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
    let validator_set_infos = op_tool.validator_set(None, Some(&backend)).unwrap();
    assert_eq!(num_nodes, validator_set_infos.len());
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
    let validator_set_infos = op_tool.validator_set(None, Some(&backend)).unwrap();
    assert_eq!(num_nodes, validator_set_infos.len());

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

/// Creates a new account address and key for testing.
fn create_new_test_account() -> (Ed25519PrivateKey, AccountAddress) {
    let mut rng = OsRng;
    let key = Ed25519PrivateKey::generate(&mut rng);
    let auth_key = AuthenticationKey::ed25519(&key.public_key());
    let account = auth_key.derived_address();
    (key, account)
}

/// Creates a new validator operator using the given file writer and verifies
/// the operator account is correctly initialized on-chain.
fn create_operator_with_file_writer(file_writer: fn(&Ed25519PublicKey, PathBuf)) {
    let (mut swarm, op_tool, _, _) = launch_swarm_with_op_tool_and_backend(1);

    // Create a new operator key and account
    let (operator_key, operator_account) = create_new_test_account();

    let validator = swarm.validators().next().unwrap();
    // Verify the corresponding account doesn't exist on-chain
    let diem_json_rpc = JsonRpcDiemInterface::new(validator.json_rpc_endpoint().to_string());
    diem_json_rpc
        .retrieve_account_state(operator_account)
        .unwrap_err();

    // Write the key to a file using the provided file writer
    let key_file_path = write_key_to_file(&operator_key.public_key(), validator, file_writer);

    // Create the operator account
    let backend = create_root_storage(&mut swarm);
    let op_human_name = "new_operator";
    let (txn_ctx, account_address) = op_tool
        .create_validator_operator(
            op_human_name,
            key_file_path.to_str().unwrap(),
            &backend,
            false,
        )
        .unwrap();
    assert_eq!(operator_account, account_address);
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify the operator account now exists on-chain
    let account_state = diem_json_rpc
        .retrieve_account_state(operator_account)
        .unwrap();
    let op_config_resource = account_state
        .get_validator_operator_config_resource()
        .unwrap()
        .unwrap();
    assert_eq!(op_human_name.as_bytes(), op_config_resource.human_name);
}

/// Creates a new validator using the given file writer and verifies
/// the account is correctly initialized on-chain.
fn create_validator_with_file_writer(file_writer: fn(&Ed25519PublicKey, PathBuf)) {
    let (mut swarm, op_tool, _, _) = launch_swarm_with_op_tool_and_backend(1);

    // Create a new validator key and account
    let (validator_key, validator_account) = create_new_test_account();

    let validator = swarm.validators().next().unwrap();
    let client = validator.json_rpc_client();
    // Verify the corresponding account doesn't exist on-chain
    let diem_json_rpc = JsonRpcDiemInterface::new(validator.json_rpc_endpoint().to_string());
    diem_json_rpc
        .retrieve_account_state(validator_account)
        .unwrap_err();

    // Write the key to a file using the provided file writer
    let key_file_path = write_key_to_file(&validator_key.public_key(), validator, file_writer);

    // Create the validator account
    let backend = create_root_storage(&mut swarm);
    let val_human_name = "new_validator";
    let (txn_ctx, account_address) = op_tool
        .create_validator(
            val_human_name,
            key_file_path.to_str().unwrap(),
            &backend,
            true,
        )
        .unwrap();
    assert!(txn_ctx.execution_result.is_none());
    assert_eq!(validator_account, account_address);

    // Wait for transaction execution
    wait_for_account_sequence_number(&client, txn_ctx.address, txn_ctx.sequence_number).unwrap();

    // Verify that the transaction was executed
    let txn_ctx = op_tool
        .validate_transaction(txn_ctx.address, txn_ctx.sequence_number)
        .unwrap();
    assert_eq!(VMStatusView::Executed, txn_ctx.execution_result.unwrap());

    // Verify the validator account now exists on-chain
    let account_state = diem_json_rpc
        .retrieve_account_state(validator_account)
        .unwrap();
    let val_config_resource = account_state
        .get_validator_config_resource()
        .unwrap()
        .unwrap();
    assert_eq!(val_human_name.as_bytes(), val_config_resource.human_name);
    assert!(val_config_resource.delegated_account.is_none());
    assert!(val_config_resource.validator_config.is_none());
}

/// Launches a validator swarm of a specified size, connects an operational
/// tool to the first node and fetches that node's secure backend.
pub fn launch_swarm_with_op_tool_and_backend(
    num_nodes: usize,
) -> (LocalSwarm, OperationalTool, SecureBackend, Storage) {
    let swarm = new_local_swarm(num_nodes);
    let chain_id = swarm.chain_id();
    let validator = swarm.validators().next().unwrap();

    // Connect the operator tool to the node's JSON RPC API
    let op_tool = OperationalTool::new(validator.json_rpc_endpoint().to_string(), chain_id);

    // Load validator's on disk storage
    let backend = load_validators_backend_storage(validator);
    let storage: Storage = (&backend).try_into().unwrap();

    (swarm, op_tool, backend, storage)
}

/// Writes a given key to file using a specified file writer and test environment.
fn write_key_to_file(
    key: &Ed25519PublicKey,
    node: &LocalNode,
    file_writer: fn(&Ed25519PublicKey, PathBuf),
) -> PathBuf {
    let node_config_path = node.config_path();
    let file_path = node_config_path.with_file_name("KEY_FILE");
    file_writer(key, file_path.clone());
    file_path
}

fn wait_for_account_sequence_number(
    client: &BlockingClient,
    address: AccountAddress,
    seq: u64,
) -> Result<()> {
    const DEFAULT_WAIT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

    let start = std::time::Instant::now();
    while start.elapsed() < DEFAULT_WAIT_TIMEOUT {
        let account_txn = client
            .get_account_transaction(address, seq, false)?
            .into_inner();
        if let Some(txn) = account_txn {
            if let diem_sdk::client::views::TransactionDataView::UserTransaction {
                sequence_number,
                ..
            } = txn.transaction
            {
                if sequence_number >= seq {
                    return Ok(());
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    bail!(
        "wait for account(address={}) transaction(seq={}) timeout",
        address,
        seq
    )
}

pub fn wait_for_transaction_on_all_nodes(
    swarm: &LocalSwarm,
    account: AccountAddress,
    sequence_number: u64,
) {
    for validator in swarm.validators() {
        let client = validator.json_rpc_client();
        wait_for_account_sequence_number(&client, account, sequence_number).unwrap();
    }
}
