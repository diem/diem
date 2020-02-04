// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{DisconnectReason, Peer, PeerHandle, PeerNotification},
    protocols::identity::Identity,
    ProtocolId,
};
use channel;
use futures::{
    channel::oneshot,
    executor::block_on,
    future::join,
    io::{AsyncReadExt, AsyncWriteExt},
    stream::StreamExt,
};
use libra_config::config::RoleType;
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    multiplexing::{
        yamux::{Mode, StreamHandle, Yamux},
        StreamMultiplexer,
    },
    negotiate::negotiate_inbound,
    transport::ConnectionOrigin,
};
use parity_multiaddr::Multiaddr;
use std::fmt::Debug;
use std::time::Duration;
use tokio::time::timeout;

const HELLO_PROTOCOL: &[u8] = b"/hello-world/1.0.0";

fn build_test_connection() -> (Yamux<MemorySocket>, Yamux<MemorySocket>) {
    let (dialer, listener) = MemorySocket::new_pair();

    (
        Yamux::new(dialer, Mode::Client),
        Yamux::new(listener, Mode::Server),
    )
}

fn build_test_identity(peer_id: PeerId) -> Identity {
    Identity::new(peer_id, Vec::new(), RoleType::Validator)
}

fn build_test_peer(
    origin: ConnectionOrigin,
) -> (
    Peer<Yamux<MemorySocket>>,
    PeerHandle<StreamHandle<MemorySocket>>,
    Yamux<MemorySocket>,
    channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
) {
    let (a, b) = build_test_connection();
    let identity = build_test_identity(PeerId::random());
    let peer_id = identity.peer_id();
    let (peer_notifs_tx, peer_notifs_rx) = channel::new_test(1);
    let (peer_req_tx, peer_req_rx) = channel::new_test(0);

    let peer = Peer::new(
        identity,
        a,
        origin,
        vec![ProtocolId::from_static(HELLO_PROTOCOL)],
        peer_notifs_tx,
        peer_req_rx,
    );
    let peer_handle = PeerHandle::new(peer_id, Multiaddr::empty(), origin, peer_req_tx);

    (peer, peer_handle, b, peer_notifs_rx)
}

fn build_test_connected_peers() -> (
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle<MemorySocket>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    ),
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle<MemorySocket>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    ),
) {
    let (peer_a, peer_handle_a, connection_a, peer_notifs_rx_a) =
        build_test_peer(ConnectionOrigin::Inbound);
    let (mut peer_b, peer_handle_b, _connection_b, peer_notifs_rx_b) =
        build_test_peer(ConnectionOrigin::Outbound);
    // Make sure both peers are connected
    peer_b.connection = connection_a;

    (
        (peer_a, peer_handle_a, peer_notifs_rx_a),
        (peer_b, peer_handle_b, peer_notifs_rx_b),
    )
}

async fn assert_new_substream_event<TSubstream>(
    peer_id: PeerId,
    peer_notifs_rx: &mut channel::Receiver<PeerNotification<TSubstream>>,
) where
    TSubstream: Debug,
{
    match peer_notifs_rx.next().await {
        Some(PeerNotification::NewSubstream(actual_peer_id, _)) => {
            assert_eq!(actual_peer_id, peer_id);
        }
        event => {
            panic!("Expected a NewSubstream, received: {:?}", event);
        }
    }
}

async fn assert_peer_disconnected_event<TSubstream>(
    peer_id: PeerId,
    role: RoleType,
    reason: DisconnectReason,
    peer_notifs_rx: &mut channel::Receiver<PeerNotification<TSubstream>>,
) where
    TSubstream: Debug,
{
    match peer_notifs_rx.next().await {
        Some(PeerNotification::PeerDisconnected(
            actual_peer_id,
            actual_role,
            _origin,
            actual_reason,
        )) => {
            assert_eq!(actual_peer_id, peer_id);
            assert_eq!(actual_role, role);
            assert_eq!(actual_reason, reason);
        }
        event => {
            panic!(
                "Expected a Requested PeerDisconnected, received: {:?}",
                event
            );
        }
    }
}

