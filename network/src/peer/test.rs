// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

// NB: The following tests can be added, but are left out since we will remove stream handing from
// Peer in the next phase of migration to new wire protocol.
// TODO: Test handling of inbound msg if stream fails negotiation.
// TODO: Test handling of inbound msg if stream negotiated but then closed.
// TODO: Test handling of outbound msg if stream fails negotiation.
// TODO: Test handling of outbound msg if stream negotiated but then closed.
// TODO: Test handling of inbound msg without unknown protocol_id.

use crate::{
    peer::{DisconnectReason, Peer, PeerHandle, PeerNotification},
    protocols::{
        identity::Identity,
        wire::messaging::v1::{DirectSendMsg, NetworkMessage},
    },
    ProtocolId,
};
use bytes::Bytes;
use futures::{future::join, stream::StreamExt, SinkExt};
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    compat::IoCompat,
    multiplexing::{yamux::Yamux, Control, StreamMultiplexer},
    negotiate::{negotiate_inbound, negotiate_outbound_select},
    transport::ConnectionOrigin,
};
use parity_multiaddr::Multiaddr;
use std::{
    collections::HashSet, iter::FromIterator, mem::ManuallyDrop, str::FromStr, time::Duration,
};
use tokio::{runtime::Runtime, time::timeout};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

static PROTOCOL: ProtocolId = ProtocolId::MempoolDirectSend;

fn build_test_identity(peer_id: PeerId) -> Identity {
    Identity::new(peer_id, Vec::new())
}

fn build_test_peer(
    origin: ConnectionOrigin,
) -> (
    Peer<MemorySocket>,
    PeerHandle,
    MemorySocket,
    channel::Receiver<PeerNotification>,
    channel::Receiver<PeerNotification>,
    channel::Receiver<PeerNotification>,
) {
    let (a, b) = MemorySocket::new_pair();
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
        HashSet::new(), /* rpc protocols */
        peer_rpc_notifs_tx,
        HashSet::from_iter(vec![PROTOCOL].into_iter()), /* direct_send protocols */
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
        Peer<MemorySocket>,
        PeerHandle,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
    ),
    (
        Peer<MemorySocket>,
        PeerHandle,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
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

async fn assert_new_message_event(peer_notifs_rx: &mut channel::Receiver<PeerNotification>) {
    let event = peer_notifs_rx.next().await;
    assert!(
        matches!(event, Some(PeerNotification::NewMessage(_))),
        "Expected NewMessage event. Received: {:?}",
        event
    );
}

async fn assert_peer_disconnected_event(
    peer_id: PeerId,
    reason: DisconnectReason,
    peer_notifs_rx: &mut channel::Receiver<PeerNotification>,
) {
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

async fn create_yamux_socket(
    connection: MemorySocket,
    origin: ConnectionOrigin,
) -> (
    <Yamux<MemorySocket> as StreamMultiplexer>::Listener,
    <Yamux<MemorySocket> as StreamMultiplexer>::Control,
) {
    Yamux::upgrade_connection(connection, origin)
        .await
        .expect("Failed to wrap conncection with yamux")
        .start()
        .await
}

#[test]
fn peer_send_message() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (
        peer,
        mut peer_handle,
        connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let send_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Bytes::from_static(b"hello world"),
    });
    let recv_msg = send_msg.clone();

    let server = async move {
        let (mut substream_listener, _) =
            create_yamux_socket(connection, ConnectionOrigin::Outbound).await;
        let substream = substream_listener.next().await;
        // Drive the connection.
        let driver = async move { while let Some(_) = substream_listener.next().await {} };
        tokio::spawn(driver);
        // First, the client should negotiate protocol.
        let (substream, _protocol) = negotiate_inbound(
            substream.unwrap().unwrap(),
            &[lcs::to_bytes(&PROTOCOL).unwrap()],
        )
        .await
        .unwrap();
        // The client should then send the network message.
        let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
        let msg = substream.next().await.unwrap();
        let msg: NetworkMessage = lcs::from_bytes(&msg.unwrap().freeze()).unwrap();
        assert_eq!(msg, recv_msg);
        substream.close().await.unwrap();
    };

    let client = async move {
        peer_handle.send_message(send_msg, PROTOCOL).await.unwrap();
        ManuallyDrop::new(peer_handle);
    };
    rt.spawn(peer.start());
    rt.block_on(join(server, client));
}

