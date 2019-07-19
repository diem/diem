// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for validator_network.
#![cfg(test)]
use crate::{
    common::NetworkPublicKeys,
    proto::{ConsensusMsg, MempoolSyncMsg, RequestBlock, RespondBlock, SignedTransaction},
    validator_network::{
        network_builder::{NetworkBuilder, TransportType},
        Event, CONSENSUS_RPC_PROTOCOL, MEMPOOL_DIRECT_SEND_PROTOCOL,
    },
    ProtocolId,
};
use crypto::x25519;
use futures::{executor::block_on, future::join, StreamExt};
use nextgen_crypto::{ed25519::compat, test_utils::TEST_SEED};
use parity_multiaddr::Multiaddr;
use protobuf::Message as proto_msg;
use rand::{rngs::StdRng, SeedableRng};
use std::{collections::HashMap, time::Duration};
use tokio::runtime::Runtime;
use types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    PeerId,
};

#[test]
fn test_network_builder() {
    let runtime = Runtime::new().unwrap();
    let peer_id = PeerId::random();
    let addr: Multiaddr = "/memory/0".parse().unwrap();
    let mempool_sync_protocol = ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL);
    let consensus_get_blocks_protocol = ProtocolId::from_static(b"get_blocks");
    let synchronizer_get_chunks_protocol = ProtocolId::from_static(b"get_chunks");
    let (signing_private_key, signing_public_key) = compat::generate_keypair(None);
    let (identity_private_key, identity_public_key) = x25519::generate_keypair();

    let (
        (_mempool_network_sender, _mempool_network_events),
        (_consensus_network_sender, _consensus_network_events),
        _listen_addr,
    ) = NetworkBuilder::new(runtime.executor(), peer_id, addr)
        .transport(TransportType::Memory)
        .signing_keys((signing_private_key, signing_public_key.clone()))
        .identity_keys((identity_private_key, identity_public_key))
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
        .consensus_protocols(vec![consensus_get_blocks_protocol.clone()])
        .mempool_protocols(vec![mempool_sync_protocol.clone()])
        .direct_send_protocols(vec![mempool_sync_protocol])
        .rpc_protocols(vec![
            consensus_get_blocks_protocol,
            synchronizer_get_chunks_protocol,
        ])
        .build();
}

