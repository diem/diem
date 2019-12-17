// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.
use crate::{
    common::NetworkPublicKeys,
    proto::{
        BroadcastTransactionsRequest, BroadcastTransactionsResponse, ConsensusMsg,
        ConsensusMsg_oneof, MempoolSyncMsg, MempoolSyncMsg_oneof, RequestBlock, RespondBlock,
    },
    utils::MessageExt,
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        Event, CONSENSUS_RPC_PROTOCOL, MEMPOOL_RPC_PROTOCOL,
    },
    ProtocolId,
};

use assert_matches;
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
    let mempool_sync_protocol = ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL);
    let consensus_get_blocks_protocol = ProtocolId::from_static(b"get_blocks");
    let synchronizer_get_chunks_protocol = ProtocolId::from_static(b"get_chunks");
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (signing_private_key, signing_public_key) = compat::generate_keypair(&mut rng);
    let (_identity_private_key, identity_public_key) = x25519::compat::generate_keypair(&mut rng);

    let (_listen_addr, mut network_provider) =
        NetworkBuilder::new(runtime.handle().clone(), peer_id, addr, RoleType::Validator)
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
            .channel_size(8)
            .rpc_protocols(vec![
                consensus_get_blocks_protocol.clone(),
                mempool_sync_protocol.clone(),
                synchronizer_get_chunks_protocol.clone(),
            ])
            .build();
    let (_mempool_network_sender, _mempool_network_events) =
        network_provider.add_mempool(vec![mempool_sync_protocol.clone()]);
    let (_consensus_network_sender, _consensus_network_events) =
        network_provider.add_consensus(vec![consensus_get_blocks_protocol.clone()]);
    let (_state_sync_network_sender, _state_sync_network_events) =
        network_provider.add_state_synchronizer(vec![synchronizer_get_chunks_protocol.clone()]);
    runtime.spawn(network_provider.start());
}

