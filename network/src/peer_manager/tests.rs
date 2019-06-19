// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer_manager::{
        DisconnectReason, InternalEvent, Peer, PeerHandle, PeerManager, PeerManagerNotification,
        PeerManagerRequest,
    },
    protocols::{
        identity::{exchange_identity, Identity},
        peer_id_exchange::PeerIdExchange,
    },
    ProtocolId,
};
use channel;
use futures::{
    channel::oneshot,
    compat::Compat01As03,
    executor::block_on,
    future::{join, FutureExt, TryFutureExt},
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    stream::StreamExt,
};
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
use std::{collections::HashMap, io, time::Duration};
use tokio::{runtime::TaskExecutor, timer::Timeout};
use types::PeerId;

const HELLO_PROTOCOL: &[u8] = b"/hello-world/1.0.0";

// Builds a concrete typed transport (instead of using impl Trait) for testing PeerManager.
// Specifically this transport is compatible with the `build_test_connection` test helper making
// it easy to build connections without going through the whole transport pipeline.
fn build_test_transport(
    own_peer_id: PeerId,
) -> BoxedTransport<(Identity, Yamux<MemorySocket>), std::io::Error> {
    let memory_transport = MemoryTransport::default();
    let peer_id_exchange_config = PeerIdExchange::new(own_peer_id);
    let own_identity = Identity::new(own_peer_id, Vec::new());

    memory_transport
        .and_then(move |socket, origin| peer_id_exchange_config.exchange_peer_id(socket, origin))
        .and_then(|(peer_id, socket), origin| {
            async move {
                let muxer = Yamux::upgrade_connection(socket, origin).await?;
                Ok((peer_id, muxer))
            }
        })
        .and_then(move |(peer_id, muxer), origin| {
            async move {
                let (identity, muxer) = exchange_identity(&own_identity, muxer, origin).await?;
                assert_eq!(identity.peer_id(), peer_id);

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
    Identity::new(peer_id, Vec::new())
}

fn build_test_peer(
    origin: ConnectionOrigin,
) -> (
    Peer<Yamux<MemorySocket>>,
    PeerHandle<StreamHandle<MemorySocket>>,
    Yamux<MemorySocket>,
    channel::Receiver<InternalEvent<Yamux<MemorySocket>>>,
) {
    let (a, b) = build_test_connection();
    let identity = build_test_identity(PeerId::random());
    let peer_id = identity.peer_id();
    let (internal_event_tx, internal_event_rx) = channel::new_test(1);
    let (peer_req_tx, peer_req_rx) = channel::new_test(0);

    let peer = Peer::new(
        identity,
        a,
        origin,
        vec![ProtocolId::from_static(HELLO_PROTOCOL)],
        internal_event_tx,
        peer_req_rx,
    );
    let peer_handle = PeerHandle::new(peer_id, Multiaddr::empty(), origin, peer_req_tx);

    (peer, peer_handle, b, internal_event_rx)
}

fn build_test_connected_peers() -> (
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle<MemorySocket>>,
        channel::Receiver<InternalEvent<Yamux<MemorySocket>>>,
    ),
    (
        Peer<Yamux<MemorySocket>>,
        PeerHandle<StreamHandle<MemorySocket>>,
        channel::Receiver<InternalEvent<Yamux<MemorySocket>>>,
    ),
) {
    let (peer_a, peer_handle_a, connection_a, internal_event_rx_a) =
        build_test_peer(ConnectionOrigin::Inbound);
    let (mut peer_b, peer_handle_b, _connection_b, internal_event_rx_b) =
        build_test_peer(ConnectionOrigin::Outbound);
    // Make sure both peers are connected
    peer_b.connection = connection_a;

    (
        (peer_a, peer_handle_a, internal_event_rx_a),
        (peer_b, peer_handle_b, internal_event_rx_b),
    )
}

#[test]
fn peer_open_substream() {
    let (peer, _peer_handle, connection, _internal_event_rx) =
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
        (peer_a, mut peer_handle_a, mut internal_event_rx_a),
        (peer_b, mut peer_handle_b, mut internal_event_rx_b),
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
        let timeout_a = Compat01As03::new(Timeout::new(
            substream_rx_a.boxed().compat(),
            Duration::from_secs(10),
        ));
        let timeout_b = Compat01As03::new(Timeout::new(
            substream_rx_b.boxed().compat(),
            Duration::from_secs(10),
        ));
        let _ = timeout_a.await.unwrap().unwrap();
        let _ = timeout_b.await.unwrap().unwrap();

        // Check that we received the new inbound substream for both peers
        assert_new_substream_event(peer_handle_a.peer_id, &mut internal_event_rx_a).await;
        assert_new_substream_event(peer_handle_b.peer_id, &mut internal_event_rx_b).await;

        // Shut one peers and the other should shutdown due to ConnectionLost
        peer_handle_a.disconnect().await;

        // Check that we received both shutdown events
        assert_peer_disconnected_event(
            peer_handle_a.peer_id,
            DisconnectReason::Requested,
            &mut internal_event_rx_a,
        )
        .await;
        assert_peer_disconnected_event(
            peer_handle_b.peer_id,
            DisconnectReason::ConnectionLost,
            &mut internal_event_rx_b,
        )
        .await;
    };

    runtime.spawn(peer_a.start().boxed().unit_error().compat());
    runtime.spawn(peer_b.start().boxed().unit_error().compat());

    runtime
        .block_on_all(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_disconnect_request() {
    let (peer, mut peer_handle, _connection, mut internal_event_rx) =
        build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        peer_handle.disconnect().await;
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::Requested,
            &mut internal_event_rx,
        )
        .await;
    };

    block_on(join(test, peer.start()));
}

#[test]
fn peer_disconnect_connection_lost() {
    let (peer, peer_handle, connection, mut internal_event_rx) =
        build_test_peer(ConnectionOrigin::Inbound);

    let test = async move {
        connection.close().await.unwrap();
        assert_peer_disconnected_event(
            peer_handle.peer_id,
            DisconnectReason::ConnectionLost,
            &mut internal_event_rx,
        )
        .await;
    };

    block_on(join(test, peer.start()));
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
    executor: TaskExecutor,
    peer_id: PeerId,
) -> (
    PeerManager<
        BoxedTransport<(Identity, Yamux<MemorySocket>), std::io::Error>,
        Yamux<MemorySocket>,
    >,
    channel::Sender<PeerManagerRequest<impl AsyncRead + AsyncWrite>>,
    channel::Receiver<PeerManagerNotification<impl AsyncRead + AsyncWrite>>,
) {
    let protocol = ProtocolId::from_static(HELLO_PROTOCOL);
    let (peer_manager_request_tx, peer_manager_request_rx) = channel::new_test(0);
    let (hello_tx, hello_rx) = channel::new_test(0);
    let mut protocol_handlers = HashMap::new();
    protocol_handlers.insert(protocol.clone(), hello_tx);

    let peer_manager = PeerManager::new(
        build_test_transport(peer_id),
        executor.clone(),
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

async fn assert_new_substream_event<TMuxer: StreamMultiplexer>(
    peer_id: PeerId,
    internal_event_rx: &mut channel::Receiver<InternalEvent<TMuxer>>,
) {
    match internal_event_rx.next().await {
        Some(InternalEvent::NewSubstream(actual_peer_id, _)) => {
            assert_eq!(actual_peer_id, peer_id);
        }
        event => {
            panic!("Expected a NewSubstream, received: {:?}", event);
        }
    }
}

async fn assert_peer_disconnected_event<TMuxer: StreamMultiplexer>(
    peer_id: PeerId,
    reason: DisconnectReason,
    internal_event_rx: &mut channel::Receiver<InternalEvent<TMuxer>>,
) {
    match internal_event_rx.next().await {
        Some(InternalEvent::PeerDisconnected(actual_peer_id, _origin, actual_reason)) => {
            assert_eq!(actual_peer_id, peer_id);
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
    requested_shutdown: bool,
    mut internal_event_rx: &mut channel::Receiver<InternalEvent<TMuxer>>,
) {
    // If PeerManager needed to kill the existing connection we'll see a Requested shutdown
    // event
    if requested_shutdown {
        assert_peer_disconnected_event(
            expected_peer_id,
            DisconnectReason::Requested,
            &mut internal_event_rx,
        )
        .await;
    }

    assert!(open_hello_substream(&dropped_connection).await.is_err());
    assert!(open_hello_substream(&live_connection).await.is_ok());

    // Make sure we get the incoming substream and shutdown events
    assert_new_substream_event(expected_peer_id, &mut internal_event_rx).await;

    live_connection.close().await.unwrap();

    assert_peer_disconnected_event(
        expected_peer_id,
        DisconnectReason::ConnectionLost,
        &mut internal_event_rx,
    )
    .await;
}

#[test]
fn peer_manager_simultaneous_dial_two_inbound() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[1]);

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

        // outbound2 should have been dropped since it was the second inbound connection
        check_correct_connection_is_live(
            outbound1,
            outbound2,
            ids[0],
            false,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbout_remote_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[0]);

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
            false,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_inbound_outbout_own_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[1]);

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
            true,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_remote_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[0]);

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
            true,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_outbound_inbound_own_id_larger() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[1]);

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
            false,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_two_outbound() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[1]);

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
            false,
            &mut peer_manager.internal_event_rx,
        )
        .await;
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn peer_manager_simultaneous_dial_disconnect_event() {
    let mut runtime = ::tokio::runtime::Runtime::new().unwrap();

    // Create a list of ordered PeerIds so we can ensure how PeerIds will be compared.
    let ids = ordered_peer_ids(2);
    let (mut peer_manager, _request_tx, _hello_rx) =
        build_test_peer_manager(runtime.executor(), ids[1]);

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
        let event = InternalEvent::PeerDisconnected(
            ids[0],
            ConnectionOrigin::Inbound,
            DisconnectReason::ConnectionLost,
        );
        peer_manager.handle_internal_event(event).await;

        assert!(peer_manager.active_peers.contains_key(&ids[0]));
    };

    runtime
        .block_on(test.boxed().unit_error().compat())
        .unwrap();
}
