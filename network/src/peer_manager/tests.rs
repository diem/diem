// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::DisconnectReason,
    peer_manager::{
        conn_status_channel, error::PeerManagerError, ConnectionNotification, ConnectionRequest,
        ConnectionStatusNotification, PeerManager, PeerManagerNotification, PeerManagerRequest,
    },
    protocols::wire::{
        handshake::v1::MessagingProtocolVersion,
        messaging::v1::{NetworkMessage, Nonce},
    },
    transport,
    transport::{Connection, ConnectionId, ConnectionMetadata},
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::{channel::oneshot, io::AsyncWriteExt, sink::SinkExt, stream::StreamExt};
use libra_config::config::RoleType;
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    compat::IoCompat,
    transport::{boxed::BoxedTransport, memory::MemoryTransport, ConnectionOrigin, TransportExt},
};
use parity_multiaddr::Multiaddr;
use std::{collections::HashMap, iter::FromIterator, num::NonZeroUsize, str::FromStr};
use tokio::runtime::Handle;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

const TEST_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;

// Builds a concrete typed transport (instead of using impl Trait) for testing PeerManager.
// Specifically this transport is compatible with the `build_test_connection` test helper making
// it easy to build connections without going through the whole transport pipeline.
pub fn build_test_transport(
) -> BoxedTransport<Connection<MemorySocket>, impl ::std::error::Error + Sync + Send + 'static> {
    let memory_transport = MemoryTransport::default();

    memory_transport
        .and_then(move |socket, addr, origin| async move {
            Ok(Connection {
                socket,
                metadata: ConnectionMetadata::new(
                    PeerId::random(),
                    ConnectionId::default(),
                    addr,
                    origin,
                    MessagingProtocolVersion::V1,
                    [TEST_PROTOCOL].iter().into(),
                ),
            })
        })
        .boxed()
}

fn build_test_connection() -> (MemorySocket, MemorySocket) {
    MemorySocket::new_pair()
}

fn ordered_peer_ids(num: usize) -> Vec<PeerId> {
    let mut ids = Vec::new();
    for _ in 0..num {
        ids.push(PeerId::random());
    }
    ids.sort();
    ids
}

