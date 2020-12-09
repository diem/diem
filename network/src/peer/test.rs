// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    constants::MAX_FRAME_SIZE,
    peer::{DisconnectReason, Peer, PeerHandle, PeerNotification},
    protocols::{
        direct_send::Message,
        wire::{
            handshake::v1::MessagingProtocolVersion,
            messaging::v1::{
                DirectSendMsg, NetworkMessage, NetworkMessageSink, NetworkMessageStream,
            },
        },
    },
    transport::{Connection, ConnectionId, ConnectionMetadata},
    ProtocolId,
};
use bytes::Bytes;
use diem_config::network_id::NetworkContext;
use diem_network_address::NetworkAddress;
use diem_types::PeerId;
use futures::{
    future,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    stream::StreamExt,
    SinkExt,
};
use memsocket::MemorySocket;
use netcore::{compat::IoCompat, transport::ConnectionOrigin};
use std::str::FromStr;
use tokio::runtime::{Handle, Runtime};

static PROTOCOL: ProtocolId = ProtocolId::MempoolDirectSend;

fn build_test_peer(
    executor: Handle,
    origin: ConnectionOrigin,
) -> (
    Peer<MemorySocket>,
    PeerHandle,
    MemorySocket,
    channel::Receiver<PeerNotification>,
    channel::Receiver<PeerNotification>,
) {
    let (a, b) = MemorySocket::new_pair();
    let peer_id = PeerId::random();
    let (peer_notifs_tx, peer_notifs_rx) = channel::new_test(1);
    let (peer_rpc_notifs_tx, peer_rpc_notifs_rx) = channel::new_test(1);
    let (peer_req_tx, peer_req_rx) = channel::new_test(0);
    let connection = Connection {
        metadata: ConnectionMetadata::new(
            peer_id,
            ConnectionId::default(),
            NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8081").unwrap(),
            origin,
            MessagingProtocolVersion::V1,
            [].iter().into(),
        ),
        socket: a,
    };
    let connection_metadata = connection.metadata.clone();

    let peer = Peer::new(
        NetworkContext::mock(),
        executor,
        connection,
        peer_req_rx,
        peer_notifs_tx,
        peer_rpc_notifs_tx,
        MAX_FRAME_SIZE,
    );
    let peer_handle = PeerHandle::new(NetworkContext::mock(), connection_metadata, peer_req_tx);

    (peer, peer_handle, b, peer_notifs_rx, peer_rpc_notifs_rx)
}

fn build_test_connected_peers(
    executor: Handle,
) -> (
    (
        Peer<MemorySocket>,
        PeerHandle,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
    ),
    (
        Peer<MemorySocket>,
        PeerHandle,
        channel::Receiver<PeerNotification>,
        channel::Receiver<PeerNotification>,
    ),
) {
    let (peer_a, peer_handle_a, connection_a, peer_notifs_rx_a, peer_rpc_notifs_rx_a) =
        build_test_peer(executor.clone(), ConnectionOrigin::Inbound);
    let (mut peer_b, peer_handle_b, _connection_b, peer_notifs_rx_b, peer_rpc_notifs_rx_b) =
        build_test_peer(executor, ConnectionOrigin::Outbound);
    // Make sure both peers are connected
    peer_b.connection = Some(connection_a);
    (
        (
            peer_a,
            peer_handle_a,
            peer_notifs_rx_a,
            peer_rpc_notifs_rx_a,
        ),
        (
            peer_b,
            peer_handle_b,
            peer_notifs_rx_b,
            peer_rpc_notifs_rx_b,
        ),
    )
}

fn build_network_sink_stream<'a>(
    connection: &'a mut MemorySocket,
) -> (
    NetworkMessageSink<impl AsyncWrite + 'a>,
    NetworkMessageStream<impl AsyncRead + 'a>,
) {
    let (read_half, write_half) = tokio::io::split(IoCompat::new(connection));
    let sink = NetworkMessageSink::new(IoCompat::new(write_half), MAX_FRAME_SIZE);
    let stream = NetworkMessageStream::new(IoCompat::new(read_half), MAX_FRAME_SIZE);
    (sink, stream)
}

async fn assert_peer_disconnected_event(
    peer_id: PeerId,
    reason: DisconnectReason,
    peer_notifs_rx: &mut channel::Receiver<PeerNotification>,
) {
    match peer_notifs_rx.next().await {
        Some(PeerNotification::PeerDisconnected(conn_info, actual_reason)) => {
            assert_eq!(conn_info.remote_peer_id, peer_id);
            assert_eq!(actual_reason, reason);
        }
        event => panic!("Expected a PeerDisconnected, received: {:?}", event),
    }
}

