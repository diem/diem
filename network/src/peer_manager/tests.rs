// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::{DisconnectReason, PeerNotification},
    peer_manager::{PeerManager, PeerManagerNotification, PeerManagerRequest},
    protocols::identity::{exchange_identity, Identity},
    ProtocolId,
};
use channel;
use futures::{
    io::{AsyncRead, AsyncWrite},
    stream::StreamExt,
};
use libra_config::config::RoleType;
use libra_types::PeerId;
use memsocket::MemorySocket;
use netcore::{
    multiplexing::{
        yamux::{Mode, Yamux},
        StreamMultiplexer,
    },
    negotiate::negotiate_outbound_interactive,
    transport::{boxed::BoxedTransport, memory::MemoryTransport, ConnectionOrigin, TransportExt},
};
use parity_multiaddr::Multiaddr;
use std::fmt::Debug;
use std::{collections::HashMap, io};
use tokio::runtime::Handle;

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