#[test]
fn peer_open_substream() {
    let (peer, _peer_handle, connection, _peer_notifs_rx) =
        build_test_peer(ConnectionOrigin::Inbound);

    let server = async move {
        let substream_listener = connection.listen_for_inbound();
        let (substream, _substream_listener) = substream_listener.into_future().await;
        let (mut substream, _protocol) =
            negotiate_inbound(substream.unwrap().unwrap(), [HELLO_PROTOCOL])
                .await
                .unwrap();
        substream.write_all(b"hello world").await.unwrap();
        substream.flush().await.unwrap();
        substream.close().await.unwrap();
        // Wait to read EOF from the other side in order to hold open the connection for
        // the remote side to read
        let mut buf = Vec::new();
        substream.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf.len(), 0);
    };

    let client = async move {
        let (substream_tx, substream_rx) = oneshot::channel();
        peer.handle_open_outbound_substream_request(
            ProtocolId::from_static(HELLO_PROTOCOL),
            substream_tx,
        )
        .await;
        let mut substream = substream_rx.await.unwrap().unwrap();
        let mut buf = Vec::new();
        substream.read_to_end(&mut buf).await.unwrap();
        substream.close().await.unwrap();
        assert_eq!(buf, b"hello world");
    };

    block_on(join(server, client));
}

// Test that if two peers request to open a substream with each other simultaneously that
// we won't deadlock.
#[test]
fn peer_open_substream_simultaneous() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();
    let (
        (peer_a, mut peer_handle_a, mut peer_notifs_rx_a),
        (peer_b, mut peer_handle_b, mut peer_notifs_rx_b),
    ) = build_test_connected_peers();

    let test = async move {
        let (substream_tx_a, substream_rx_a) = oneshot::channel();
        let (substream_tx_b, substream_rx_b) = oneshot::channel();

        // Send open substream requests to both peer_a and peer_b
        peer_handle_a
            .open_substream(ProtocolId::from_static(HELLO_PROTOCOL), substream_tx_a)
            .await;
        peer_handle_b
            .open_substream(ProtocolId::from_static(HELLO_PROTOCOL), substream_tx_b)
            .await;

        // These both should complete, but in the event they deadlock wrap them in a timeout
        let timeout_a = timeout(Duration::from_secs(10), substream_rx_a);
        let timeout_b = timeout(Duration::from_secs(10), substream_rx_b);
        let _ = timeout_a.await.unwrap().unwrap();
        let _ = timeout_b.await.unwrap().unwrap();

        // Check that we received the new inbound substream for both peers
        assert_new_substream_event(peer_handle_a.peer_id, &mut peer_notifs_rx_a).await;
        assert_new_substream_event(peer_handle_b.peer_id, &mut peer_notifs_rx_b).await;

        // Shut one peers and the other should shutdown due to ConnectionLost
        peer_handle_a.disconnect().await;

        // Check that we received both shutdown events
        assert_peer_disconnected_event(
            peer_handle_a.peer_id,
            RoleType::Validator,
            DisconnectReason::Requested,
            &mut peer_notifs_rx_a,
        )
        .await;
        assert_peer_disconnected_event(
            peer_handle_b.peer_id,
            RoleType::Validator,
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx_b,
        )
        .await;
    };

    runtime.spawn(peer_a.start());
    runtime.spawn(peer_b.start());

    runtime.block_on(test);
}

#[tokio::test]
async fn peer_disconnect_request() {
    let (peer, mut peer_handle, _connection, mut internal_event_rx) =
        build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        peer_handle.disconnect().await;
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            RoleType::Validator,
            DisconnectReason::Requested,
            &mut internal_event_rx,
        )
        .await;
    };

    join(test, peer.start()).await;
}

#[tokio::test]
async fn peer_disconnect_connection_lost() {
    let (peer, peer_handle, connection, mut internal_event_rx) =
        build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        connection.close().await.unwrap();
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            RoleType::Validator,
            DisconnectReason::ConnectionLost,
            &mut internal_event_rx,
        )
        .await;
    };

    join(test, peer.start()).await;
}

#[test]
#[should_panic]
fn peer_panics_when_request_tx_has_dropped() {
    let (peer, peer_handle, _conn, _event_rx) = build_test_peer(ConnectionOrigin::Inbound);

    drop(peer_handle);
    block_on(peer.start());
}
