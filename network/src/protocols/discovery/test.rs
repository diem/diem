// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{peer_manager::PeerManagerRequest, proto::DiscoveryMsg};
use core::str::FromStr;
use crypto::{signing, x25519};
use futures::future::{FutureExt, TryFutureExt};
use memsocket::MemorySocket;
use tokio::runtime::Runtime;

fn get_random_seed() -> PeerInfo {
    let mut peer_info = PeerInfo::new();
    peer_info.set_epoch(1);
    peer_info.mut_addrs().push(
        Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090")
            .unwrap()
            .as_ref()
            .into(),
    );
    peer_info
}

fn setup_discovery(
    rt: &mut Runtime,
    peer_id: PeerId,
    address: Multiaddr,
    seed_peer_id: PeerId,
    seed_peer_info: PeerInfo,
    signer: Signer,
    trusted_peers: Arc<RwLock<HashMap<PeerId, NetworkPublicKeys>>>,
) -> (
    channel::Receiver<PeerManagerRequest<MemorySocket>>,
    channel::Receiver<ConnectivityRequest>,
    channel::Sender<PeerManagerNotification<MemorySocket>>,
    channel::Sender<()>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = channel::new_test(0);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(1);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = channel::new_test(0);
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let discovery = {
        Discovery::new(
            peer_id,
            vec![address],
            signer,
            vec![(seed_peer_id, seed_peer_info)].into_iter().collect(),
            trusted_peers,
            ticker_rx,
            PeerManagerRequestSender::new(peer_mgr_reqs_tx),
            peer_mgr_notifs_rx,
            conn_mgr_reqs_tx,
            Duration::from_secs(180),
        )
    };
    rt.spawn(discovery.start().boxed().unit_error().compat());
    (
        peer_mgr_reqs_rx,
        conn_mgr_reqs_rx,
        peer_mgr_notifs_tx,
        ticker_tx,
    )
}

fn get_addrs(note: &Note) -> Vec<Multiaddr> {
    let peer_info: PeerInfo = protobuf::parse_from_bytes(note.get_peer_info()).unwrap();
    let mut addrs = vec![];
    for addr in peer_info.get_addrs() {
        addrs.push(Multiaddr::try_from(addr.clone()).unwrap());
    }
    addrs
}

async fn expect_address_update(
    conn_mgr_reqs_rx: &mut channel::Receiver<ConnectivityRequest>,
    peer_id: PeerId,
    addr: Multiaddr,
) {
    match conn_mgr_reqs_rx.next().await.unwrap() {
        ConnectivityRequest::UpdateAddresses(p, addrs) => {
            assert_eq!(peer_id, p);
            assert_eq!(1, addrs.len());
            assert_eq!(addr, addrs[0]);
        }
        _ => {
            panic!("unexpected request to connectivity manager");
        }
    }
}

fn generate_network_pub_keys_and_signer(peer_id: PeerId) -> (NetworkPublicKeys, Signer) {
    let (signing_priv_key, signing_pub_key) = signing::generate_keypair();
    let (_, identity_pub_key) = x25519::generate_keypair();
    (
        NetworkPublicKeys {
            signing_public_key: signing_pub_key,
            identity_public_key: identity_pub_key,
        },
        Signer::new(peer_id, signing_pub_key, signing_priv_key),
    )
}

