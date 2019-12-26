// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.
use crate::{
    common::NetworkPublicKeys,
    proto::{ConsensusMsg, ConsensusMsg_oneof, MempoolSyncMsg, RequestBlock, RespondBlock},
    utils::MessageExt,
    validator_network::{
        self,
        network_builder::{NetworkBuilder, TransportType},
        Event,
    },
};
use futures::{future::join, StreamExt};
use libra_config::config::RoleType;
use libra_crypto::{ed25519::compat, test_utils::TEST_SEED, traits::ValidKey, x25519};
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    proto::types::SignedTransaction,
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    PeerId,
};
use parity_multiaddr::Multiaddr;
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    time::Duration,
};
use tokio::runtime::Runtime;

#[test]
fn test_network_builder() {
    let runtime = Runtime::new().unwrap();
    let peer_id = PeerId::random();
    let addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (signing_private_key, signing_public_key) = compat::generate_keypair(&mut rng);
    let (_identity_private_key, identity_public_key) = x25519::compat::generate_keypair(&mut rng);

    let mut network_builder =
        NetworkBuilder::new(runtime.handle().clone(), peer_id, addr, RoleType::Validator);
    network_builder
        .transport(TransportType::Memory)
        .signing_keys((signing_private_key, signing_public_key.clone()))
        .trusted_peers(
            vec![(
                peer_id,
                NetworkPublicKeys {
                    signing_public_key,
                    identity_public_key,
                },
            )]
            .into_iter()
            .collect(),
        )
        .channel_size(8);
    let (_mempool_network_sender, _mempool_network_events) =
        validator_network::mempool::add_to_network(&mut network_builder);
    let (_consensus_network_sender, _consensus_network_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let (_state_sync_network_sender, _state_sync_network_events) =
        validator_network::state_synchronizer::add_to_network(&mut network_builder);
    let _listen_addr = network_builder.build();
}

#[test]
fn test_mempool_sync() {
    ::libra_logger::try_init_for_testing();
    let mut runtime = Runtime::new().unwrap();

    // Setup peer ids.
    let listener_peer_id = PeerId::random();
    let dialer_peer_id = PeerId::random();
    // Setup signing public keys.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (dialer_signing_private_key, dialer_signing_public_key) =
        compat::generate_keypair(&mut rng);
    // Setup identity public keys.
    let (_listener_identity_private_key, listener_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);
    let (_dialer_identity_private_key, dialer_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    let trusted_peers: HashMap<_, _> = vec![
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key,
            },
        ),
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key,
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let listener_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .trusted_peers(trusted_peers.clone())
        .transport(TransportType::Memory)
        .channel_size(8);
    let (_, mut listener_mp_net_events) =
        validator_network::mempool::add_to_network(&mut network_builder);
    let listener_addr = network_builder.build();

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::Memory)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .trusted_peers(trusted_peers)
        .seed_peers(
            [(listener_peer_id, vec![listener_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .channel_size(8);
    let (mut dialer_mp_net_sender, mut dialer_mp_net_events) =
        validator_network::mempool::add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    // The dialer dials the listener and sends a mempool sync message
    let mut mempool_msg = MempoolSyncMsg::default();
    mempool_msg.peer_id = dialer_peer_id.into();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    let txn: SignedTransaction = get_test_signed_txn(sender, 0, &keypair.0, keypair.1, None)
        .try_into()
        .unwrap();
    mempool_msg.transactions.push(txn.clone());

    let f_dialer = async move {
        // Wait until dialing finished and NewPeer event received
        match dialer_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, listener_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // Dialer sends a mempool sync message
        dialer_mp_net_sender
            .send_to(listener_peer_id, mempool_msg)
            .await
            .unwrap();
    };

    // The listener receives a mempool sync message
    let f_listener = async move {
        // The listener receives a NewPeer event first
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, dialer_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // The listener then receives the mempool sync message
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::Message((peer_id, msg)) => {
                assert_eq!(peer_id, dialer_peer_id);
                let dialer_peer_id_bytes = Vec::from(&dialer_peer_id);
                assert_eq!(msg.peer_id, dialer_peer_id_bytes);
                let transactions: Vec<SignedTransaction> = msg.transactions;
                assert_eq!(transactions, vec![txn]);
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    runtime.block_on(join(f_dialer, f_listener));
}

// Test that an end-point operating with remote_authentication can connect to
// an end-point without remote_authentication if both are correctly configured.
#[test]
fn test_unauthenticated_remote_mempool_sync() {
    ::libra_logger::try_init_for_testing();
    let mut runtime = Runtime::new().unwrap();

    // Setup signing public keys.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (dialer_signing_private_key, dialer_signing_public_key) =
        compat::generate_keypair(&mut rng);
    // Setup public keys and peer ids.
    let (listener_identity_private_key, listener_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);
    let listener_peer_id = PeerId::try_from(listener_identity_public_key.to_bytes()).unwrap();
    let (dialer_identity_private_key, dialer_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);
    let dialer_peer_id = PeerId::try_from(dialer_identity_public_key.to_bytes()).unwrap();

    let trusted_peers: HashMap<_, _> = vec![
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key.clone(),
            },
        ),
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key.clone(),
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let listener_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .enable_remote_authentication(false)
        .transport(TransportType::PermissionlessMemoryNoise(Some((
            listener_identity_private_key,
            listener_identity_public_key,
        ))))
        .channel_size(8);
    let (_, mut listener_mp_net_events) =
        validator_network::mempool::add_to_network(&mut network_builder);
    let listener_addr = network_builder.build();

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::MemoryNoise(Some((
            dialer_identity_private_key,
            dialer_identity_public_key,
        ))))
        .trusted_peers(trusted_peers)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .seed_peers(
            [(listener_peer_id, vec![listener_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .channel_size(8);
    let (mut dialer_mp_net_sender, mut dialer_mp_net_events) =
        validator_network::mempool::add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    // The dialer dials the listener and sends a mempool sync message
    let mut mempool_msg = MempoolSyncMsg::default();
    mempool_msg.peer_id = dialer_peer_id.into();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    let txn: SignedTransaction = get_test_signed_txn(sender, 0, &keypair.0, keypair.1, None)
        .try_into()
        .unwrap();
    mempool_msg.transactions.push(txn.clone());

    let f_dialer = async move {
        // Wait until dialing finished and NewPeer event received
        match dialer_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, listener_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // Dialer sends a mempool sync message
        dialer_mp_net_sender
            .send_to(listener_peer_id, mempool_msg)
            .await
            .unwrap();
    };

    // The listener receives a mempool sync message
    let f_listener = async move {
        // The listener receives a NewPeer event first
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, dialer_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // The listener then receives the mempool sync message
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::Message((peer_id, msg)) => {
                assert_eq!(peer_id, dialer_peer_id);
                let dialer_peer_id_bytes = Vec::from(&dialer_peer_id);
                assert_eq!(msg.peer_id, dialer_peer_id_bytes);
                let transactions: Vec<SignedTransaction> = msg.transactions;
                assert_eq!(transactions, vec![txn]);
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    runtime.block_on(join(f_dialer, f_listener));
}

#[test]
fn test_consensus_rpc() {
    ::libra_logger::try_init_for_testing();
    let mut runtime = Runtime::new().unwrap();

    // Setup peer ids.
    let listener_peer_id = PeerId::random();
    let dialer_peer_id = PeerId::random();
    // Setup signing public keys.
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (listener_signing_private_key, listener_signing_public_key) =
        compat::generate_keypair(&mut rng);
    let (dialer_signing_private_key, dialer_signing_public_key) =
        compat::generate_keypair(&mut rng);
    // Setup identity public keys.
    let (_listener_identity_private_key, listener_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);
    let (_dialer_identity_private_key, dialer_identity_public_key) =
        x25519::compat::generate_keypair(&mut rng);

    let trusted_peers: HashMap<_, _> = vec![
        (
            listener_peer_id,
            NetworkPublicKeys {
                signing_public_key: listener_signing_public_key.clone(),
                identity_public_key: listener_identity_public_key,
            },
        ),
        (
            dialer_peer_id,
            NetworkPublicKeys {
                signing_public_key: dialer_signing_public_key.clone(),
                identity_public_key: dialer_identity_public_key,
            },
        ),
    ]
    .into_iter()
    .collect();

    // Set up the listener network
    let listener_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    );
    network_builder
        .signing_keys((listener_signing_private_key, listener_signing_public_key))
        .trusted_peers(trusted_peers.clone())
        .transport(TransportType::Memory)
        .channel_size(8);
    let (_, mut listener_con_net_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let listener_addr = network_builder.build();

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let mut network_builder = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    );
    network_builder
        .transport(TransportType::Memory)
        .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
        .trusted_peers(trusted_peers)
        .seed_peers(
            [(listener_peer_id, vec![listener_addr])]
                .iter()
                .cloned()
                .collect(),
        )
        .channel_size(8);
    let (mut dialer_con_net_sender, mut dialer_con_net_events) =
        validator_network::consensus::add_to_network(&mut network_builder);
    let _dialer_addr = network_builder.build();

    let req_block_msg = RequestBlock::default();

    let res_block_msg = RespondBlock::default();

    // The dialer dials the listener and sends a RequestBlock rpc request
    let req_block_msg_clone = req_block_msg.clone();
    let res_block_msg_clone = res_block_msg.clone();
    let f_dialer = async move {
        // Wait until dialing finished and NewPeer event received
        match dialer_con_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, listener_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // Dialer sends a RequestBlock rpc request.
        let res_block_msg = dialer_con_net_sender
            .request_block(
                listener_peer_id,
                req_block_msg_clone,
                Duration::from_secs(10),
            )
            .await
            .unwrap();
        assert_eq!(res_block_msg, res_block_msg_clone);
    };

    // The listener receives a RequestBlock rpc request and sends a RespondBlock
    // rpc response.
    let f_listener = async move {
        // The listener receives a NewPeer event first
        match listener_con_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, dialer_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }
        // Note: This is temporary.
        // Duplicate event since Consensus is registered as upstream for 2 protocols.
        match listener_con_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, dialer_peer_id);
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // The listener then handles the RequestBlock rpc request.
        match listener_con_net_events.next().await.unwrap().unwrap() {
            Event::RpcRequest((peer_id, req_msg, res_tx)) => {
                assert_eq!(peer_id, dialer_peer_id);

                // Check the request
                assert_eq!(
                    req_msg.message,
                    Some(ConsensusMsg_oneof::RequestBlock(req_block_msg))
                );

                // Send the serialized response back.
                let res_msg = ConsensusMsg {
                    message: Some(ConsensusMsg_oneof::RespondBlock(res_block_msg)),
                };
                let res_data = res_msg.to_bytes().unwrap();
                res_tx.send(Ok(res_data)).unwrap();
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    runtime.block_on(join(f_dialer, f_listener));
}
