// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer_manager::{
        self, conn_status_channel, ConnectionRequestSender, PeerManagerNotification,
        PeerManagerRequest,
    },
    protocols::direct_send::Message,
    ProtocolId,
};
use anyhow::anyhow;
use channel::{libra_channel, message_queues::QueueStyle};
use core::str::FromStr;
use futures::channel::oneshot;
use libra_config::config::RoleType;
use libra_crypto::{test_utils::TEST_SEED, *};
use rand::{rngs::StdRng, SeedableRng};
use std::num::NonZeroUsize;
use tokio::runtime::Runtime;

fn gen_peer_info() -> PeerInfo {
    PeerInfo {
        addrs: vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()],
        epoch: 1,
    }
}

fn get_raw_message(msg: DiscoveryMsg) -> Message {
    Message {
        protocol: ProtocolId::DiscoveryDirectSend,
        mdata: lcs::to_bytes(&msg).unwrap().into(),
    }
}

fn parse_raw_message(msg: Message) -> Result<DiscoveryMsg, NetworkError> {
    assert_eq!(msg.protocol, ProtocolId::DiscoveryDirectSend);
    let msg: DiscoveryMsg = lcs::from_bytes(&msg.mdata)
        .map_err(|err| anyhow!(err).context(NetworkErrorKind::ParsingError))?;
    Ok(msg)
}

