// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::DisconnectReason,
    peer_manager::{
        ConnectionNotification, ConnectionRequest, PeerManager, PeerManagerNotification,
        PeerManagerRequest,
    },
    protocols::identity::{exchange_identity, Identity},
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::{channel::oneshot, stream::StreamExt};
use libra_config::config::RoleType;
use libra_logger::prelude::*;
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    multiplexing::{yamux::Yamux, Control, StreamMultiplexer},
    negotiate::negotiate_outbound_interactive,
    transport::{boxed::BoxedTransport, memory::MemoryTransport, ConnectionOrigin, TransportExt},
};
use parity_multiaddr::Multiaddr;
use std::{
    collections::{HashMap, HashSet},
    io,
    iter::FromIterator,
    num::NonZeroUsize,
    str::FromStr,
};
use tokio::runtime::Handle;

const TEST_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;

// Builds a concrete typed transport (instead of using impl Trait) for testing PeerManager.
// Specifically this transport is compatible with the `build_test_connection` test helper making
// it easy to build connections without going through the whole transport pipeline.
pub fn build_test_transport(
    own_identity: Identity,
) -> BoxedTransport<(Identity, MemorySocket), impl ::std::error::Error + Sync + Send + 'static> {
    let memory_transport = MemoryTransport::default();
    memory_transport
        .and_then(move |socket, origin| async move {
            let (identity, socket) = exchange_identity(&own_identity, socket, origin).await?;
            Ok((identity, socket))
        })
        .boxed()
}

fn build_test_connection() -> (MemorySocket, MemorySocket) {
    MemorySocket::new_pair()
}

fn build_test_identity(peer_id: PeerId) -> Identity {
    Identity::new(peer_id, Vec::new())
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
        BoxedTransport<(Identity, MemorySocket), impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerRequest>,
    libra_channel::Sender<PeerId, ConnectionRequest>,
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerNotification>,
) {
    let (peer_manager_request_tx, peer_manager_request_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_reqs_tx, connection_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (hello_tx, hello_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);

    let peer_manager = PeerManager::new(
        executor,
        build_test_transport(Identity::new(peer_id, vec![])),
        peer_id,
        RoleType::Validator,
        "/memory/0".parse().unwrap(),
        peer_manager_request_rx,
        connection_reqs_rx,
        HashSet::from_iter([TEST_PROTOCOL].iter().cloned()), /* rpc protocols */
        HashSet::new(),                                      /* direct-send protocols */
        HashMap::from_iter([(TEST_PROTOCOL, hello_tx)].iter().cloned()),
        vec![],
        1024, /* max concurrent network requests */
        1024, /* max concurrent network notifications */
        1024, /* channel size */
    );

    (
        peer_manager,
        peer_manager_request_tx,
        connection_reqs_tx,
        hello_rx,
    )
}

async fn create_yamux_socket(
    connection: MemorySocket,
    origin: ConnectionOrigin,
) -> io::Result<(
    <Yamux<MemorySocket> as StreamMultiplexer>::Listener,
    <Yamux<MemorySocket> as StreamMultiplexer>::Control,
)> {
    debug!("Starting yamux upgrade");
    // Perform yamux upgrade as the opposite end-point.
    let res = Yamux::upgrade_connection(
        connection,
        if origin == ConnectionOrigin::Inbound {
            ConnectionOrigin::Outbound
        } else {
            ConnectionOrigin::Inbound
        },
    )
    .await?
    .start()
    .await;
    debug!("Completed yamux upgrade");
    Ok(res)
}

async fn open_hello_substream<T: Control>(connection: &mut T) -> io::Result<()> {
    let outbound = connection.open_stream().await?;
    let (_, _) =
        negotiate_outbound_interactive(outbound, [lcs::to_bytes(&TEST_PROTOCOL).unwrap()]).await?;
    Ok(())
}

async fn assert_peer_disconnected_event(
    peer_id: PeerId,
    origin: ConnectionOrigin,
    reason: DisconnectReason,
    peer_manager: &mut PeerManager<
        BoxedTransport<(Identity, MemorySocket), impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
) {
    let connection_event = peer_manager.connection_notifs_rx.select_next_some().await;
    match &connection_event {
        ConnectionNotification::Disconnected(
            ref actual_identity,
            ref _actual_addr,
            ref actual_origin,
            ref actual_reason,
        ) => {
            assert_eq!(actual_identity.peer_id(), peer_id);
            assert_eq!(*actual_reason, reason);
            assert_eq!(*actual_origin, origin);
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
    live_connection: MemorySocket,
    dropped_connection: MemorySocket,
    live_connection_origin: ConnectionOrigin,
    dropped_connection_origin: ConnectionOrigin,
    expected_peer_id: PeerId,
    requested_shutdown: bool,
    peer_manager: &mut PeerManager<
        BoxedTransport<(Identity, MemorySocket), impl std::error::Error + Sync + Send + 'static>,
        MemorySocket,
    >,
) {
    // We should be able to negotiate yamux handshake on both connections.
    let f_open_stream_on_dropped_conn: io::Result<()> = async move {
        // Upgrade connection to Yamux.
        let (_, mut control_dropped) =
            create_yamux_socket(dropped_connection, dropped_connection_origin).await?;
        // Open a substream.
        open_hello_substream(&mut control_dropped).await?;
        Ok(())
    }
    .await;
    assert!(f_open_stream_on_dropped_conn.is_err());
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

    let f_open_stream_on_live_conn: io::Result<()> = async move {
        // Upgrade connection to Yamux.
        let (_s, mut control_live) =
            create_yamux_socket(live_connection, live_connection_origin).await?;
        // Open a substream.
        open_hello_substream(&mut control_live).await?;
        // Close the connection.
        control_live.close().await?;
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

#[test]
fn peer_manager_simultaneous_dial_two_inbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two inbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::from_str("/ip6/::1/tcp/8080").unwrap(),
            ConnectionOrigin::Inbound,
            inbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
            ConnectionOrigin::Inbound,
            inbound2,
        );

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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Inbound first, outbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[1]),
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            inbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[1]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound2,
        );

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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Inbound first, outbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            inbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound2,
        );

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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Outbound first, inbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[1]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[1]),
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            inbound2,
        );

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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Outbound first, inbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            inbound2,
        );

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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two Outbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound1,
        );

        let (outbound2, inbound2) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound2,
        );
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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        let (outbound, _inbound) = build_test_connection();
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound,
        );

        // Create a PeerDisconnect event with the opposite origin of the one stored in
        // PeerManager to ensure that handling the event won't cause the PeerHandle to be
        // removed from PeerManager. This would happen if the Disconnected event from a closed
        // connection arrives after the new connection has been added to active_peers.
        let event = ConnectionNotification::Disconnected(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Inbound,
            DisconnectReason::ConnectionLost,
        );
        peer_manager.handle_connection_event(event);
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
    let (mut peer_manager, _request_tx, _connection_reqs_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        let (outbound, _inbound) = build_test_connection();
        // Trigger add_peer function PeerManager.
        peer_manager.add_peer(
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            outbound,
        );

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
            build_test_identity(ids[0]),
            Multiaddr::empty(),
            ConnectionOrigin::Outbound,
            DisconnectReason::Requested,
        );
        peer_manager.handle_connection_event(event);

        // Sender of disconnect request should receive acknowledgement once connection is closed.
        disconnect_resp_rx.await.unwrap().unwrap();
    };

    runtime.block_on(test);
}