// Sending an outbound DirectSend should write it to the wire.
#[test]
fn peer_send_message() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (peer, mut peer_handle, mut connection, _peer_notifs_rx, _peer_rpc_notifs_rx) =
        build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);
    let (mut client_sink, mut client_stream) = build_network_sink_stream(&mut connection);

    let send_msg = Message {
        protocol_id: PROTOCOL,
        mdata: Bytes::from("hello world"),
    };
    let recv_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Vec::from("hello world"),
    });

    let client = async {
        // Client should receive the direct send messages.
        for _ in 0..30 {
            let msg = client_stream.next().await.unwrap().unwrap();
            assert_eq!(msg, recv_msg);
        }
        // Client then closes the connection.
        client_sink.close().await.unwrap();
    };

    let server = async {
        // Server sends some direct send messages.
        for _ in 0..30 {
            peer_handle
                .send_direct_send(send_msg.clone())
                .await
                .unwrap();
        }
    };
    rt.block_on(future::join3(peer.start(), server, client));
}

// Reading an inbound DirectSendMsg off the wire should notify the PeerManager of
// an inbound DirectSend.
#[test]
fn peer_recv_message() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (peer, _peer_handle, connection, mut peer_notifs_rx, _peer_rpc_notifs_rx) =
        build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let send_msg = NetworkMessage::DirectSendMsg(DirectSendMsg {
        protocol_id: PROTOCOL,
        priority: 0,
        raw_msg: Vec::from("hello world"),
    });
    let recv_msg = PeerNotification::RecvDirectSend(Message {
        protocol_id: PROTOCOL,
        mdata: Bytes::from("hello world"),
    });

    let client = async move {
        let mut connection = NetworkMessageSink::new(connection, MAX_FRAME_SIZE);
        for _ in 0..30 {
            // The client should then send the network message.
            connection.send(&send_msg).await.unwrap();
        }
        // Client then closes connection.
        connection.close().await.unwrap();
    };

    let server = async move {
        for _ in 0..30 {
            // Wait to receive notification of DirectSendMsg from Peer.
            let received = peer_notifs_rx.next().await.unwrap();
            assert_eq!(recv_msg, received);
        }
    };
    rt.block_on(future::join3(peer.start(), server, client));
}

// Two connected Peer actors should be able to send/recv a DirectSend from each
// other and then shutdown gracefully.
#[test]
fn peers_send_message_concurrent() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (
        (peer_a, mut peer_handle_a, mut peer_notifs_rx_a, _peer_rpc_notifs_rx_a),
        (peer_b, mut peer_handle_b, mut peer_notifs_rx_b, _peer_rpc_notifs_rx_b),
    ) = build_test_connected_peers(rt.handle().clone());

    let test = async move {
        let msg_a = Message {
            protocol_id: PROTOCOL,
            mdata: Bytes::from("hello world"),
        };
        let msg_b = Message {
            protocol_id: PROTOCOL,
            mdata: Bytes::from("namaste"),
        };

        // Peer A -> msg_a -> Peer B
        peer_handle_a.send_direct_send(msg_a.clone()).await.unwrap();
        // Peer A <- msg_b <- Peer B
        peer_handle_b.send_direct_send(msg_b.clone()).await.unwrap();

        // Check that each peer received the other's message
        let notif_a = peer_notifs_rx_a.next().await;
        let notif_b = peer_notifs_rx_b.next().await;
        assert_eq!(notif_a, Some(PeerNotification::RecvDirectSend(msg_b)));
        assert_eq!(notif_b, Some(PeerNotification::RecvDirectSend(msg_a)));

        // Shut one peers and the other should shutdown due to ConnectionLost
        peer_handle_a.disconnect().await;

        // Check that we received both shutdown events
        assert_peer_disconnected_event(
            peer_handle_a.peer_id(),
            DisconnectReason::Requested,
            &mut peer_notifs_rx_a,
        )
        .await;
        assert_peer_disconnected_event(
            peer_handle_b.peer_id(),
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx_b,
        )
        .await;
    };

    rt.block_on(future::join3(peer_a.start(), peer_b.start(), test));
}

// PeerManager can request a Peer to shutdown.
#[test]
fn peer_disconnect_request() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (peer, mut peer_handle, _connection, mut peer_notifs_rx, _peer_rpc_notifs_rx) =
        build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let test = async move {
        peer_handle.disconnect().await;
        assert_peer_disconnected_event(
            peer_handle.peer_id(),
            DisconnectReason::Requested,
            &mut peer_notifs_rx,
        )
        .await;
    };

    rt.block_on(future::join(peer.start(), test));
}

// Peer will shutdown if the underlying connection is lost.
#[test]
fn peer_disconnect_connection_lost() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (peer, peer_handle, mut connection, mut peer_notifs_rx, _peer_rpc_notifs_rx) =
        build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let test = async move {
        connection.close().await.unwrap();
        assert_peer_disconnected_event(
            peer_handle.peer_id(),
            DisconnectReason::ConnectionLost,
            &mut peer_notifs_rx,
        )
        .await;
    };
    rt.block_on(future::join(peer.start(), test));
}

#[test]
fn peer_terminates_when_request_tx_has_dropped() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (peer, peer_handle, _connection, _peer_notifs_rx, _peer_rpc_notifs_rx) =
        build_test_peer(rt.handle().clone(), ConnectionOrigin::Inbound);

    let drop = async move {
        // Drop peer handle.
        drop(peer_handle);
    };
    rt.block_on(future::join(peer.start(), drop));
}
