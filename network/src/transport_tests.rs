// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    protocols::wire::handshake::v1::{MessagingProtocolVersion, ProtocolId, SupportedProtocols},
    transport::*,
};
use bytes::{Bytes, BytesMut};
use futures::{future, io::AsyncWriteExt, stream::StreamExt};
use libra_config::{config::HANDSHAKE_VERSION, network_id::NetworkContext};
use libra_crypto::{test_utils::TEST_SEED, traits::Uniform, x25519};
use libra_network_address::{NetworkAddress, Protocol::*};
use libra_types::{chain_id::ChainId, PeerId};
use netcore::{
    framing::{read_u16frame, write_u16frame},
    transport::{memory, ConnectionOrigin, Transport},
};
use rand::{rngs::StdRng, SeedableRng};
use std::{
    collections::{HashMap, HashSet},
    io,
    sync::{Arc, RwLock},
};
use tokio::runtime::Runtime;

/// helper to build trusted peer map
fn build_trusted_peers(
    id1: PeerId,
    key1: &x25519::PrivateKey,
    id2: PeerId,
    key2: &x25519::PrivateKey,
) -> Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>> {
    let pubkey_set1 = [key1.public_key()].iter().copied().collect();
    let pubkey_set2 = [key2.public_key()].iter().copied().collect();
    Arc::new(RwLock::new(
        vec![(id1, pubkey_set1), (id2, pubkey_set2)]
            .into_iter()
            .collect(),
    ))
}

enum Auth {
    Mutual,
    ServerOnly,
}

fn setup<TTransport>(
    base_transport: TTransport,
    auth: Auth,
) -> (
    Runtime,
    (PeerId, LibraNetTransport<TTransport>),
    (PeerId, LibraNetTransport<TTransport>),
    Option<Arc<RwLock<HashMap<PeerId, HashSet<x25519::PublicKey>>>>>,
    SupportedProtocols,
)
where
    TTransport: Transport<Error = io::Error> + Clone,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    let rt = Runtime::new().unwrap();

    let mut rng = StdRng::from_seed(TEST_SEED);
    let listener_key = x25519::PrivateKey::generate(&mut rng);
    let dialer_key = x25519::PrivateKey::generate(&mut rng);

    let (listener_peer_id, dialer_peer_id, trusted_peers) = match auth {
        Auth::Mutual => {
            let listener_peer_id = PeerId::random();
            let dialer_peer_id = PeerId::random();

            let trusted_peers =
                build_trusted_peers(dialer_peer_id, &dialer_key, listener_peer_id, &listener_key);

            (listener_peer_id, dialer_peer_id, Some(trusted_peers))
        }
        Auth::ServerOnly => {
            let listener_peer_id = PeerId::from_identity_public_key(listener_key.public_key());
            let dialer_peer_id = PeerId::from_identity_public_key(dialer_key.public_key());

            (listener_peer_id, dialer_peer_id, None)
        }
    };

    let supported_protocols = SupportedProtocols::from(
        [ProtocolId::ConsensusRpc, ProtocolId::DiscoveryDirectSend].iter(),
    );
    let chain_id = ChainId::default();
    let listener_transport = LibraNetTransport::new(
        base_transport.clone(),
        NetworkContext::mock_with_peer_id(listener_peer_id),
        listener_key,
        trusted_peers.clone(),
        HANDSHAKE_VERSION,
        chain_id,
        supported_protocols.clone(),
    );

    let dialer_transport = LibraNetTransport::new(
        base_transport,
        NetworkContext::mock_with_peer_id(dialer_peer_id),
        dialer_key,
        trusted_peers.clone(),
        HANDSHAKE_VERSION,
        chain_id,
        supported_protocols.clone(),
    );

    (
        rt,
        (listener_peer_id, listener_transport),
        (dialer_peer_id, dialer_transport),
        trusted_peers,
        supported_protocols,
    )
}

async fn write_read_msg(socket: &mut impl TSocket, msg: &[u8]) -> Bytes {
    write_u16frame(socket, msg).await.unwrap();
    socket.flush().await.unwrap();

    let mut buf = BytesMut::new();
    read_u16frame(socket, &mut buf).await.unwrap();
    buf.freeze()
}

/// Check that the network address matches the format
/// `"/memory/<port>/ln-noise-ik/<pubkey>/ln-handshake/<version>"`
fn expect_memory_noise_addr(addr: &NetworkAddress) {
    assert!(
        matches!(addr.as_slice(), [Memory(_), NoiseIK(_), Handshake(_)]),
        "addr: '{}'",
        addr
    );
}

/// Check that the network address matches the format
/// `"/ip4/<ipaddr>/tcp/<port>/ln-noise-ik/<pubkey>/ln-handshake/<version>"`
fn expect_ip4_tcp_noise_addr(addr: &NetworkAddress) {
    assert!(
        matches!(addr.as_slice(), [Ip4(_), Tcp(_), NoiseIK(_), Handshake(_)]),
        "addr: '{}'",
        addr
    );
}

