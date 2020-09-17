// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    error::NetworkErrorKind,
    peer_manager::{
        self, conn_notifs_channel, conn_notifs_channel::Sender, ConnectionNotification,
        ConnectionRequestSender, PeerManagerNotification, PeerManagerRequest,
    },
    protocols::{
        direct_send::Message,
        network::{NewNetworkEvents, NewNetworkSender},
    },
    ProtocolId,
};
use anyhow::anyhow;
use channel::{libra_channel, libra_channel::ElementStatus, message_queues::QueueStyle};
use futures::channel::oneshot;
use libra_config::{config::RoleType, network_id::NetworkId};
use libra_network_address::NetworkAddress;
use netcore::transport::ConnectionOrigin;
use std::{num::NonZeroUsize, str::FromStr};
use tokio::runtime::Runtime;

fn get_raw_message(msg: GossipDiscoveryMsg) -> Message {
    Message {
        protocol_id: ProtocolId::DiscoveryDirectSend,
        mdata: lcs::to_bytes(&msg).unwrap().into(),
    }
}

fn parse_raw_message(msg: Message) -> Result<GossipDiscoveryMsg, NetworkError> {
    assert_eq!(msg.protocol_id, ProtocolId::DiscoveryDirectSend);
    let msg: GossipDiscoveryMsg = lcs::from_bytes(&msg.mdata)
        .map_err(|err| anyhow!(err).context(NetworkErrorKind::ParsingError))?;
    Ok(msg)
}

fn setup_discovery(
    rt: &mut Runtime,
    peer_id: PeerId,
    addrs: Vec<NetworkAddress>,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    channel::Receiver<ConnectivityRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_notifs_channel::Sender,
    channel::Sender<()>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_reqs_tx, _) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(1);
    let (network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let discovery = {
        GossipDiscovery::new(
            Arc::new(NetworkContext::new(
                NetworkId::Validator,
                RoleType::Validator,
                peer_id,
            )),
            addrs,
            ticker_rx,
            GossipDiscoveryNetworkSender::new(
                PeerManagerRequestSender::new(peer_mgr_reqs_tx),
                ConnectionRequestSender::new(connection_reqs_tx),
            ),
            GossipDiscoveryNetworkEvents::new(network_notifs_rx, connection_notifs_rx),
            conn_mgr_reqs_tx,
        )
    };
    rt.spawn(discovery.start());
    (
        peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
        network_notifs_tx,
        connection_notifs_tx,
        ticker_tx,
    )
}

fn send_new_peer_with_feedback(
    connection_notifs_tx: &mut Sender,
    peer_id: PeerId,
    address: NetworkAddress,
    delivered_tx: oneshot::Sender<ElementStatus<ConnectionNotification>>,
) {
    let notif = peer_manager::ConnectionNotification::NewPeer(
        peer_id,
        address,
        ConnectionOrigin::Inbound,
        NetworkContext::mock(),
    );
    connection_notifs_tx
        .push_with_feedback(peer_id, notif, Some(delivered_tx))
        .unwrap();
}

async fn expect_address_update(
    conn_mgr_reqs_rx: &mut channel::Receiver<ConnectivityRequest>,
    expected_address_map: HashMap<PeerId, Vec<NetworkAddress>>,
) {
    match conn_mgr_reqs_rx.next().await.unwrap() {
        ConnectivityRequest::UpdateAddresses(src, address_map) => {
            assert_eq!(DiscoverySource::Gossip, src);
            assert_eq!(expected_address_map, address_map);
        }
        req => {
            panic!("Unexpected request to connectivity manager: {:?}", req);
        }
    }
}

#[test]
// Test behavior on receipt of an inbound DiscoveryMsg.
fn inbound() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let self_peer_id = PeerId::random();
    let self_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];

    // Setup other peer.
    let other_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();

    // Setup new peer to be added later.
    let new_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/7070").unwrap()];
    let new_peer_id = PeerId::random();

    // Setup discovery.
    let (_, mut conn_mgr_reqs_rx, mut network_notifs_tx, _, _) =
        setup_discovery(&mut rt, self_peer_id, self_addrs.clone());

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Send a message from other peer containing their discovery note.
        let other_note = Note::new(
            other_peer_id,
            other_addrs.clone(),
            b"example.com",
            100, /* epoch */
        );
        let msg = GossipDiscoveryMsg {
            notes: vec![other_note],
        };
        let msg_key = (other_peer_id, ProtocolId::DiscoveryDirectSend);
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                msg_key,
                PeerManagerNotification::RecvMessage(other_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Connectivity manager receives address of other peer.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            [
                (other_peer_id, other_addrs),
                (self_peer_id, self_addrs.clone()),
            ]
            .iter()
            .cloned()
            .collect(),
        )
        .await;

        // Send a message from other peer containing their updated discovery note
        // and another peer's new note.
        let new_note = Note::new(
            new_peer_id,
            new_addrs.clone(),
            b"example.com",
            200, /* epoch */
        );

        // Update other peer's note.
        let other_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap()];
        let other_note = Note::new(
            other_peer_id,
            other_addrs.clone(),
            b"example.com",
            300, /* epoch */
        );

        let msg = GossipDiscoveryMsg {
            notes: vec![new_note, other_note],
        };
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                msg_key,
                PeerManagerNotification::RecvMessage(other_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Connectivity manager receives new addresses.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            [
                (new_peer_id, new_addrs),
                (other_peer_id, other_addrs),
                (self_peer_id, self_addrs),
            ]
            .iter()
            .cloned()
            .collect(),
        )
        .await;
    };
    rt.block_on(f_network);
}