fn setup_discovery(
    rt: &mut Runtime,
    peer_id: PeerId,
    addrs: Vec<Multiaddr>,
    signer: Signer,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    channel::Receiver<ConnectivityRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_status_channel::Sender,
    channel::Sender<()>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_reqs_tx, _) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(1);
    let (network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_notifs_tx, connection_notifs_rx) = conn_status_channel::new();
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let role = RoleType::Validator;
    let discovery = {
        Discovery::new(
            peer_id,
            role,
            addrs,
            signer,
            trusted_peers,
            ticker_rx,
            DiscoveryNetworkSender::new(
                PeerManagerRequestSender::new(peer_mgr_reqs_tx),
                ConnectionRequestSender::new(connection_reqs_tx),
            ),
            DiscoveryNetworkEvents::new(network_notifs_rx, connection_notifs_rx),
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

async fn expect_address_update(
    conn_mgr_reqs_rx: &mut channel::Receiver<ConnectivityRequest>,
    expected_peer_id: PeerId,
    expected_addrs: &[Multiaddr],
) {
    match conn_mgr_reqs_rx.next().await.unwrap() {
        ConnectivityRequest::UpdateAddresses(peer_id, addrs) => {
            assert_eq!(expected_peer_id, peer_id);
            assert_eq!(expected_addrs, &addrs[..]);
        }
        req => {
            panic!("Unexpected request to connectivity manager: {:?}", req);
        }
    }
}

fn generate_network_pub_keys_and_signer(peer_id: PeerId) -> (NetworkPublicKeys, Signer) {
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (signing_priv_key, _) = compat::generate_keypair(&mut rng);
    let (_, identity_pub_key) = x25519::compat::generate_keypair(&mut rng);
    (
        NetworkPublicKeys {
            signing_public_key: signing_priv_key.public_key(),
            identity_public_key: identity_pub_key,
        },
        Signer::new(peer_id, signing_priv_key),
    )
}

#[test]
// Test behavior on receipt of an inbound DiscoveryMsg.
fn inbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let self_peer_id = PeerId::random();
    let self_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(self_peer_id);

    // Setup other peer.
    let other_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();
    let (other_pub_keys, other_signer) = generate_network_pub_keys_and_signer(other_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![
            (other_peer_id, other_pub_keys),
            (self_peer_id, self_pub_keys),
        ]
        .into_iter()
        .collect(),
    ));

    // Setup new peer to be added later.
    let new_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/7070").unwrap()];
    let new_peer_id = PeerId::random();
    let (new_pub_keys, new_signer) = generate_network_pub_keys_and_signer(new_peer_id);

    // Setup discovery.
    let (_, mut conn_mgr_reqs_rx, mut network_notifs_tx, _, _) = setup_discovery(
        &mut rt,
        self_peer_id,
        self_addrs,
        self_signer,
        trusted_peers.clone(),
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Send a message from other peer containing their discovery note.
        let other_note = Note::new(
            &other_signer,
            other_peer_id,
            other_addrs.clone(),
            b"example.com",
            get_unix_epoch(),
        );
        let msg = DiscoveryMsg {
            notes: vec![other_note],
        };
        let msg_key = (other_peer_id, ProtocolId::DiscoveryDirectSend);
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                msg_key.clone(),
                PeerManagerNotification::RecvMessage(other_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Connectivity manager receives address of other peer.
        expect_address_update(&mut conn_mgr_reqs_rx, other_peer_id, &other_addrs[..]).await;

        // Send a message from other peer containing their updated discovery note
        // and another peer's new note.

        // Add the new peer's pubkey to the trusted peers set.
        trusted_peers
            .write()
            .unwrap()
            .insert(new_peer_id, new_pub_keys);

        let new_note = Note::new(
            &new_signer,
            new_peer_id,
            new_addrs.clone(),
            b"example.com",
            get_unix_epoch(),
        );

        // Update other peer's note.
        let other_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap()];
        let other_note = Note::new(
            &other_signer,
            other_peer_id,
            other_addrs.clone(),
            b"example.com",
            get_unix_epoch(),
        );

        let msg = DiscoveryMsg {
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

        // Connectivity manager receives address of other peer.
        expect_address_update(&mut conn_mgr_reqs_rx, new_peer_id, &new_addrs[..]).await;
        expect_address_update(&mut conn_mgr_reqs_rx, other_peer_id, &other_addrs[..]).await;
    };
    rt.block_on(f_network);
}

#[test]
// Test that discovery actor sends a DiscoveryMsg to a neighbor on receiving a clock tick.
fn outbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();

    // Setup self peer.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup other peer.
    let other_peer_id = PeerId::random();
    let other_peer_info = gen_peer_info();
    let (other_pub_keys, _) = generate_network_pub_keys_and_signer(other_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(other_peer_id, other_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (
        mut network_reqs_rx,
        _conn_mgr_req_rx,
        _network_notifs_tx,
        mut connection_notifs_tx,
        mut ticker_tx,
    ) = setup_discovery(&mut rt, peer_id, addrs.clone(), self_signer, trusted_peers);

    // Fake connectivity manager and dialer.
    let f_network = async move {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        // Notify discovery actor of connection to other peer.
        connection_notifs_tx
            .push_with_feedback(
                other_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    other_peer_id,
                    other_peer_info.addrs[0].clone(),
                ),
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
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();

    // Setup self peer.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup other peer.
    let other_peer_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();
    let (other_pub_keys, _) = generate_network_pub_keys_and_signer(other_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(other_peer_id, other_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (mut network_reqs_rx, _, mut network_notifs_tx, mut connection_notifs_tx, mut ticker_tx) =
        setup_discovery(&mut rt, peer_id, addrs, self_signer.clone(), trusted_peers);

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Notify discovery actor of connection to other peer.
        let (delivered_tx, delivered_rx) = oneshot::channel();
        connection_notifs_tx
            .push_with_feedback(
                other_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    other_peer_id,
                    other_peer_addrs[0].clone(),
                ),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has higher epoch than
        // current note.
        let old_self_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = get_unix_epoch() + 100;
        let old_note = Note::new(
            &self_signer,
            peer_id,
            old_self_addrs.clone(),
            b"example.com",
            old_epoch,
        );
        let msg = DiscoveryMsg {
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
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup other.
    let other_peer_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap()];
    let other_peer_id = PeerId::random();
    let (other_pub_keys, _) = generate_network_pub_keys_and_signer(other_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(other_peer_id, other_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (mut network_reqs_rx, _, mut network_notifs_tx, mut connection_notifs_tx, mut ticker_tx) =
        setup_discovery(&mut rt, peer_id, addrs, self_signer.clone(), trusted_peers);

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Notify discovery actor of connection to other peer.
        let (delivered_tx, delivered_rx) = oneshot::channel();
        connection_notifs_tx
            .push_with_feedback(
                other_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    other_peer_id,
                    other_peer_addrs[0].clone(),
                ),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has u64::MAX epoch.
        let old_self_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = std::u64::MAX;
        let old_note = Note::new(
            &self_signer,
            peer_id,
            old_self_addrs.clone(),
            b"example.com",
            old_epoch,
        );
        let msg = DiscoveryMsg {
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