#[tokio::test]
async fn peer_recv_message() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let (
        peer,
        _peer_handle,
        connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        mut peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let send_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Bytes::from_static(b"hello world"),
    });
    let recv_msg = send_msg.clone();

    let server = async move {
        let (mut substream_listener, mut control) =
            create_yamux_socket(connection, ConnectionOrigin::Outbound).await;
        // Drive the connection.
        let driver = async move { while let Some(_) = substream_listener.next().await {} };
        tokio::spawn(driver);
        let substream = control.open_stream().await.unwrap();
        // First, the client should negotiate protocol.
        let substream = negotiate_outbound_select(substream, lcs::to_bytes(&PROTOCOL).unwrap())
            .await
            .unwrap();
        // The client should then send the network message.
        let mut substream = Framed::new(IoCompat::new(substream), LengthDelimitedCodec::new());
        substream
            .send(lcs::to_bytes(&send_msg).unwrap().into())
            .await
            .unwrap();
        // Client then closes substream.
        substream.close().await.unwrap();
    };

    let client = async move {
        // Wait to receive notification of DirectSendMsg from Peer.
        let received = peer_direct_send_notifs_rx.next().await.unwrap();
        assert!(
            matches!(received, PeerNotification::NewMessage(received_msg) if received_msg == recv_msg)
        );
    };
    tokio::spawn(peer.start());
    join(server, client).await;
}

// Test that if two peers request to open a substream with each other simultaneously that
// we won't deadlock.
#[tokio::test]
async fn peer_open_substream_simultaneous() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let (
        (
            peer_a,
            mut peer_handle_a,
            mut peer_notifs_rx_a,
            _peer_rpc_notifs_rx_a,
            mut peer_direct_send_notifs_rx_a,
        ),
        (
            peer_b,
            mut peer_handle_b,
            mut peer_notifs_rx_b,
            _peer_rpc_notifs_rx_b,
            mut peer_direct_send_notifs_rx_b,
        ),
    ) = build_test_connected_peers();

    let test = async move {
        let msg_a = NetworkMessage::DirectSendMsg(DirectSendMsg {
            protocol_id: PROTOCOL,
            priority: 0,
            raw_msg: Bytes::from_static(b"hello world"),
        });
        let msg_b = NetworkMessage::DirectSendMsg(DirectSendMsg {
            protocol_id: PROTOCOL,
            priority: 0,
            raw_msg: Bytes::from_static(b"namaste"),
        });

        // Send open substream requests to both peer_a and peer_b
        let send_msg_a = peer_handle_a.send_message(msg_a, PROTOCOL);
        let send_msg_b = peer_handle_b.send_message(msg_b, PROTOCOL);
        // These both should complete, but in the event they deadlock wrap them in a timeout
        let timeout_a = timeout(Duration::from_secs(10), send_msg_a);
        let timeout_b = timeout(Duration::from_secs(10), send_msg_b);
        timeout_a.await.unwrap().unwrap();
        timeout_b.await.unwrap().unwrap();

        // Check that we received the new inbound substream for both peers
        assert_new_message_event(&mut peer_direct_send_notifs_rx_a).await;
        assert_new_message_event(&mut peer_direct_send_notifs_rx_b).await;

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
    ::libra_logger::Logger::new().environment_only(true).init();
    let (
        peer,
        mut peer_handle,
        connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        // Create yamux socket -> this ensures that Peer actor also upgrades its connection to
        // yamux before accepting any requests.
        let (_, _control) = create_yamux_socket(connection, ConnectionOrigin::Outbound).await;
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
    ::libra_logger::Logger::new().environment_only(true).init();
    let (
        peer,
        peer_handle,
        connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        // Create yamux socket -> this ensures that Peer actor also upgrades its connection to
        // yamux.
        let (_, mut control) = create_yamux_socket(connection, ConnectionOrigin::Outbound).await;
        control.close().await.unwrap();
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx,
        )
        .await;
    };
    join(test, peer.start()).await;
}

#[test]
fn peer_terminates_when_request_tx_has_dropped() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (
        peer,
        peer_handle,
        connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(ConnectionOrigin::Inbound);
    // Perform yamux handshake in a different task.
    let drop = async move {
        let (_, control) = create_yamux_socket(connection, ConnectionOrigin::Outbound).await;
        // Do not drop control.
        ManuallyDrop::new(control);
        // Drop peer handle.
        drop(peer_handle);
    };
    rt.block_on(join(peer.start(), drop));
}
