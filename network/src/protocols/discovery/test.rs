// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::peer_manager::{self, conn_status_channel, PeerManagerNotification, PeerManagerRequest};
use crate::proto::DiscoveryMsg;
use crate::protocols::direct_send::Message;
use crate::validator_network::DISCOVERY_DIRECT_SEND_PROTOCOL;
use crate::ProtocolId;
use channel::libra_channel;
use channel::message_queues::QueueStyle;
use core::str::FromStr;
use futures::channel::oneshot;
use libra_config::config::RoleType;
use libra_crypto::{test_utils::TEST_SEED, *};
use prost::Message as _;
use rand::{rngs::StdRng, SeedableRng};
use std::num::NonZeroUsize;
use tokio::runtime::Runtime;

fn gen_peer_info() -> PeerInfo {
    let mut peer_info = PeerInfo::default();
    peer_info.epoch = 1;
    peer_info.addrs.push(
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090")
            .unwrap()
            .as_ref()
            .into(),
    );
    peer_info
}

fn get_raw_message(msg: DiscoveryMsg) -> Message {
    Message {
        protocol: ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
        mdata: msg.to_bytes().unwrap(),
    }
}

fn parse_raw_message(msg: Message) -> Result<DiscoveryMsg, NetworkError> {
    assert_eq!(msg.protocol, DISCOVERY_DIRECT_SEND_PROTOCOL);
    let msg = DiscoveryMsg::decode(msg.mdata.as_ref())?;
    Ok(msg)
}

fn get_addrs_from_note(note: &Note) -> Vec<Multiaddr> {
    let signed_peer_info = note.signed_peer_info.as_ref().unwrap();
    let peer_info = PeerInfo::decode(signed_peer_info.peer_info.as_ref()).unwrap();
    let mut addrs = vec![];
    for addr in peer_info.addrs {
        addrs.push(Multiaddr::try_from(addr.clone()).unwrap());
    }
    addrs
}

fn get_epoch_from_note(note: &Note) -> u64 {
    let signed_peer_info = note.signed_peer_info.as_ref().unwrap();
    let peer_info = PeerInfo::decode(signed_peer_info.peer_info.as_ref()).unwrap();
    peer_info.epoch
}

fn get_addrs_from_info(peer_info: &PeerInfo) -> Vec<Multiaddr> {
    peer_info
        .addrs
        .iter()
        .map(|addr| Multiaddr::try_from(addr.clone()).unwrap())
        .collect()
}