fn build_test_peer_manager(
    executor: Handle,
    peer_id: PeerId,
) -> (
    PeerManager<
        BoxedTransport<Connection<MemorySocket>, impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    libra_channel::Sender<PeerId, ConnectionRequest>,
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_status_channel::Receiver,
) {
    let (peer_manager_request_tx, peer_manager_request_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_reqs_tx, connection_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (hello_tx, hello_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (conn_status_tx, conn_status_rx) = conn_status_channel::new();

    let peer_manager = PeerManager::new(
        executor,
        build_test_transport(),
        peer_id,
        RoleType::Validator,
        "/memory/0".parse().unwrap(),
        peer_manager_request_rx,
        connection_reqs_rx,
        HashMap::from_iter([(TEST_PROTOCOL, hello_tx)].iter().cloned()),
        vec![conn_status_tx],
        1024, /* max concurrent network requests */
        1024, /* max concurrent network notifications */
        1024, /* channel size */
    );

    (
        peer_manager,
        peer_manager_request_tx,
        connection_reqs_tx,
        hello_rx,
        conn_status_rx,
    )
}

async fn ping_pong(connection: &mut MemorySocket) -> Result<(), PeerManagerError> {
    let mut connection = Framed::new(IoCompat::new(connection), LengthDelimitedCodec::new());
    let ping = NetworkMessage::Ping(Nonce(42));
    connection
        .send(lcs::to_bytes(&ping).unwrap().into())
        .await?;
    let raw_pong = connection.next().await.ok_or_else(|| {
        PeerManagerError::TransportError(anyhow::anyhow!("Failed to read pong msg"))
    })??;
    let pong: NetworkMessage = lcs::from_bytes(&raw_pong)?;
    assert_eq!(pong, NetworkMessage::Pong(Nonce(42)));
    Ok(())
}

async fn assert_peer_disconnected_event(
    peer_id: PeerId,
    origin: ConnectionOrigin,
    reason: DisconnectReason,
    peer_manager: &mut PeerManager<
        BoxedTransport<Connection<MemorySocket>, impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
) {
    let connection_event = peer_manager.connection_notifs_rx.select_next_some().await;
    match &connection_event {
        ConnectionNotification::Disconnected(ref actual_metadata, ref actual_reason) => {
            assert_eq!(actual_metadata.peer_id(), peer_id);
            assert_eq!(*actual_reason, reason);
            assert_eq!(actual_metadata.origin(), origin);
            peer_manager.handle_connection_event(connection_event);
        }
        event => {
            panic!("Expected a LostPeer event, received: {:?}", event);
        }
    }
}

// This helper function is used to help identify that the expected connection was dropped due
// to simultaneous dial tie-breaking.  It also checks the correct events were sent from the
// Peer actors to PeerManager's internal_event_rx.
async fn check_correct_connection_is_live(
    mut live_connection: MemorySocket,
    mut dropped_connection: MemorySocket,
    live_connection_origin: ConnectionOrigin,
    dropped_connection_origin: ConnectionOrigin,
    expected_peer_id: PeerId,
    requested_shutdown: bool,
    peer_manager: &mut PeerManager<
        BoxedTransport<Connection<MemorySocket>, impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
) {
    // If PeerManager needed to kill the existing connection we'll see a Requested shutdown
    // event
    if requested_shutdown {
        assert_peer_disconnected_event(
            expected_peer_id,
            dropped_connection_origin,
            DisconnectReason::Requested,
            peer_manager,
        )
        .await;
    }
    // TODO: There's a race here since the connection may not have actually been closed yet.
    // We should not be able to send a ping on the dropped connection.
    let f_open_stream_on_dropped_conn: Result<(), PeerManagerError> = async move {
        // Send ping and wait for pong.
        ping_pong(&mut dropped_connection).await?;
        Ok(())
    }
    .await;
    assert!(f_open_stream_on_dropped_conn.is_err());

    let f_open_stream_on_live_conn: Result<(), PeerManagerError> = async move {
        // Send ping and wait for pong.
        ping_pong(&mut live_connection).await?;
        // Close the connection.
        live_connection.close().await?;
        Ok(())
    }
    .await;
    assert!(f_open_stream_on_live_conn.is_ok());
    assert_peer_disconnected_event(
        expected_peer_id,
        live_connection_origin,
        DisconnectReason::ConnectionLost,
        peer_manager,
    )
    .await;
}

fn create_connection<TSocket: transport::TSocket>(
    socket: TSocket,
    peer_id: PeerId,
    addr: Multiaddr,
    origin: ConnectionOrigin,
    connection_id: ConnectionId,
) -> Connection<TSocket> {
    Connection {
        socket,
        metadata: ConnectionMetadata::new(
            peer_id,
            connection_id,
            addr,
            origin,
            MessagingProtocolVersion::V1,
            [TEST_PROTOCOL].iter().into(),
        ),
    }
}

#[test]
fn peer_manager_simultaneous_dial_two_inbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_statux_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two inbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound1,
            ids[0],
            Multiaddr::from_str("/ip6/::1/tcp/8080").unwrap(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound2,
            ids[0],
            Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(1),
        ));

        // outbound1 should have been dropped since it was the older inbound connection
        check_correct_connection_is_live(
            outbound2,
            outbound1,
            ConnectionOrigin::Inbound,
            ConnectionOrigin::Inbound,
            ids[0],
            true,
            &mut peer_manager,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbound_remote_id_larger() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Inbound first, outbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound1,
            ids[1],
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound2,
            ids[1],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(1),
        ));

        // inbound2 should be dropped because for outbound1 the remote peer has a greater
        // PeerId and is the "dialer"
        check_correct_connection_is_live(
            outbound1,
            inbound2,
            ConnectionOrigin::Inbound,
            ConnectionOrigin::Outbound,
            ids[1],
            false,
            &mut peer_manager,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbound_own_id_larger() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Inbound first, outbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound1,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound2,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(1),
        ));

        // outbound1 should be dropped because for inbound2 PeerManager's PeerId is greater and
        // is the "dialer"
        check_correct_connection_is_live(
            inbound2,
            outbound1,
            ConnectionOrigin::Outbound,
            ConnectionOrigin::Inbound,
            ids[0],
            true,
            &mut peer_manager,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_remote_id_larger() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Outbound first, inbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound1,
            ids[1],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound2,
            ids[1],
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(1),
        ));

        // inbound1 should be dropped because for outbound2 the remote peer has a greater
        // PeerID and is the "dialer"
        check_correct_connection_is_live(
            outbound2,
            inbound1,
            ConnectionOrigin::Inbound,
            ConnectionOrigin::Outbound,
            ids[1],
            true,
            &mut peer_manager,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_own_id_larger() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Outbound first, inbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound1,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            inbound2,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            ConnectionId::from(1),
        ));

        // outbound2 should be dropped because for inbound1 PeerManager's PeerId is greater and
        // is the "dialer"
        check_correct_connection_is_live(
            inbound1,
            outbound2,
            ConnectionOrigin::Outbound,
            ConnectionOrigin::Inbound,
            ids[0],
            false,
            &mut peer_manager,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_two_outbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two Outbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound1,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(0),
        ));

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound2,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(1),
        ));
        // inbound1 should have been dropped since it was the older outbound connection
        check_correct_connection_is_live(
            inbound2,
            inbound1,
            ConnectionOrigin::Outbound,
            ConnectionOrigin::Outbound,
            ids[0],
            true,
            &mut peer_manager,
        )
        .await;
    };
    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_disconnect_event() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, _conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        let (outbound, _inbound) = build_test_connection();
        peer_manager.add_peer(create_connection(
            outbound,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(1),
        ));

        // Create a PeerDisconnect event with an older connection_id.  This would happen if the
        // Disconnected event from a closed connection arrives after the new connection has been
        // added to active_peers.
        let event = ConnectionNotification::Disconnected(
            ConnectionMetadata::new(
                ids[0],
                ConnectionId::from(0),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                MessagingProtocolVersion::V1,
                [TEST_PROTOCOL].iter().into(),
            ),
            DisconnectReason::ConnectionLost,
        );
        peer_manager.handle_connection_event(event);
        // The active connection should still remain.
        assert!(peer_manager.active_peers.contains_key(&ids[0]));
    };

    runtime.block_on(test);
}

