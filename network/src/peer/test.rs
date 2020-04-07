// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{DisconnectReason, Peer, PeerHandle, PeerNotification},
    peer_manager::{Connection, ConnectionId, ConnectionMetadata},
    protocols::{
        identity::Identity,
        wire::messaging::v1::{DirectSendMsg, NetworkMessage},
    },
    ProtocolId,
};
use bytes::Bytes;
use futures::{future::join, io::AsyncWriteExt, stream::StreamExt, SinkExt};
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{compat::IoCompat, transport::ConnectionOrigin};
use parity_multiaddr::Multiaddr;
use std::{mem::ManuallyDrop, str::FromStr, time::Duration};
use tokio::{
    runtime::{Handle, Runtime},
    time::timeout,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

static PROTOCOL: ProtocolId = ProtocolId::MempoolDirectSend;

fn build_test_identity(peer_id: PeerId) -> Identity {
    Identity::new(peer_id, Vec::new())
}

fn build_test_peer(
    executor: Handle,
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
    let connection = Connection {
        metadata: ConnectionMetadata::new(
            identity,
            ConnectionId::default(),
            Multiaddr::from_str("/ip4/127.0.0.1/tcp/8081").unwrap(),
            origin,
        ),
        socket: a,
    };

    let peer = Peer::new(
        executor,
        connection,
        peer_req_rx,
        peer_notifs_tx,
        peer_rpc_notifs_tx,
        peer_direct_send_notifs_tx,
    );
    let peer_handle = PeerHandle::new(peer_id, peer_req_tx);

    (
        peer,
        peer_handle,
        b,
        peer_notifs_rx,
        peer_rpc_notifs_rx,
        peer_direct_send_notifs_rx,
    )
}

fn build_test_connected_peers(
    executor: Handle,
) -> (
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
    ) = build_test_peer(executor.clone(), ConnectionOrigin::Inbound);
    let (
        mut peer_b,
        peer_handle_b,
        _connection_b,
        peer_notifs_rx_b,
        peer_rpc_notifs_rx_b,
        peer_direct_send_notifs_rx_b,
    ) = build_test_peer(executor, ConnectionOrigin::Outbound);
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
        Some(PeerNotification::PeerDisconnected(conn_info, actual_reason)) => {
            assert_eq!(conn_info.peer_identity().peer_id(), peer_id);
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
    ) = build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let send_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Bytes::from_static(b"hello world"),
    });
    let recv_msg = send_msg.clone();

    let server = async move {
        // The client should then send the network message.
        let mut connection = Framed::new(IoCompat::new(connection), LengthDelimitedCodec::new());
        let msg = connection.next().await.unwrap();
        let msg: NetworkMessage = lcs::from_bytes(&msg.unwrap().freeze()).unwrap();
        assert_eq!(msg, recv_msg);
        connection.close().await.unwrap();
    };

    let client = async move {
        peer_handle.send_message(send_msg, PROTOCOL).await.unwrap();
        ManuallyDrop::new(peer_handle);
    };
    rt.spawn(peer.start());
    rt.block_on(join(server, client));
}

#[test]
fn peer_recv_message() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (
        peer,
        _peer_handle,
        connection,
        _peer_notifs_rx,
        _peer_rpc_notifs_rx,
        mut peer_direct_send_notifs_rx,
    ) = build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let send_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Bytes::from_static(b"hello world"),
    });
    let recv_msg = send_msg.clone();

    let server = async move {
        let mut connection = Framed::new(IoCompat::new(connection), LengthDelimitedCodec::new());
        for _ in 0..30 {
            // The client should then send the network message.
            connection
                .send(lcs::to_bytes(&send_msg).unwrap().into())
                .await
                .unwrap();
        }
        // Client then closes connection.
        connection.close().await.unwrap();
    };

    let client = async move {
        for _ in 0..30 {
            // Wait to receive notification of DirectSendMsg from Peer.
            let received = peer_direct_send_notifs_rx.next().await.unwrap();
            assert!(
                matches!(received, PeerNotification::NewMessage(received_msg) if received_msg == recv_msg)
            );
        }
    };
    rt.spawn(peer.start());
    rt.block_on(join(server, client));
}

// Test that if two peers request to open a substream with each other simultaneously that
// we won't deadlock.
#[test]
fn peer_open_substream_simultaneous() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
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
    ) = build_test_connected_peers(rt.handle().clone());

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

    rt.spawn(peer_a.start());
    rt.spawn(peer_b.start());
    rt.block_on(test);
}

#[test]
fn peer_disconnect_request() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (
        peer,
        mut peer_handle,
        _connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let test = async move {
        peer_handle.disconnect().await;
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::Requested,
            &mut peer_notifs_rx,
        )
        .await;
    };

    rt.block_on(join(test, peer.start()));
}

#[test]
fn peer_disconnect_connection_lost() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (
        peer,
        peer_handle,
        mut connection,
        mut peer_notifs_rx,
        _peer_rpc_notifs_rx,
        _peer_direct_send_notifs_rx,
    ) = build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let test = async move {
        connection.close().await.unwrap();
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx,
        )
        .await;
    };
    rt.block_on(join(test, peer.start()));
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
    ) = build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);
    // Perform yamux handshake in a different task.
    let drop = async move {
        // Do not drop connection.
        ManuallyDrop::new(connection);
        // Drop peer handle.
        drop(peer_handle);
    };
    rt.block_on(join(peer.start(), drop));
}