fn setup_discovery(
    rt: &mut Runtime,
    peer_id: PeerId,
    addrs: Vec<Multiaddr>,
    seed_peer_id: PeerId,
    seed_peer_info: PeerInfo,
    signer: Signer,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    channel::Receiver<ConnectivityRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_status_channel::Sender,
    channel::Sender<()>,
) {
    let (network_reqs_tx, network_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(1);
    let (network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (control_notifs_tx, control_notifs_rx) = conn_status_channel::new();
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let role = RoleType::Validator;
    let discovery = {
        Discovery::new(
            peer_id,
            role,
            addrs,
            signer,
            vec![(seed_peer_id, seed_peer_info)].into_iter().collect(),
            trusted_peers,
            ticker_rx,
            DiscoveryNetworkSender::new(network_reqs_tx),
            DiscoveryNetworkEvents::new(network_notifs_rx, control_notifs_rx),
            conn_mgr_reqs_tx,
        )
    };
    rt.spawn(discovery.start());
    (
        network_reqs_rx,
        conn_mgr_reqs_rx,
        network_notifs_tx,
        control_notifs_tx,
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup seed.
    let seed_peer_info = gen_peer_info();
    let seed_peer_addrs = get_addrs_from_info(&seed_peer_info);
    let seed_peer_id = PeerId::random();
    let (seed_pub_keys, seed_signer) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (_, mut conn_mgr_reqs_rx, mut network_notifs_tx, _, _) = setup_discovery(
        &mut rt,
        peer_id,
        addrs,
        seed_peer_id,
        seed_peer_info,
        self_signer,
        trusted_peers.clone(),
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Connectivity manager receives addresses of the seed peer during bootstrap.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, &seed_peer_addrs[..]).await;

        // Send DiscoveryMsg consisting of 2 notes to the discovery actor - one note for the
        // seed peer and one for another peer. The discovery actor should send addresses of the new
        // peer to the connectivity manager.
        let peer_id_other = PeerId::random();
        let addrs_other = vec![Multiaddr::from_str("/ip4/172.29.52.192/tcp/8080").unwrap()];
        let seed_note = create_note(
            &seed_signer,
            seed_peer_id,
            seed_peer_addrs.clone(),
            b"example.com",
            get_unix_epoch(),
        );
        let (pub_keys_other, signer_other) = generate_network_pub_keys_and_signer(peer_id_other);
        trusted_peers
            .write()
            .unwrap()
            .insert(peer_id_other, pub_keys_other);
        let note_other = {
            create_note(
                &signer_other,
                peer_id_other,
                addrs_other.clone(),
                b"example.com",
                get_unix_epoch(),
            )
        };
        let mut msg = DiscoveryMsg::default();
        msg.notes.push(note_other.clone());
        msg.notes.push(seed_note.clone());
        let key = (
            seed_peer_id,
            ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
        );
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                key.clone(),
                PeerManagerNotification::RecvMessage(seed_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Connectivity manager receives address of new peer.
        expect_address_update(&mut conn_mgr_reqs_rx, peer_id_other, &addrs_other[..]).await;

        // The addrs sent to connectivity manager should also include the
        // configured seed peer addrs for seed-peer-received notes.
        let mut expected_seed_addrs = seed_peer_addrs.clone();
        expected_seed_addrs.extend_from_slice(&seed_peer_addrs[..]);

        // Connectivity manager receives a connect to the seed peer at the same address.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            seed_peer_id,
            &expected_seed_addrs[..],
        )
        .await;

        // Send new message.
        // Compose new msg.
        let mut msg = DiscoveryMsg::default();
        msg.notes.push(note_other);
        let new_seed_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/8098").unwrap()];
        {
            let seed_note = create_note(
                &seed_signer,
                seed_peer_id,
                new_seed_addrs.clone(),
                b"example.com",
                get_unix_epoch(),
            );
            msg.notes.push(seed_note);
            let key = (
                seed_peer_id,
                ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
            );
            let (delivered_tx, delivered_rx) = oneshot::channel();
            network_notifs_tx
                .push_with_feedback(
                    key.clone(),
                    PeerManagerNotification::RecvMessage(peer_id_other, get_raw_message(msg)),
                    Some(delivered_tx),
                )
                .unwrap();
            delivered_rx.await.unwrap();
        }

        // The addrs sent to connectivity manager should also include the
        // configured seed peer addrs for seed-peer-received notes.
        let mut expected_seed_addrs = new_seed_addrs.clone();
        expected_seed_addrs.extend_from_slice(&seed_peer_addrs[..]);

        // Connectivity manager receives new address of seed peer.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            seed_peer_id,
            &expected_seed_addrs[..],
        )
        .await;
    };
    rt.block_on(f_network);
}

#[test]
// Test that discovery actor sends a DiscoveryMsg to a neighbor on receiving a clock tick.
fn outbound() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup seed.
    let seed_peer_id = PeerId::random();
    let seed_peer_info = gen_peer_info();
    let (seed_pub_keys, _) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (
        mut network_reqs_rx,
        _conn_mgr_req_rx,
        _network_notifs_tx,
        mut control_notifs_tx,
        mut ticker_tx,
    ) = setup_discovery(
        &mut rt,
        peer_id,
        addrs.clone(),
        seed_peer_id,
        seed_peer_info.clone(),
        self_signer,
        trusted_peers,
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        // Notify discovery actor of connection to seed peer.
        control_notifs_tx
            .push_with_feedback(
                seed_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    seed_peer_id,
                    Multiaddr::try_from(seed_peer_info.addrs.get(0).unwrap().clone()).unwrap(),
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
                assert_eq!(peer, seed_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the seed peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(Vec::from(peer_id), msg.notes[0].peer_id);
                assert_eq!(addrs, get_addrs_from_note(&msg.notes[0]));
            }
            req => {
                panic!("Unexpected request to peer manager: {:?}", req);
            }
        }
    };

    rt.block_on(f_network);
}

#[test]
fn addr_update_includes_seed_addrs() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup seed.
    let seed_peer_info = gen_peer_info();
    let mut seed_peer_addrs = get_addrs_from_info(&seed_peer_info);
    let seed_peer_id = PeerId::random();
    let (seed_pub_keys, seed_signer) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (_, mut conn_mgr_reqs_rx, mut network_notifs_tx, _, _) = setup_discovery(
        &mut rt,
        peer_id,
        addrs,
        seed_peer_id,
        seed_peer_info,
        self_signer,
        trusted_peers,
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Connectivity manager receives addresses of the seed peer during bootstrap.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, &seed_peer_addrs[..]).await;

        // Send DiscoveryMsg consisting of the new seed peer's discovery note.
        // The discovery actor should send the addrs in the new seed peer note
        // _and_ the configured seed addrs to the connectivity manager.
        let new_seed_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let seed_note = create_note(
            &seed_signer,
            seed_peer_id,
            new_seed_addrs.clone(),
            b"example.com",
            get_unix_epoch(),
        );
        let mut msg = DiscoveryMsg::default();
        msg.notes.push(seed_note.clone());
        let key = (
            seed_peer_id,
            ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
        );
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                key.clone(),
                PeerManagerNotification::RecvMessage(seed_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // The addrs sent to connectivity manager should also include the
        // configured seed peer addrs for seed-peer-received notes.
        let mut expected_seed_addrs = new_seed_addrs.clone();
        expected_seed_addrs.append(&mut seed_peer_addrs);

        // Connectivity manager receives an update including the seed peer's new
        // note addrs and the original configured seed addrs.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            seed_peer_id,
            &expected_seed_addrs[..],
        )
        .await;
    };
    rt.block_on(f_network);
}

