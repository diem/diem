// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer_manager::{
        DisconnectReason, Peer, PeerHandle, PeerManager, PeerManagerNotification,
        PeerManagerRequest, PeerNotification,
    },
    protocols::identity::{exchange_identity, Identity},
    ProtocolId,
};
use channel;
use futures::{
    channel::oneshot,
    executor::block_on,
    future::join,
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
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
    negotiate::{negotiate_inbound, negotiate_outbound_interactive},
    transport::{boxed::BoxedTransport, memory::MemoryTransport, ConnectionOrigin, TransportExt},
};
use parity_multiaddr::Multiaddr;
use std::fmt::Debug;
use std::{collections::HashMap, io, time::Duration};
use tokio::{runtime::Handle, time::timeout};

const HELLO_PROTOCOL: &[u8] = b"/hello-world/1.0.0";

// Builds a concrete typed transport (instead of using impl Trait) for testing PeerManager.
// Specifically this transport is compatible with the `build_test_connection` test helper making
// it easy to build connections without going through the whole transport pipeline.
pub fn build_test_transport(
    own_identity: Identity,
) -> BoxedTransport<(Identity, Yamux<MemorySocket>), impl ::std::error::Error> {
    let memory_transport = MemoryTransport::default();
    memory_transport
        .and_then(|socket, origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok(muxer)
            }
        })
        .and_then(move |muxer, origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;

                Ok((identity, muxer))
            }
        })
        .boxed()
}

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

//
// Simultaneous Dial Tests
//

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
        BoxedTransport<(Identity, Yamux<MemorySocket>), impl std::error::Error>,
        Yamux<MemorySocket>,
    >,
    channel::Sender<PeerManagerRequest<impl AsyncRead + AsyncWrite>>,
    channel::Receiver<PeerManagerNotification<impl AsyncRead + AsyncWrite>>,
) {
    let protocol = ProtocolId::from_static(HELLO_PROTOCOL);
    let (peer_manager_request_tx, peer_manager_request_rx) = channel::new_test(0);
    let (hello_tx, hello_rx) = channel::new_test(0);
    let mut protocol_handlers = HashMap::new();
    protocol_handlers.insert(protocol, hello_tx);

    let peer_manager = PeerManager::new(
        build_test_transport(Identity::new(peer_id, vec![], RoleType::Validator)),
        executor,
        peer_id,
        "/memory/0".parse().unwrap(),
        peer_manager_request_rx,
        protocol_handlers,
        Vec::new(),
    );

    (peer_manager, peer_manager_request_tx, hello_rx)
}

async fn open_hello_substream<T: StreamMultiplexer>(connection: &T) -> io::Result<()> {
    let outbound = connection.open_outbound().await?;
    let (_, _) = negotiate_outbound_interactive(outbound, [HELLO_PROTOCOL]).await?;
    Ok(())
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

// This helper function is used to help identify that the expected connection was dropped due
// to simultaneous dial tie-breaking.  It also checks the correct events were sent from the
// Peer actors to PeerManager's internal_event_rx.
async fn check_correct_connection_is_live<TMuxer: StreamMultiplexer>(
    live_connection: TMuxer,
    dropped_connection: TMuxer,
    expected_peer_id: PeerId,
    expected_role: RoleType,
    requested_shutdown: bool,
    mut peer_notifs_rx: &mut channel::Receiver<PeerNotification<TMuxer::Substream>>,
) {
    // If PeerManager needed to kill the existing connection we'll see a Requested shutdown
    // event
    if requested_shutdown {
        assert_peer_disconnected_event(
            expected_peer_id,
            expected_role,
            DisconnectReason::Requested,
            &mut peer_notifs_rx,
        )
        .await;
    }

    assert!(open_hello_substream(&dropped_connection).await.is_err());
    assert!(open_hello_substream(&live_connection).await.is_ok());

    // Make sure we get the incoming substream and shutdown events
    assert_new_substream_event(expected_peer_id, &mut peer_notifs_rx).await;

    live_connection.close().await.unwrap();

    assert_peer_disconnected_event(
        expected_peer_id,
        expected_role,
        DisconnectReason::ConnectionLost,
        &mut peer_notifs_rx,
    )
    .await;
}

#[test]
fn peer_manager_simultaneous_dial_two_inbound() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two inbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound2,
            )
            .await;

        // outbound1 should have been dropped since it was the older inbound connection
        check_correct_connection_is_live(
            outbound2,
            outbound1,
            ids[0],
            role,
            true,
            &mut peer_manager.peer_notifs_rx,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbout_remote_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Inbound first, outbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[1]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[1]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound2,
            )
            .await;

        // inbound2 should be dropped because for outbound1 the remote peer has a greater
        // PeerId and is the "dialer"
        check_correct_connection_is_live(
            outbound1,
            inbound2,
            ids[1],
            role,
            false,
            &mut peer_manager.peer_notifs_rx,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbout_own_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Inbound first, outbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound2,
            )
            .await;

        // outbound1 should be dropped because for inbound2 PeerManager's PeerId is greater and
        // is the "dialer"
        check_correct_connection_is_live(
            inbound2,
            outbound1,
            ids[0],
            role,
            true,
            &mut peer_manager.peer_notifs_rx,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_remote_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[0]);

    let test = async move {
        //
        // Outbound first, inbound second with own_peer_id < remote_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[1]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[1]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound2,
            )
            .await;

        // inbound1 should be dropped because for outbound2 the remote peer has a greater
        // PeerID and is the "dialer"
        check_correct_connection_is_live(
            outbound2,
            inbound1,
            ids[1],
            role,
            true,
            &mut peer_manager.peer_notifs_rx,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_own_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Outbound first, inbound second with remote_peer_id < own_peer_id
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Inbound,
                inbound2,
            )
            .await;

        // outbound2 should be dropped because for inbound1 PeerManager's PeerId is greater and
        // is the "dialer"
        check_correct_connection_is_live(
            inbound1,
            outbound2,
            ids[0],
            role,
            false,
            &mut peer_manager.peer_notifs_rx,
        )
        .await;
    };

    runtime.block_on(test);
}

#[test]
fn peer_manager_simultaneous_dial_two_outbound() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        //
        // Two Outbound connections
        //
        let (outbound1, inbound1) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound1,
            )
            .await;
        let (outbound2, inbound2) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound2,
            )
            .await;

        // inbound2 should have been dropped since it was the second outbound connection
        check_correct_connection_is_live(
            inbound1,
            inbound2,
            ids[0],
            role,
            false,
            &mut peer_manager.peer_notifs_rx,
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
    let role = RoleType::Validator;
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.handle().clone(), ids[1]);

    let test = async move {
        let (outbound, _inbound) = build_test_connection();
        peer_manager
            .add_peer(
                build_test_identity(ids[0]),
                Multiaddr::empty(),
                ConnectionOrigin::Outbound,
                outbound,
            )
            .await;

        // Create a PeerDisconnect event with the opposite origin of the one stored in
        // PeerManager to ensure that handling the event won't cause the PeerHandle to be
        // removed from PeerManager
        let event = PeerNotification::PeerDisconnected(
            ids[0],
            role,
            ConnectionOrigin::Inbound,
            DisconnectReason::ConnectionLost,
        );
        peer_manager.handle_peer_event(event).await;

        assert!(peer_manager.active_peers.contains_key(&ids[0]));
    };

    runtime.block_on(test);
}