#[test]
// Test that discovery actor sends a DiscoveryMsg to a neighbor on receiving a clock tick.
fn outbound() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self peer.
    let peer_id = PeerId::random();
    let addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];

    // Setup other peer.
    let other_peer_id = PeerId::random();
    let other_peer_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8080").unwrap();

    // Setup discovery.
    let (
        mut network_reqs_rx,
        _conn_mgr_req_rx,
        _network_notifs_tx,
        mut connection_notifs_tx,
        mut ticker_tx,
    ) = setup_discovery(&mut rt, peer_id, addrs.clone());

    // Fake connectivity manager and dialer.
    let f_network = async move {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        // Notify discovery actor of connection to other peer.
        send_new_peer_with_feedback(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_addr.clone(),
            delivered_tx,
        );
        delivered_rx.await.unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Check request sent as message over network.
        match network_reqs_rx.select_next_some().await {
            PeerManagerRequest::SendMessage(peer, raw_msg) => {
                assert_eq!(peer, other_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the other peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(peer_id, msg.notes[0].peer_id);
                assert_eq!(&addrs, msg.notes[0].addrs());
            }
            req => {
                panic!("Unexpected request to peer manager: {:?}", req);
            }
        }
    };

    rt.block_on(f_network);
}

#[test]
fn old_note_higher_epoch() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self peer.
    let peer_id = PeerId::random();
    let addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];

    // Setup other peer.
    let other_peer_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();

    // Setup discovery.
    let (mut network_reqs_rx, _, mut network_notifs_tx, mut connection_notifs_tx, mut ticker_tx) =
        setup_discovery(&mut rt, peer_id, addrs);

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Notify discovery actor of connection to other peer.
        let (delivered_tx, delivered_rx) = oneshot::channel();
        send_new_peer_with_feedback(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_addrs[0].clone(),
            delivered_tx,
        );
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has higher epoch than
        // current note.
        let old_self_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = get_unix_epoch() + 1_000_000;
        let old_note = Note::new(peer_id, old_self_addrs.clone(), b"example.com", old_epoch);
        let msg = GossipDiscoveryMsg {
            notes: vec![old_note],
        };
        let msg_key = (other_peer_id, ProtocolId::DiscoveryDirectSend);
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                msg_key,
                PeerManagerNotification::RecvMessage(other_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Check request sent as message over network.
        match network_reqs_rx.select_next_some().await {
            PeerManagerRequest::SendMessage(peer, raw_msg) => {
                assert_eq!(peer, other_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the other peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(peer_id, msg.notes[0].peer_id);
                assert!(msg.notes[0].epoch() > old_epoch);
            }
            req => {
                panic!("Unexpected request to peer manager: {:?}", req);
            }
        }
    };
    rt.block_on(f_network);
}

#[test]
fn old_note_max_epoch() {
    ::libra_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];

    // Setup other.
    let other_peer_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();

    // Setup discovery.
    let (mut network_reqs_rx, _, mut network_notifs_tx, mut connection_notifs_tx, mut ticker_tx) =
        setup_discovery(&mut rt, peer_id, addrs);

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Notify discovery actor of connection to other peer.
        let (delivered_tx, delivered_rx) = oneshot::channel();
        send_new_peer_with_feedback(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_addrs[0].clone(),
            delivered_tx,
        );
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has u64::MAX epoch.
        let old_self_addrs = vec![NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = std::u64::MAX;
        let old_note = Note::new(peer_id, old_self_addrs.clone(), b"example.com", old_epoch);
        let msg = GossipDiscoveryMsg {
            notes: vec![old_note],
        };
        let msg_key = (other_peer_id, ProtocolId::DiscoveryDirectSend);
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                msg_key,
                PeerManagerNotification::RecvMessage(other_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Check request sent as message over network.
        match network_reqs_rx.select_next_some().await {
            PeerManagerRequest::SendMessage(peer, raw_msg) => {
                assert_eq!(peer, other_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the other peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(peer_id, msg.notes[0].peer_id);
                assert!(msg.notes[0].epoch() < old_epoch);
            }
            req => {
                panic!("Unexpected request to peer manager: {:?}", req);
            }
        }
    };
    rt.block_on(f_network);
}
