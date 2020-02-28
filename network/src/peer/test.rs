// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{DisconnectReason, Peer, PeerHandle, PeerNotification},
    protocols::identity::Identity,
    ProtocolId,
};
use futures::{
    executor::block_on,
    future::join,
    io::{AsyncReadExt, AsyncWriteExt},
    stream::StreamExt,
};
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
use std::{collections::HashSet, fmt::Debug, iter::FromIterator, str::FromStr, time::Duration};
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
    Identity::new(peer_id, Vec::new())
}

fn build_test_peer(
    origin: ConnectionOrigin,
) -> (
    Peer<Yamux<MemorySocket>>,
    PeerHandle<StreamHandle>,
    Yamux<MemorySocket>,
    channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
) {
    let (a, b) = build_test_connection();
    let identity = build_test_identity(PeerId::random());
    let peer_id = identity.peer_id();
    let (peer_notifs_tx, peer_notifs_rx) = channel::new_test(1);
    let (peer_rpc_notifs_tx, peer_rpc_notifs_rx) = channel::new_test(1);
    let (peer_direct_send_notifs_tx, peer_direct_send_notifs_rx) = channel::new_test(1);
    let (peer_req_tx, peer_req_rx) = channel::new_test(0);

    let peer = Peer::new(
        identity,
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/8081").unwrap(),
        origin,
        a,
        peer_req_rx,
        peer_notifs_tx,
        HashSet::from_iter(vec![ProtocolId::from_static(HELLO_PROTOCOL)].into_iter()), /* rpc protocols */
        peer_rpc_notifs_tx,
        HashSet::new(), /* direct_send protocols */
        peer_direct_send_notifs_tx,
    );
    let peer_handle = PeerHandle::new(peer_id, Multiaddr::empty(), peer_req_tx);

    (
        peer,
        peer_handle,
        b,
        peer_notifs_rx,
        peer_rpc_notifs_rx,
        peer_direct_send_notifs_rx,
    )
}

fn build_test_connected_peers() -> (
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    ),
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
        channel::Receiver<PeerNotification<<Yamux<MemorySocket> as StreamMultiplexer>::Substream>>,
    ),
) {
    let (
        peer_a,
        peer_handle_a,
        connection_a,
        peer_notifs_rx_a,
        peer_rpc_notifs_rx_a,
        peer_direct_send_notifs_rx_a,
    ) = build_test_peer(ConnectionOrigin::Inbound);
    let (
        mut peer_b,
        peer_handle_b,
        _connection_b,
        peer_notifs_rx_b,
        peer_rpc_notifs_rx_b,
        peer_direct_send_notifs_rx_b,
    ) = build_test_peer(ConnectionOrigin::Outbound);
    // Make sure both peers are connected
    peer_b.connection = Some(connection_a);
    (
        (
            peer_a,
            peer_handle_a,
            peer_notifs_rx_a,
            peer_rpc_notifs_rx_a,
            peer_direct_send_notifs_rx_a,
        ),
        (
            peer_b,
            peer_handle_b,
            peer_notifs_rx_b,
            peer_rpc_notifs_rx_b,
            peer_direct_send_notifs_rx_b,
        ),
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
    reason: DisconnectReason,
    peer_notifs_rx: &mut channel::Receiver<PeerNotification<TSubstream>>,
) where
    TSubstream: Debug,
{
    match peer_notifs_rx.next().await {
        Some(PeerNotification::PeerDisconnected(
            actual_identity,
            _actual_addr,
            _actual_origin,
            actual_reason,
        )) => {
            assert_eq!(actual_identity.peer_id(), peer_id);
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

#[tokio::test]
async fn peer_open_substream() {
    ::libra_logger::try_init_for_testing();
    let (
        peer,
        mut peer_handle,
        connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let server = async move {
        let (mut substream_listener, _) = connection.start().await;
        let substream = substream_listener.next().await;
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
        let mut substream = peer_handle
            .open_substream(ProtocolId::from_static(HELLO_PROTOCOL))
            .await
            .unwrap();
        let mut buf = Vec::new();
        substream.read_to_end(&mut buf).await.unwrap();
        substream.close().await.unwrap();
        assert_eq!(buf, b"hello world");
    };
    tokio::spawn(peer.start());
    join(server, client).await;
}

// Test that if two peers request to open a substream with each other simultaneously that
// we won't deadlock.
#[tokio::test]
async fn peer_open_substream_simultaneous() {
    ::libra_logger::try_init_for_testing();
    let (
        (
            peer_a,
            mut peer_handle_a,
            mut peer_notifs_rx_a,
            mut peer_rpc_notifs_rx_a,
            _peer_direct_send_notifs_rx_a,
        ),
        (
            peer_b,
            mut peer_handle_b,
            mut peer_notifs_rx_b,
            mut peer_rpc_notifs_rx_b,
            _peer_direct_send_notifs_rx_b,
        ),
    ) = build_test_connected_peers();

    let test = async move {
        // Send open substream requests to both peer_a and peer_b
        let substream_rx_a = peer_handle_a.open_substream(ProtocolId::from_static(HELLO_PROTOCOL));
        let substream_rx_b = peer_handle_b.open_substream(ProtocolId::from_static(HELLO_PROTOCOL));
        // These both should complete, but in the event they deadlock wrap them in a timeout
        let timeout_a = timeout(Duration::from_secs(10), substream_rx_a);
        let timeout_b = timeout(Duration::from_secs(10), substream_rx_b);
        let _a = timeout_a.await.unwrap().unwrap();
        let _b = timeout_b.await.unwrap().unwrap();

        // Check that we received the new inbound substream for both peers
        assert_new_substream_event(peer_handle_a.peer_id, &mut peer_rpc_notifs_rx_a).await;
        assert_new_substream_event(peer_handle_b.peer_id, &mut peer_rpc_notifs_rx_b).await;

        // Shut one peers and the other should shutdown due to ConnectionLost
        peer_handle_a.disconnect().await;

        // Check that we received both shutdown events
        assert_peer_disconnected_event(
            peer_handle_a.peer_id,
            DisconnectReason::Requested,
            &mut peer_notifs_rx_a,
        )
        .await;
        assert_peer_disconnected_event(
            peer_handle_b.peer_id,
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx_b,
        )
        .await;
    };

    tokio::spawn(peer_a.start());
    tokio::spawn(peer_b.start());

    test.await;
}

#[tokio::test]
async fn peer_disconnect_request() {
    let (
        peer,
        mut peer_handle,
        _connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        peer_handle.disconnect().await;
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::Requested,
            &mut peer_notifs_rx,
        )
        .await;
    };

    join(test, peer.start()).await;
}

#[tokio::test]
async fn peer_disconnect_connection_lost() {
    libra_logger::try_init_for_testing();
    let (
        peer,
        peer_handle,
        connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        drop(connection);
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx,
        )
        .await;
    };
    join(test, peer.start()).await;
}

#[tokio::test]
async fn peer_terminates_when_request_tx_has_dropped() {
    let (
        peer,
        peer_handle,
        _connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);
    drop(peer_handle);
    block_on(peer.start());
}