#[test]
// Test behavior on receipt of an inbound DiscoveryMsg.
fn inbound() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    // Setup self.
    let peer_id = PeerId::random();
    let address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);
    // Setup seed.
    let mut seed_peer_info = get_random_seed();
    let seed_peer_id = PeerId::random();
    let (seed_pub_keys, seed_signer) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));
    // Setup discovery.
    let (_, mut conn_mgr_reqs_rx, mut peer_mgr_notifs_tx, _) = setup_discovery(
        &mut rt,
        peer_id,
        address.clone(),
        seed_peer_id,
        seed_peer_info.clone(),
        self_signer,
        trusted_peers.clone(),
    );

    // Fake connectivity manager and dialer.
    let f_peer_mgr = async move {
        let seed_peer_address = Multiaddr::try_from(seed_peer_info.get_addrs()[0].clone()).unwrap();
        // Connectivity manager receives addresses of the seed peer during bootstrap.
        expect_address_update(
            &mut conn_mgr_reqs_rx,
            seed_peer_id,
            seed_peer_address.clone(),
        )
        .await;

        let (dialer_substream, listener_substream) = MemorySocket::new_pair();
        // Notify discovery actor of inbound substream.

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewInboundSubstream(
                seed_peer_id,
                NegotiatedSubstream {
                    protocol: ProtocolId::from_static(DISCOVERY_PROTOCOL_NAME),
                    substream: listener_substream,
                },
            ))
            .await
            .unwrap();
        // Wrap dialer substream in a framed substream.
        let mut dialer_substream =
            Framed::new(dialer_substream.compat(), UviBytes::<Bytes>::default()).sink_compat();

        // Send DiscoveryMsg consisting of 2 notes to the discovery actor - one note for the
        // seed peer and one for another peer. The discovery actor should send addresses of the new
        // peer to the connectivity manager.
        let peer_id_other = PeerId::random();
        let address_other = Multiaddr::from_str("/ip4/172.29.52.192/tcp/8080").unwrap();
        let seed_note = create_note(&seed_signer, seed_peer_id, seed_peer_info.clone());
        let (pub_keys_other, signer_other) = generate_network_pub_keys_and_signer(peer_id_other);
        trusted_peers
            .write()
            .unwrap()
            .insert(peer_id_other, pub_keys_other);
        let note_other = {
            let mut peer_info = PeerInfo::new();
            let addrs = peer_info.mut_addrs();
            addrs.clear();
            addrs.push(address_other.as_ref().into());
            create_note(&signer_other, peer_id_other, peer_info)
        };
        let mut msg = DiscoveryMsg::new();
        msg.mut_notes().push(note_other.clone());
        msg.mut_notes().push(seed_note.clone());
        dialer_substream
            .send(msg.write_to_bytes().unwrap().into())
            .await
            .unwrap();

        // Connectivity manager receives address of new peer.
        expect_address_update(&mut conn_mgr_reqs_rx, peer_id_other, address_other).await;

        // Connectivity manager receives a connect to the seed peer at the same address.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, seed_peer_address).await;

        let (dialer_substream, listener_substream) = MemorySocket::new_pair();
        // Notify discovery actor of inbound substream.

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewInboundSubstream(
                peer_id_other,
                NegotiatedSubstream {
                    protocol: ProtocolId::from_static(DISCOVERY_PROTOCOL_NAME),
                    substream: listener_substream,
                },
            ))
            .await
            .unwrap();
        // Wrap dialer substream in a framed substream.
        let mut dialer_substream =
            Framed::new(dialer_substream.compat(), UviBytes::<Bytes>::default()).sink_compat();
        // Compose new msg.
        let mut msg = DiscoveryMsg::new();
        msg.mut_notes().push(note_other);
        let new_seed_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8098").unwrap();
        {
            seed_peer_info.set_epoch(3000);
            seed_peer_info.mut_addrs().clear();
            seed_peer_info
                .mut_addrs()
                .push(new_seed_addr.as_ref().into());
            let seed_note = create_note(&seed_signer, seed_peer_id, seed_peer_info);
            msg.mut_notes().push(seed_note);
            dialer_substream
                .send(msg.write_to_bytes().unwrap().into())
                .await
                .unwrap();
        }

        // Connectivity manager receives new address of seed peer.
        expect_address_update(&mut conn_mgr_reqs_rx, seed_peer_id, new_seed_addr).await;
    };
    rt.block_on(f_peer_mgr.boxed().unit_error().compat())
        .unwrap();
}

#[test]
// Test that discovery actor sends a DiscoveryMsg to a neighbor on receiving a clock tick.
fn outbound() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    // Setup self.
    let peer_id = PeerId::random();
    let address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let (self_pub_keys, self_signer) = generate_network_pub_keys_and_signer(peer_id);
    // Setup seed.
    let seed_peer_id = PeerId::random();
    let seed_peer_info = get_random_seed();
    let (seed_pub_keys, _) = generate_network_pub_keys_and_signer(seed_peer_id);
    let trusted_peers = Arc::new(RwLock::new(
        vec![(seed_peer_id, seed_pub_keys), (peer_id, self_pub_keys)]
            .into_iter()
            .collect(),
    ));
    // Setup discovery.
    let (mut peer_mgr_reqs_rx, _conn_mgr_req_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_discovery(
            &mut rt,
            peer_id,
            address.clone(),
            seed_peer_id,
            seed_peer_info.clone(),
            self_signer,
            trusted_peers.clone(),
        );

    // Fake connectivity manager and dialer.
    let f_peer_mgr = async move {
        let (dialer_substream, listener_substream) = MemorySocket::new_pair();
        let seed_peer_address = Multiaddr::try_from(seed_peer_info.get_addrs()[0].clone()).unwrap();
        // Notify discovery actor of connection to seed peer.
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                seed_peer_id,
                seed_peer_address,
            ))
            .await
            .unwrap();

        // Trigger outbound msg.
        ticker_tx.send(()).await.unwrap();

        // Request outgoing substream from PeerManager.
        match peer_mgr_reqs_rx.next().await.unwrap() {
            PeerManagerRequest::OpenSubstream(peer, protocol, ch) => {
                assert_eq!(peer, seed_peer_id);
                assert_eq!(protocol, DISCOVERY_PROTOCOL_NAME);
                ch.send(Ok(dialer_substream)).unwrap();
            }
            _ => {
                panic!("unexpected request to peer manager");
            }
        }

        // Receive DiscoveryMsg from actor. The message should contain only a note for the
        // sending peer since it doesn't yet have the note for the seed peer.
        let msg = recv_msg(listener_substream).await.unwrap();
        assert_eq!(1, msg.get_notes().len());
        assert_eq!(Vec::from(peer_id), msg.get_notes()[0].get_peer_id());
        assert_eq!(address, get_addrs(&msg.get_notes()[0])[0]);
    };

    rt.block_on(f_peer_mgr.boxed().unit_error().compat())
        .unwrap();
}