#[test]
fn old_note_higher_epoch() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup seed.
    let seed_peer_info = gen_peer_info();
    let seed_peer_addrs = get_addrs_from_info(&seed_peer_info);
    let seed_peer_id = PeerId::random();
    let (seed_pub_keys, _) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (
        mut network_reqs_rx,
        mut conn_mgr_reqs_rx,
        mut network_notifs_tx,
        mut control_notifs_tx,
        mut ticker_tx,
    ) = setup_discovery(
        &mut rt,
        peer_id,
        addrs,
        seed_peer_id,
        seed_peer_info,
        self_signer.clone(),
        trusted_peers,
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Connectivity manager receives addresses of the seed peer during bootstrap.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, &seed_peer_addrs[..]).await;

        let (delivered_tx, delivered_rx) = oneshot::channel();
        // Notify discovery actor of connection to seed peer.
        control_notifs_tx
            .push_with_feedback(
                seed_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    seed_peer_id,
                    Multiaddr::try_from(seed_peer_addrs.get(0).unwrap().clone()).unwrap(),
                ),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has higher epoch than
        // current note.
        let old_self_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = get_unix_epoch() + 100;
        let old_note = create_note(
            &self_signer,
            peer_id,
            old_self_addrs.clone(),
            b"example.com",
            old_epoch,
        );
        let mut msg = DiscoveryMsg::default();
        msg.notes.push(old_note.clone());
        let key = (
            seed_peer_id,
            ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
        );
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                key.clone(),
                PeerManagerNotification::RecvMessage(seed_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Check request sent as message over network.
        match network_reqs_rx.select_next_some().await {
            PeerManagerRequest::SendMessage(peer, raw_msg) => {
                assert_eq!(peer, seed_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the seed peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(Vec::from(peer_id), msg.notes[0].peer_id);
                assert!(get_epoch_from_note(&msg.notes[0]) > old_epoch);
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();

    // Setup self.
    let peer_id = PeerId::random();
    let addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap()];
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);

    // Setup seed.
    let seed_peer_info = gen_peer_info();
    let seed_peer_addrs = get_addrs_from_info(&seed_peer_info);
    let seed_peer_id = PeerId::random();
    let (seed_pub_keys, _) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));

    // Setup discovery.
    let (
        mut network_reqs_rx,
        mut conn_mgr_reqs_rx,
        mut network_notifs_tx,
        mut control_notifs_tx,
        mut ticker_tx,
    ) = setup_discovery(
        &mut rt,
        peer_id,
        addrs,
        seed_peer_id,
        seed_peer_info,
        self_signer.clone(),
        trusted_peers,
    );

    // Fake connectivity manager and dialer.
    let f_network = async move {
        // Connectivity manager receives addresses of the seed peer during bootstrap.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, &seed_peer_addrs[..]).await;

        let (delivered_tx, delivered_rx) = oneshot::channel();
        // Notify discovery actor of connection to seed peer.
        control_notifs_tx
            .push_with_feedback(
                seed_peer_id,
                peer_manager::ConnectionStatusNotification::NewPeer(
                    seed_peer_id,
                    Multiaddr::try_from(seed_peer_addrs.get(0).unwrap().clone()).unwrap(),
                ),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Send DiscoveryMsg consisting of the this node's older note which has u64::MAX epoch.
        let old_self_addrs = vec![Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap()];
        let old_epoch = std::u64::MAX;
        let old_note = create_note(
            &self_signer,
            peer_id,
            old_self_addrs.clone(),
            b"example.com",
            old_epoch,
        );
        let mut msg = DiscoveryMsg::default();
        msg.notes.push(old_note.clone());
        let key = (
            seed_peer_id,
            ProtocolId::from_static(DISCOVERY_DIRECT_SEND_PROTOCOL),
        );
        let (delivered_tx, delivered_rx) = oneshot::channel();
        network_notifs_tx
            .push_with_feedback(
                key.clone(),
                PeerManagerNotification::RecvMessage(seed_peer_id, get_raw_message(msg)),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Check request sent as message over network.
        match network_reqs_rx.select_next_some().await {
            PeerManagerRequest::SendMessage(peer, raw_msg) => {
                assert_eq!(peer, seed_peer_id);
                let msg = parse_raw_message(raw_msg).unwrap();
                // Receive DiscoveryMsg from actor. The message should contain only a note for the
                // sending peer since it doesn't yet have the note for the seed peer.
                assert_eq!(1, msg.notes.len());
                assert_eq!(Vec::from(peer_id), msg.notes[0].peer_id);
                assert!(get_epoch_from_note(&msg.notes[0]) < old_epoch);
            }
            req => {
                panic!("Unexpected request to peer manager: {:?}", req);
            }
        }
    };
    rt.block_on(f_network);
}