#[test]
fn test_mempool_sync() {
    ::libra_logger::try_init_for_testing();
    let mut runtime = Runtime::new().unwrap();
    let mempool_sync_protocol = ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL);

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
    let (listener_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    )
    .signing_keys((listener_signing_private_key, listener_signing_public_key))
    .trusted_peers(trusted_peers.clone())
    .transport(TransportType::Memory)
    .channel_size(8)
    .rpc_protocols(vec![mempool_sync_protocol.clone()])
    .build();
    let (_, mut listener_mp_net_events) =
        network_provider.add_mempool(vec![mempool_sync_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    )
    .transport(TransportType::Memory)
    .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
    .trusted_peers(trusted_peers.clone())
    .seed_peers(
        [(listener_peer_id, vec![listener_addr])]
            .iter()
            .cloned()
            .collect(),
    )
    .channel_size(8)
    .rpc_protocols(vec![mempool_sync_protocol.clone()])
    .build();
    let (mut dialer_mp_net_sender, mut dialer_mp_net_events) =
        network_provider.add_mempool(vec![mempool_sync_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

    // The dialer dials the listener and sends a mempool sync message
    let mut submit_txns_req = BroadcastTransactionsRequest::default();
    submit_txns_req.peer_id = dialer_peer_id.into();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    let txn: SignedTransaction = get_test_signed_txn(sender, 0, keypair.0, keypair.1, None)
        .try_into()
        .unwrap();
    submit_txns_req.transactions.push(txn.clone());

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
            .broadcast_transactions(listener_peer_id, submit_txns_req, Duration::from_secs(5))
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
            Event::RpcRequest((peer_id, msg, callback)) => {
                assert_eq!(peer_id, dialer_peer_id);
                let dialer_peer_id_bytes = Vec::from(&dialer_peer_id);
                let msg = assert_matches::assert_matches!(
                    msg.message,
                    Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(msg)) => msg
                );
                assert_eq!(msg.peer_id, dialer_peer_id_bytes);
                let transactions: Vec<SignedTransaction> = msg.transactions;
                assert_eq!(transactions, vec![txn]);

                // send response back to callback
                let mut resp = BroadcastTransactionsResponse::default();
                resp.backpressure_ms = 0;
                let response_msg = MempoolSyncMsg {
                    message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(resp)),
                };
                let response_data = response_msg
                    .to_bytes()
                    .expect("[test_mempool_sync] failed to serialize proto");
                callback.send(Ok(response_data)).unwrap();
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    runtime.block_on(join(f_dialer, f_listener));
}

// Test that a permissioned end-point can connect to a permission-less end-point if both are
// correctly configured.
#[test]
fn test_permissionless_mempool_sync() {
    ::libra_logger::try_init_for_testing();
    let mut runtime = Runtime::new().unwrap();
    let mempool_sync_protocol = ProtocolId::from_static(MEMPOOL_RPC_PROTOCOL);

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
    let (listener_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    )
    .signing_keys((listener_signing_private_key, listener_signing_public_key))
    .permissioned(false)
    .transport(TransportType::PermissionlessMemoryNoise(Some((
        listener_identity_private_key,
        listener_identity_public_key,
    ))))
    .channel_size(8)
    .rpc_protocols(vec![mempool_sync_protocol.clone()])
    .build();
    let (_, mut listener_mp_net_events) =
        network_provider.add_mempool(vec![mempool_sync_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    )
    .transport(TransportType::MemoryNoise(Some((
        dialer_identity_private_key,
        dialer_identity_public_key,
    ))))
    .trusted_peers(trusted_peers.clone())
    .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
    .seed_peers(
        [(listener_peer_id, vec![listener_addr])]
            .iter()
            .cloned()
            .collect(),
    )
    .channel_size(8)
    .rpc_protocols(vec![mempool_sync_protocol.clone()])
    .build();
    let (mut dialer_mp_net_sender, mut dialer_mp_net_events) =
        network_provider.add_mempool(vec![mempool_sync_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

    // The dialer dials the listener and sends a mempool sync message
    let mut submit_txns_req = BroadcastTransactionsRequest::default();
    submit_txns_req.peer_id = dialer_peer_id.into();
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    let txn: SignedTransaction = get_test_signed_txn(sender, 0, keypair.0, keypair.1, None)
        .try_into()
        .unwrap();
    submit_txns_req.transactions.push(txn.clone());

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
            .broadcast_transactions(listener_peer_id, submit_txns_req, Duration::from_secs(5))
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
            Event::RpcRequest((peer_id, msg, callback)) => {
                assert_eq!(peer_id, dialer_peer_id);
                let dialer_peer_id_bytes = Vec::from(&dialer_peer_id);
                let msg = assert_matches::assert_matches!(
                    msg.message,
                    Some(MempoolSyncMsg_oneof::BroadcastTransactionsRequest(msg)) => msg
                );
                assert_eq!(msg.peer_id, dialer_peer_id_bytes);
                let transactions: Vec<SignedTransaction> = msg.transactions;
                assert_eq!(transactions, vec![txn]);

                // send response back to callback
                let mut resp = BroadcastTransactionsResponse::default();
                resp.backpressure_ms = 0;
                let response_msg = MempoolSyncMsg {
                    message: Some(MempoolSyncMsg_oneof::BroadcastTransactionsResponse(resp)),
                };
                let response_data = response_msg
                    .to_bytes()
                    .expect("[test_mempool_sync] failed to serialize proto");
                callback.send(Ok(response_data)).unwrap();
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
    let rpc_protocol = ProtocolId::from_static(CONSENSUS_RPC_PROTOCOL);

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
    let (listener_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        listener_peer_id,
        listener_addr,
        RoleType::Validator,
    )
    .signing_keys((listener_signing_private_key, listener_signing_public_key))
    .trusted_peers(trusted_peers.clone())
    .transport(TransportType::Memory)
    .channel_size(8)
    .rpc_protocols(vec![rpc_protocol.clone()])
    .build();
    let (_, mut listener_con_net_events) =
        network_provider.add_consensus(vec![rpc_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();
    let (_dialer_addr, mut network_provider) = NetworkBuilder::new(
        runtime.handle().clone(),
        dialer_peer_id,
        dialer_addr,
        RoleType::Validator,
    )
    .transport(TransportType::Memory)
    .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
    .trusted_peers(trusted_peers.clone())
    .seed_peers(
        [(listener_peer_id, vec![listener_addr])]
            .iter()
            .cloned()
            .collect(),
    )
    .channel_size(8)
    .rpc_protocols(vec![rpc_protocol.clone()])
    .build();
    let (mut dialer_con_net_sender, mut dialer_con_net_events) =
        network_provider.add_consensus(vec![rpc_protocol.clone()]);
    runtime.handle().spawn(network_provider.start());

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
    let req_block_msg_clone = req_block_msg.clone();
    let res_block_msg_clone = res_block_msg.clone();
    let f_listener = async move {
        // The listener receives a NewPeer event first
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
                    Some(ConsensusMsg_oneof::RequestBlock(req_block_msg_clone))
                );

                // Send the serialized response back.
                let res_msg = ConsensusMsg {
                    message: Some(ConsensusMsg_oneof::RespondBlock(res_block_msg_clone)),
                };
                let res_data = res_msg.to_bytes().unwrap();
                res_tx.send(Ok(res_data)).unwrap();
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    runtime.block_on(join(f_dialer, f_listener));
}