fn test_transport_success<TTransport>(
    base_transport: TTransport,
    auth: Auth,
    listen_addr: &str,
    expect_formatted_addr: fn(&NetworkAddress),
) where
    TTransport: Transport<Error = io::Error> + Clone,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    let (
        mut rt,
        (listener_peer_id, listener_transport),
        (dialer_peer_id, dialer_transport),
        _trusted_peers,
        supported_protocols,
    ) = setup(base_transport, auth);

    let (mut inbounds, listener_addr) = rt.enter(|| {
        listener_transport
            .listen_on(listen_addr.parse().unwrap())
            .unwrap()
    });
    expect_formatted_addr(&listener_addr);
    let supported_protocols_clone = supported_protocols.clone();

    // we accept the dialer's inbound connection, check the connection metadata,
    // and verify that the upgraded socket actually works (sends and receives
    // bytes).
    let listener_task = async move {
        // accept one inbound connection from dialer
        let (inbound, _dialer_addr) = inbounds.next().await.unwrap().unwrap();
        let mut conn = inbound.await.unwrap();

        // check connection metadata
        assert_eq!(conn.metadata.peer_id, dialer_peer_id);
        expect_formatted_addr(&conn.metadata.addr);
        assert_eq!(conn.metadata.origin, ConnectionOrigin::Inbound);
        assert_eq!(
            conn.metadata.messaging_protocol,
            MessagingProtocolVersion::V1
        );
        assert_eq!(
            conn.metadata.application_protocols,
            supported_protocols_clone,
        );

        // test the socket works
        let msg = write_read_msg(&mut conn.socket, b"foobar").await;
        assert_eq!(&msg, b"barbaz".as_ref());
        conn.socket.close().await.unwrap();
    };

    // dial the listener, check the connection metadata, and verify that the
    // upgraded socket actually works (sends and receives bytes).
    let dialer_task = async move {
        // dial listener
        let mut conn = dialer_transport
            .dial(listener_peer_id, listener_addr.clone())
            .unwrap()
            .await
            .unwrap();

        // check connection metadata
        assert_eq!(conn.metadata.peer_id, listener_peer_id);
        assert_eq!(conn.metadata.addr, listener_addr);
        assert_eq!(conn.metadata.origin, ConnectionOrigin::Outbound);
        assert_eq!(
            conn.metadata.messaging_protocol,
            MessagingProtocolVersion::V1
        );
        assert_eq!(conn.metadata.application_protocols, supported_protocols);

        // test the socket works
        let msg = write_read_msg(&mut conn.socket, b"barbaz").await;
        assert_eq!(&msg, b"foobar".as_ref());
        conn.socket.close().await.unwrap();
    };

    rt.block_on(future::join(listener_task, dialer_task));
}

fn test_transport_rejects_unauthed_dialer<TTransport>(
    base_transport: TTransport,
    listen_addr: &str,
    expect_formatted_addr: fn(&NetworkAddress),
) where
    TTransport: Transport<Error = io::Error> + Clone,
    TTransport::Output: TSocket,
    TTransport::Outbound: Send + 'static,
    TTransport::Inbound: Send + 'static,
    TTransport::Listener: Send + 'static,
{
    let (
        mut rt,
        (listener_peer_id, listener_transport),
        (dialer_peer_id, dialer_transport),
        trusted_peers,
        _supported_protocols,
    ) = setup(base_transport, Auth::Mutual);

    // remove dialer from trusted_peers set
    trusted_peers
        .as_ref()
        .unwrap()
        .write()
        .unwrap()
        .remove(&dialer_peer_id)
        .unwrap();

    let (mut inbounds, listener_addr) = rt.enter(|| {
        listener_transport
            .listen_on(listen_addr.parse().unwrap())
            .unwrap()
    });
    expect_formatted_addr(&listener_addr);

    // we try to accept one inbound connection from the dialer. however, the
    // connection upgrade should fail because the dialer is not authenticated
    // (not in the trusted peers set).
    let listener_task = async move {
        let (inbound, _dialer_addr) = inbounds.next().await.unwrap().unwrap();
        inbound
            .await
            .expect_err("should fail because the dialer is not a trusted peer");
    };

    // we attempt to dial the listener. however, the connection upgrade should
    // fail because we are not authenticated.
    let dialer_task = async move {
        // dial listener
        let fut_upgrade = dialer_transport
            .dial(listener_peer_id, listener_addr.clone())
            .unwrap();
        fut_upgrade
            .await
            .expect_err("should fail because listener rejects our unauthed connection");
    };

    rt.block_on(future::join(listener_task, dialer_task));
}

////////////////////////////////////////
// LibraNetTransport<MemoryTransport> //
////////////////////////////////////////

#[test]
fn test_memory_transport_mutual_auth() {
    test_transport_success(
        memory::MemoryTransport,
        Auth::Mutual,
        "/memory/0",
        expect_memory_noise_addr,
    );
}

#[test]
fn test_memory_transport_server_only_auth() {
    test_transport_success(
        memory::MemoryTransport,
        Auth::ServerOnly,
        "/memory/0",
        expect_memory_noise_addr,
    );
}

#[test]
fn test_memory_transport_rejects_unauthed_dialer() {
    test_transport_rejects_unauthed_dialer(
        memory::MemoryTransport,
        "/memory/0",
        expect_memory_noise_addr,
    );
}

/////////////////////////////////////
// LibraNetTransport<TcpTransport> //
/////////////////////////////////////

#[test]
fn test_tcp_transport_mutual_auth() {
    test_transport_success(
        LIBRA_TCP_TRANSPORT.clone(),
        Auth::Mutual,
        "/ip4/127.0.0.1/tcp/0",
        expect_ip4_tcp_noise_addr,
    );
}

#[test]
fn test_tcp_transport_server_only_auth() {
    test_transport_success(
        LIBRA_TCP_TRANSPORT.clone(),
        Auth::ServerOnly,
        "/ip4/127.0.0.1/tcp/0",
        expect_ip4_tcp_noise_addr,
    );
}

#[test]
fn test_tcp_transport_rejects_unauthed_dialer() {
    test_transport_rejects_unauthed_dialer(
        LIBRA_TCP_TRANSPORT.clone(),
        "/ip4/127.0.0.1/tcp/0",
        expect_ip4_tcp_noise_addr,
    );
}