#[test]
fn test_dial_disconnect() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx, mut conn_status_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        let (outbound, _inbound) = build_test_connection();
        // Trigger add_peer function PeerManager.
        peer_manager.add_peer(create_connection(
            outbound,
            ids[0],
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            ConnectionId::from(0),
        ));

        // Expect NewPeer notification from PeerManager.
        let conn_notif = conn_status_rx.next().await.unwrap();
        assert!(matches!(
            conn_notif,
            ConnectionStatusNotification::NewPeer(_, _)
        ));

        // Send DisconnectPeer request to PeerManager.
        let (disconnect_resp_tx, disconnect_resp_rx) = oneshot::channel();
        peer_manager
            .handle_connection_request(ConnectionRequest::DisconnectPeer(
                ids[0],
                disconnect_resp_tx,
            ))
            .await;

        // Send disconnected event from Peer to PeerManaager
        let event = ConnectionNotification::Disconnected(
            ConnectionMetadata::new(
                ids[0],
                ConnectionId::from(0),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                MessagingProtocolVersion::V1,
                [TEST_PROTOCOL].iter().into(),
            ),
            DisconnectReason::Requested,
        );
        peer_manager.handle_connection_event(event);

        // Expect LostPeer notification from PeerManager.
        let conn_notif = conn_status_rx.next().await.unwrap();
        assert!(matches!(
            conn_notif,
            ConnectionStatusNotification::LostPeer(_, _, _)
        ));

        // Sender of disconnect request should receive acknowledgement once connection is closed.
        disconnect_resp_rx.await.unwrap().unwrap();
    };

    runtime.block_on(test);
}
