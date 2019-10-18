// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    proto::{ErrorCode, GenerateKeyRequest, KeyType, PublicKeyRequest, SecretServiceClient},
    secret_service_client::ConsensusKeyManager,
    secret_service_node::SecretServiceNode,
};
use debug_interface::node_debug_helpers::{check_node_up, create_debug_client};
use grpcio::{ChannelBuilder, EnvBuilder};
use libra_config::config::{NodeConfig, NodeConfigHelpers};
use libra_crypto::hash::HashValue;
use libra_crypto::traits::Signature;
use libra_logger::prelude::*;
use std::{sync::Arc, thread};

/////////////////////////////////////////////////////////////////////////////////////
// These tests check interoperability of key_generation,                           //
// key_retrieval and signing for crate::secret_service_server::SecretServiceServer //
/////////////////////////////////////////////////////////////////////////////////////

#[test]
fn create_secret_service_node() {
    let node_config = NodeConfigHelpers::get_single_node_test_config(true);
    let _secret_service_node = SecretServiceNode::new(node_config);
}

fn create_client(public_port: u16) -> SecretServiceClient {
    let node_connection_str = format!("localhost:{}", public_port);
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&node_connection_str);
    SecretServiceClient::new(ch)
}

fn create_secret_service_node_and_client(node_config: NodeConfig) -> SecretServiceClient {
    let public_port = node_config.secret_service.secret_service_port;
    let debug_port = node_config.debug_interface.secret_service_node_debug_port;

    thread::spawn(move || {
        let secret_service_node = SecretServiceNode::new(node_config);
        secret_service_node.run().unwrap();
        info!("SecretService node stopped");
    });

    let debug_client = create_debug_client(debug_port);
    check_node_up(&debug_client);

    create_client(public_port)
}

// Testing higher level interface //
#[test]
fn test_generate_consensus_key() {
    let node_config = NodeConfigHelpers::get_single_node_test_config(true);
    let client = create_secret_service_node_and_client(node_config.clone());

    let key_manager = ConsensusKeyManager::new(Arc::new(client)).unwrap();
    let public_key = key_manager.get_consensus_public_key().unwrap();
    // let message_hash = b"consensus test message".digest(STANDARD_DIGESTER.get());
    let message_hash = HashValue::random();
    let signature = key_manager.sign_consensus_message(&message_hash).unwrap();
    // check that the signature verifies
    assert!(
        signature.verify(&message_hash, &public_key).is_ok(),
        "Correct signature does not verify"
    );
}

#[test]
fn test_generate_key() {
    let node_config = NodeConfigHelpers::get_single_node_test_config(true);
    let client = create_secret_service_node_and_client(node_config.clone());

    // create request to generate new key
    let mut gen_req: GenerateKeyRequest = GenerateKeyRequest::default();
    gen_req.set_spec(KeyType::Ed25519);

    let result = client.generate_key(&gen_req);
    if let Ok(response) = result {
        let is_successful = response.code() == ErrorCode::Success;
        assert!(is_successful);
    } else {
        panic!("key generation failed: {}", result.err().unwrap());
    }
}

#[test]
fn test_generate_and_retrieve_key() {
    let node_config = NodeConfigHelpers::get_single_node_test_config(true);
    let client = create_secret_service_node_and_client(node_config.clone());

    // create request to generate new key
    let mut gen_req: GenerateKeyRequest = GenerateKeyRequest::default();
    gen_req.set_spec(KeyType::Ed25519);

    let result = client.generate_key(&gen_req);
    if result.is_err() {
        panic!("key generation failed: {}", result.err().unwrap());
    }

    let response = result.ok().unwrap();
    let is_successful = response.code() == ErrorCode::Success;
    assert!(is_successful);
    let keyid = response.key_id;

    // assert_eq!(keyid, [230, 70, 121, 164, 224, 224, 49, 236, 222, 129, 71, 209, 108, 208, 39,
    // 161, 6, 166, 100, 236, 85, 0, 83, 224, 28, 229, 132, 230, 86, 31, 198, 235]);

    // existing key can be obtained
    let mut pk_req: PublicKeyRequest = PublicKeyRequest::default();
    pk_req.key_id = keyid.to_vec();

    let result = client.get_public_key(&pk_req);
    if result.is_err() {
        panic!("public key retrieval failed: {}", result.err().unwrap());
    }
    let response = result.ok().unwrap();
    let is_successful = response.code() == ErrorCode::Success;
    assert!(is_successful);
    let _result = response.public_key;

    // invalid length keyid argument returns an error
    let mut pk_req: PublicKeyRequest = PublicKeyRequest::default();
    pk_req.key_id = [0, 1, 2, 4].to_vec();

    let result = client.get_public_key(&pk_req);
    if result.is_err() {
        panic!("public key retrieval failed: {}", result.err().unwrap());
    }
    let response = result.ok().unwrap();
    let is_successful = response.code() == ErrorCode::WrongLength;
    assert!(is_successful);

    // obtaining non-existing key returns an error
    let mut pk_req: PublicKeyRequest = PublicKeyRequest::default();
    pk_req.key_id = [255; HashValue::LENGTH].to_vec();

    let result = client.get_public_key(&pk_req);
    if result.is_err() {
        panic!("public key retrieval failed: {}", result.err().unwrap());
    }
    let response = result.ok().unwrap();
    let is_successful = response.code() == ErrorCode::KeyIdNotFound;
    assert!(is_successful);
}