#[test]
fn test_mempool_sync() {
    ::logger::try_init_for_testing();
    let runtime = Runtime::new().unwrap();
    let mempool_sync_protocol = ProtocolId::from_static(MEMPOOL_DIRECT_SEND_PROTOCOL);

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
    let (listener_identity_private_key, listener_identity_public_key) = x25519::generate_keypair();
    let (dialer_identity_private_key, dialer_identity_public_key) = x25519::generate_keypair();

    // Set up the listener network
    let listener_addr: Multiaddr = "/memory/0".parse().unwrap();

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

    let ((_, mut listener_mp_net_events), _, listener_addr) =
        NetworkBuilder::new(runtime.executor(), listener_peer_id, listener_addr)
            .signing_keys((listener_signing_private_key, listener_signing_public_key))
            .identity_keys((listener_identity_private_key, listener_identity_public_key))
            .trusted_peers(trusted_peers.clone())
            .transport(TransportType::Memory)
            .channel_size(8)
            .mempool_protocols(vec![mempool_sync_protocol.clone()])
            .direct_send_protocols(vec![mempool_sync_protocol.clone()])
            .build();

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();

    let ((mut dialer_mp_net_sender, mut dialer_mp_net_events), _, _dialer_addr) =
        NetworkBuilder::new(runtime.executor(), dialer_peer_id, dialer_addr)
            .transport(TransportType::Memory)
            .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
            .identity_keys((dialer_identity_private_key, dialer_identity_public_key))
            .trusted_peers(trusted_peers.clone())
            .seed_peers(
                [(listener_peer_id, vec![listener_addr])]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .channel_size(8)
            .mempool_protocols(vec![mempool_sync_protocol.clone()])
            .direct_send_protocols(vec![mempool_sync_protocol.clone()])
            .build();

    // The dialer dials the listener and sends a mempool sync message
    let mut mempool_msg = MempoolSyncMsg::new();
    mempool_msg.set_peer_id(dialer_peer_id.into());
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    let keypair = compat::generate_keypair(&mut rng);
    let txn = get_test_signed_txn(sender, 0, keypair.0.into(), keypair.1.into(), None);
    mempool_msg.set_transactions(::protobuf::RepeatedField::from_vec(vec![txn.clone()]));

    let f_dialer = async move {
        // Wait until dialing finished and NewPeer event received
        match dialer_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, listener_peer_id.into());
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // Dialer sends a mempool sync message
        dialer_mp_net_sender
            .send_to(listener_peer_id.into(), mempool_msg)
            .await
            .unwrap();
    };

    // The listener receives a mempool sync message
    let f_listener = async move {
        // The listener receives a NewPeer event first
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, dialer_peer_id.into());
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // The listener then receives the mempool sync message
        match listener_mp_net_events.next().await.unwrap().unwrap() {
            Event::Message((peer_id, msg)) => {
                assert_eq!(peer_id, dialer_peer_id.into());
                let dialer_peer_id_bytes = Vec::from(&dialer_peer_id);
                assert_eq!(msg.peer_id, dialer_peer_id_bytes);
                let transactions: Vec<SignedTransaction> = msg.transactions.into();
                assert_eq!(transactions, vec![txn]);
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    block_on(join(f_dialer, f_listener));
}

#[test]
fn test_consensus_rpc() {
    ::logger::try_init_for_testing();
    let runtime = Runtime::new().unwrap();
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
    let (listener_identity_private_key, listener_identity_public_key) = x25519::generate_keypair();
    let (dialer_identity_private_key, dialer_identity_public_key) = x25519::generate_keypair();

    // Set up the listener network
    let listener_addr: Multiaddr = "/memory/0".parse().unwrap();

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

    let (_, (_, mut listener_con_net_events), listener_addr) =
        NetworkBuilder::new(runtime.executor(), listener_peer_id, listener_addr)
            .signing_keys((listener_signing_private_key, listener_signing_public_key))
            .identity_keys((listener_identity_private_key, listener_identity_public_key))
            .trusted_peers(trusted_peers.clone())
            .transport(TransportType::Memory)
            .channel_size(8)
            .consensus_protocols(vec![rpc_protocol.clone()])
            .rpc_protocols(vec![rpc_protocol.clone()])
            .build();

    // Set up the dialer network
    let dialer_addr: Multiaddr = "/memory/0".parse().unwrap();

    let (_, (mut dialer_con_net_sender, mut dialer_con_net_events), _dialer_addr) =
        NetworkBuilder::new(runtime.executor(), dialer_peer_id, dialer_addr)
            .transport(TransportType::Memory)
            .signing_keys((dialer_signing_private_key, dialer_signing_public_key))
            .identity_keys((dialer_identity_private_key, dialer_identity_public_key))
            .trusted_peers(trusted_peers.clone())
            .seed_peers(
                [(listener_peer_id, vec![listener_addr])]
                    .iter()
                    .cloned()
                    .collect(),
            )
            .channel_size(8)
            .consensus_protocols(vec![rpc_protocol.clone()])
            .rpc_protocols(vec![rpc_protocol.clone()])
            .build();

    let block_id = vec![0_u8; 32];
    let mut req_block_msg = RequestBlock::new();
    req_block_msg.set_block_id(block_id.into());

    let res_block_msg = RespondBlock::new();

    // The dialer dials the listener and sends a RequestBlock rpc request
    let req_block_msg_clone = req_block_msg.clone();
    let res_block_msg_clone = res_block_msg.clone();
    let f_dialer = async move {
        // Wait until dialing finished and NewPeer event received
        match dialer_con_net_events.next().await.unwrap().unwrap() {
            Event::NewPeer(peer_id) => {
                assert_eq!(peer_id, listener_peer_id.into());
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // Dialer sends a RequestBlock rpc request.
        let res_block_msg = dialer_con_net_sender
            .request_block(
                listener_peer_id.into(),
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
                assert_eq!(peer_id, dialer_peer_id.into());
            }
            event => panic!("Unexpected event {:?}", event),
        }

        // The listener then handles the RequestBlock rpc request.
        match listener_con_net_events.next().await.unwrap().unwrap() {
            Event::RpcRequest((peer_id, mut req_msg, res_tx)) => {
                assert_eq!(peer_id, dialer_peer_id.into());

                // Check the request
                assert!(req_msg.has_request_block());
                let req_block_msg = req_msg.take_request_block();
                assert_eq!(req_block_msg, req_block_msg_clone);

                // Send the serialized response back.
                let mut res_msg = ConsensusMsg::new();
                res_msg.set_respond_block(res_block_msg_clone);
                let res_data = res_msg.write_to_bytes().unwrap().into();
                res_tx.send(Ok(res_data)).unwrap();
            }
            event => panic!("Unexpected event {:?}", event),
        }
    };

    block_on(join(f_dialer, f_listener));
}
