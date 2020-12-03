// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer::DisconnectReason,
    peer_manager::{conn_notifs_channel, ConnectionRequest},
};
use channel::{diem_channel, message_queues::QueueStyle};
use core::str::FromStr;
use diem_config::{config::RoleType, network_id::NetworkId};
use diem_crypto::{test_utils::TEST_SEED, x25519, Uniform};
use diem_logger::info;
use diem_network_address::NetworkAddress;
use futures::SinkExt;
use netcore::transport::ConnectionOrigin;
use rand::rngs::StdRng;
use std::{io, num::NonZeroUsize};
use tokio::runtime::Runtime;
use tokio_retry::strategy::FixedInterval;

const MAX_TEST_CONNECTIONS: usize = 3;

fn setup_conn_mgr(
    rt: &mut Runtime,
    eligible_peers: Vec<PeerId>,
    seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
) -> (
    diem_channel::Receiver<PeerId, ConnectionRequest>,
    conn_notifs_channel::Sender,
    channel::Sender<ConnectivityRequest>,
    channel::Sender<()>,
) {
    let network_context =
        NetworkContext::new(NetworkId::Validator, RoleType::Validator, PeerId::random());

    let seed_pubkeys: HashMap<_, _> = eligible_peers
        .into_iter()
        .map(|peer_id| {
            let pubkey = x25519::PrivateKey::generate_for_testing().public_key();
            let pubkeys: HashSet<_> = [pubkey].iter().copied().collect();
            (peer_id, pubkeys)
        })
        .collect();

    setup_conn_mgr_with_context(network_context, rt, seed_addrs, seed_pubkeys)
}

fn setup_conn_mgr_with_context(
    network_context: NetworkContext,
    rt: &mut Runtime,
    seed_addrs: HashMap<PeerId, Vec<NetworkAddress>>,
    seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>>,
) -> (
    diem_channel::Receiver<PeerId, ConnectionRequest>,
    conn_notifs_channel::Sender,
    channel::Sender<ConnectivityRequest>,
    channel::Sender<()>,
) {
    let (connection_reqs_tx, connection_reqs_rx) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(0);
    let (ticker_tx, ticker_rx) = channel::new_test(0);

    let conn_mgr = {
        ConnectivityManager::new(
            Arc::new(network_context),
            Arc::new(RwLock::new(HashMap::new())),
            seed_addrs,
            seed_pubkeys,
            ticker_rx,
            ConnectionRequestSender::new(connection_reqs_tx),
            connection_notifs_rx,
            conn_mgr_reqs_rx,
            FixedInterval::from_millis(100),
            300, /* ms */
            Some(MAX_TEST_CONNECTIONS),
        )
    };
    rt.spawn(conn_mgr.start());
    (
        connection_reqs_rx,
        connection_notifs_tx,
        conn_mgr_reqs_tx,
        ticker_tx,
    )
}

fn gen_peer() -> (
    PeerId,
    x25519::PublicKey,
    HashSet<x25519::PublicKey>,
    NetworkAddress,
) {
    let peer_id = PeerId::random();
    let pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let pubkeys: HashSet<_> = [pubkey].iter().copied().collect();
    let addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let addr = addr.append_prod_protos(pubkey, diem_config::config::HANDSHAKE_VERSION);
    (peer_id, pubkey, pubkeys, addr)
}

async fn get_connected_size(conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>) -> usize {
    let (queue_size_tx, queue_size_rx) = oneshot::channel();
    conn_mgr_reqs_tx
        .send(ConnectivityRequest::GetConnectedSize(queue_size_tx))
        .await
        .unwrap();
    queue_size_rx.await.unwrap()
}

async fn get_dial_queue_size(conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>) -> usize {
    let (queue_size_tx, queue_size_rx) = oneshot::channel();
    conn_mgr_reqs_tx
        .send(ConnectivityRequest::GetDialQueueSize(queue_size_tx))
        .await
        .unwrap();
    queue_size_rx.await.unwrap()
}

async fn send_new_peer_await_delivery(
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    peer_id: PeerId,
    notif_peer_id: PeerId,
    address: NetworkAddress,
) {
    let notif = peer_manager::ConnectionNotification::NewPeer(
        notif_peer_id,
        address,
        ConnectionOrigin::Inbound,
        NetworkContext::mock(),
    );
    send_notification_await_delivery(connection_notifs_tx, peer_id, notif).await;
}

async fn send_lost_peer_await_delivery(
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    peer_id: PeerId,
    notif_peer_id: PeerId,
    address: NetworkAddress,
    reason: DisconnectReason,
) {
    let notif = peer_manager::ConnectionNotification::LostPeer(
        notif_peer_id,
        address,
        ConnectionOrigin::Inbound,
        reason,
    );
    send_notification_await_delivery(connection_notifs_tx, peer_id, notif).await;
}

async fn send_notification_await_delivery(
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    peer_id: PeerId,
    notif: peer_manager::ConnectionNotification,
) {
    let (delivered_tx, delivered_rx) = oneshot::channel();
    connection_notifs_tx
        .push_with_feedback(peer_id, notif, Some(delivered_tx))
        .unwrap();
    delivered_rx.await.unwrap();
}

async fn expect_disconnect_request(
    connection_reqs_rx: &mut diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    peer_id: PeerId,
    address: NetworkAddress,
    result: Result<(), PeerManagerError>,
) {
    let success = result.is_ok();
    match connection_reqs_rx.next().await.unwrap() {
        ConnectionRequest::DisconnectPeer(p, error_tx) => {
            assert_eq!(peer_id, p);
            error_tx.send(result).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
    if success {
        send_lost_peer_await_delivery(
            connection_notifs_tx,
            peer_id,
            peer_id,
            address,
            DisconnectReason::Requested,
        )
        .await;
    }
}

async fn expect_dial_request(
    connection_reqs_rx: &mut diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>,
    peer_id: PeerId,
    address: NetworkAddress,
    result: Result<(), PeerManagerError>,
) {
    let success = result.is_ok();
    match connection_reqs_rx.next().await.unwrap() {
        ConnectionRequest::DialPeer(p, addr, error_tx) => {
            assert_eq!(peer_id, p);
            assert_eq!(address, addr);
            error_tx.send(result).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
    if success {
        info!(
            "Sending NewPeer notification for peer: {}",
            peer_id.short_str()
        );
        send_new_peer_await_delivery(connection_notifs_tx, peer_id, peer_id, address).await;
    }

    // Wait for dial queue to be empty. Without this, it's impossible to guarantee that a completed
    // dial is removed from a dial queue. We need this guarantee to see the effects of future
    // triggers for connectivity check.
    info!("Waiting for dial queue to be empty");
    loop {
        let queue_size = get_dial_queue_size(conn_mgr_reqs_tx).await;
        if queue_size == 0 {
            break;
        }
    }
}

async fn expect_num_dials(
    connection_reqs_rx: &mut diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
    conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>,
    num_expected: usize,
) {
    for _ in 0..num_expected {
        if let ConnectionRequest::DialPeer(peer_id, address, error_tx) =
            connection_reqs_rx.next().await.unwrap()
        {
            error_tx.send(Ok(())).unwrap();

            info!(
                "Sending NewPeer notification for peer: {}",
                peer_id.short_str()
            );
            send_new_peer_await_delivery(connection_notifs_tx, peer_id, peer_id, address).await;
        } else {
            panic!("unexpected request to peer manager");
        }
    }

    // Wait for dial queue to be empty. Without this, it's impossible to guarantee that a completed
    // dial is removed from a dial queue. We need this guarantee to see the effects of future
    // triggers for connectivity check.
    info!("Waiting for dial queue to be empty");
    loop {
        let queue_size = get_dial_queue_size(conn_mgr_reqs_tx).await;
        if queue_size == 0 {
            break;
        }
    }
}

#[test]
fn connect_to_seeds_on_startup() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (seed_peer_id, _, _, seed_addr) = gen_peer();
    let seed_addrs: HashMap<_, _> = vec![(seed_peer_id, vec![seed_addr.clone()])]
        .into_iter()
        .collect();
    let eligible_peers = vec![seed_peer_id];

    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr.clone(),
            Ok(()),
        )
        .await;

        // Sending an UpdateAddresses with the same seed address should not
        // trigger any dials.
        info!("Sending same address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(seed_peer_id, vec![seed_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        let new_seed_addr = NetworkAddress::from_str("/ip4/127.0.1.1/tcp/8080").unwrap();
        // Send new address of seed peer.
        info!("Sending new address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(seed_peer_id, vec![new_seed_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // We expect the peer which changed its address to also disconnect.
        info!("Sending lost peer notification for seed peer at old address");
        send_lost_peer_await_delivery(
            &mut connection_notifs_tx,
            seed_peer_id,
            seed_peer_id,
            seed_addr.clone(),
            DisconnectReason::ConnectionLost,
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // We should try to connect to both the new address and seed address.
        info!("Waiting to receive dial request to seed peer at new address");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            new_seed_addr,
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        info!("Waiting to receive dial request to seed peer at seed address");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr,
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn addr_change() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // Send address of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;

        // Send request to connect to other peer at old address. ConnectivityManager should not
        // dial, since we are already connected at the new address. The absence of another dial
        // attempt is hard to test explicitly. It will get implicitly tested if the dial
        // attempt arrives in place of some other expected message in the future.
        info!("Sending same address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        let other_addr_new = NetworkAddress::from_str("/ip4/127.0.1.1/tcp/8080").unwrap();
        // Send new address of other peer.
        info!("Sending new address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr_new.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();
        let connected_size = get_connected_size(&mut conn_mgr_reqs_tx).await;
        assert_eq!(1, connected_size);
        // We expect the peer which changed its address to also disconnect. (even if the address doesn't match storage)
        info!("Sending lost peer notification for other peer at old address");
        send_lost_peer_await_delivery(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_id,
            other_addr_new.clone(),
            DisconnectReason::ConnectionLost,
        )
        .await;

        let connected_size = get_connected_size(&mut conn_mgr_reqs_tx).await;
        assert_eq!(0, connected_size);

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager then receives a request to connect to the other peer at new address.
        info!("Waiting to receive dial request to other peer at new address");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_new,
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn lost_connection() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // Send address of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;

        // Notify connectivity actor of loss of connection to other_peer.
        info!("Sending LostPeer event to signal connection loss");
        send_lost_peer_await_delivery(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_id,
            other_addr.clone(),
            DisconnectReason::ConnectionLost,
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer after loss of
        // connection.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn disconnect() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    let events_f = async move {
        info!("Sending pubkey set of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                [(other_peer_id, other_pubkeys)].iter().cloned().collect(),
            ))
            .await
            .unwrap();

        // Send address of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;

        // Send request to make other peer ineligible.
        info!("Sending request to make other peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                HashMap::new(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(events_f);
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    let events_f = async move {
        info!("Sending pubkey set of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                [(other_peer_id, other_pubkeys)].iter().cloned().collect(),
            ))
            .await
            .unwrap();

        // Send address of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager again receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;

        // Send request to make other peer ineligible.
        info!("Sending request to make other peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                HashMap::new(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to disconnect from the other peer, which fails.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::Interrupted,
            ))),
        )
        .await;

        // Trigger connectivity check again.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives another request to disconnect from the other peer, which now
        // succeeds.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            other_peer_id,
            other_addr.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(events_f);
}

#[test]
// Tests that if we dial an already connected peer or disconnect from an already disconnected
// peer, connectivity manager does not send any additional dial or disconnect requests.
fn no_op_requests() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let other_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
    let other_pubkeys: HashSet<_> = [other_pubkey].iter().copied().collect();
    let other_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    let events_f = async move {
        info!("Sending pubkey set of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                [(other_peer_id, other_pubkeys)].iter().cloned().collect(),
            ))
            .await
            .unwrap();

        // Send address of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(other_peer_id, vec![other_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::AlreadyConnected(other_addr.clone())),
        )
        .await;

        // Send a delayed NewPeer notification.
        info!("Sending delayed NewPeer notification for other peer");
        send_new_peer_await_delivery(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_id,
            other_addr.clone(),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Send request to make other peer ineligible.
        info!("Sending request to make other peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                HashMap::new(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to disconnect from the other peer, which fails.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            other_peer_id,
            other_addr.clone(),
            Err(PeerManagerError::NotConnected(other_peer_id)),
        )
        .await;

        // Send delayed LostPeer notification for other peer.
        send_lost_peer_await_delivery(
            &mut connection_notifs_tx,
            other_peer_id,
            other_peer_id,
            other_addr,
            DisconnectReason::ConnectionLost,
        )
        .await;

        // Trigger connectivity check again. We don't expect connectivity manager to do
        // anything - if it does, the task should panic. That may not fail the test (right
        // now), but will be easily spotted by someone running the tests locallly.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();
    };
    rt.block_on(events_f);
}

#[test]
fn backoff_on_failure() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let eligible_peers = vec![];
    let seed_addrs = HashMap::new();
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    let events_f = async move {
        let (peer_a, _, peer_a_keys, peer_a_addr) = gen_peer();
        let (peer_b, _, peer_b_keys, peer_b_addr) = gen_peer();

        info!("Sending list of eligible peers");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                DiscoverySource::OnChain,
                [(peer_a, peer_a_keys), (peer_b, peer_b_keys)]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Send address of peer a.
        info!("Sending address of peer a");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(peer_a, vec![peer_a_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();
        // Send address of peer b.
        info!("Sending address of peer b");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(peer_b, vec![peer_b_addr.clone()])]
                    .iter()
                    .cloned()
                    .collect(),
            ))
            .await
            .unwrap();

        // Send NewPeer notification for peer_b.
        info!("Sending NewPeer notification for peer b");
        send_new_peer_await_delivery(&mut connection_notifs_tx, peer_b, peer_b, peer_b_addr).await;

        // We fail 10 attempts and ensure that the elapsed duration between successive attempts is
        // always greater than 100ms (the fixed backoff). In production, an exponential backoff
        // strategy is used.
        for _ in 0..10 {
            let start = Instant::now();
            // Trigger connectivity check.
            info!("Sending tick to trigger connectivity check");
            ticker_tx.send(()).await.unwrap();
            // Peer manager receives a request to connect to the seed peer.
            info!("Waiting to receive dial request");
            expect_dial_request(
                &mut connection_reqs_rx,
                &mut connection_notifs_tx,
                &mut conn_mgr_reqs_tx,
                peer_a,
                peer_a_addr.clone(),
                Err(PeerManagerError::IoError(io::Error::from(
                    io::ErrorKind::ConnectionRefused,
                ))),
            )
            .await;
            let elapsed = Instant::now().duration_since(start);
            info!("Duration elapsed: {:?}", elapsed);
            assert!(elapsed.as_millis() >= 100);
        }
    };
    rt.block_on(events_f);
}

// Test that connectivity manager will still connect to a peer if it advertises
// multiple listen addresses and some of them don't work.
#[test]
fn multiple_addrs_basic() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // For this test, the peer advertises multiple listen addresses. Assume
        // that the first addr fails to connect while the second addr succeeds.
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(
                    other_peer_id,
                    vec![other_addr_1.clone(), other_addr_2.clone()],
                )]
                .iter()
                .cloned()
                .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_1.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Since the last connection attempt failed for other_addr_1, we should
        // attempt the next available listener address. In this case, the call
        // succeeds and we should connect to the peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_2.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

// Test that connectivity manager will work with multiple addresses even if we
// retry more times than there are addresses.
#[test]
fn multiple_addrs_wrapping() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(
                    other_peer_id,
                    vec![other_addr_1.clone(), other_addr_2.clone()],
                )]
                .iter()
                .cloned()
                .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_1.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // The second attempt also fails.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_2.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Our next attempt should wrap around to the first address.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_1.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

// Test that connectivity manager will still work when dialing a peer with
// multiple listen addrs and then that peer advertises a smaller number of addrs.
#[test]
fn multiple_addrs_shrinking() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let other_peer_id = PeerId::random();
    let eligible_peers = vec![other_peer_id];
    let seed_addrs = HashMap::new();
    info!("Other peer_id is {}", other_peer_id.short_str());
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, eligible_peers, seed_addrs);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let other_addr_1 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let other_addr_2 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();
        let other_addr_3 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of other peer.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(
                    other_peer_id,
                    vec![
                        other_addr_1.clone(),
                        other_addr_2.clone(),
                        other_addr_3.clone(),
                    ],
                )]
                .iter()
                .cloned()
                .collect(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_1,
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        let other_addr_4 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9094").unwrap();
        let other_addr_5 = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/9095").unwrap();

        // The peer issues a new, smaller set of listen addrs.
        info!("Sending address of other peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                DiscoverySource::OnChain,
                [(
                    other_peer_id,
                    vec![other_addr_4.clone(), other_addr_5.clone()],
                )]
                .iter()
                .cloned()
                .collect(),
            ))
            .await
            .unwrap();

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // After updating the addresses, we should dial the first new address,
        // other_addr_4 in this case.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            other_peer_id,
            other_addr_4,
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn public_connection_limit() {
    ::diem_logger::Logger::init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let mut seed_addrs: HashMap<PeerId, Vec<NetworkAddress>> = HashMap::new();
    let mut seed_pubkeys: HashMap<PeerId, HashSet<x25519::PublicKey>> = HashMap::new();
    for _ in 0..MAX_TEST_CONNECTIONS + 1 {
        let (peer_id, _, pubkeys, addr) = gen_peer();
        seed_pubkeys.insert(peer_id, pubkeys);
        seed_addrs.insert(peer_id, vec![addr]);
    }

    info!("Seed peers are {:?}", seed_pubkeys);
    let network_context = NetworkContext::new(
        NetworkId::vfn_network(),
        RoleType::FullNode,
        PeerId::random(),
    );
    let (mut connection_reqs_rx, mut connection_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr_with_context(network_context, &mut rt, seed_addrs, seed_pubkeys);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // Peer manager receives a request to connect to the other peer.
        info!("Waiting to receive dial request");
        expect_num_dials(
            &mut connection_reqs_rx,
            &mut connection_notifs_tx,
            &mut conn_mgr_reqs_tx,
            MAX_TEST_CONNECTIONS,
        )
        .await;

        // Queue should be empty
        let queue_size = get_dial_queue_size(&mut conn_mgr_reqs_tx).await;
        assert_eq!(0, queue_size);

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // There shouldn't be dials, we already cleared the queue, just to ensure it's still clear
        info!("Check queue size");
        let queue_size = get_dial_queue_size(&mut conn_mgr_reqs_tx).await;
        assert_eq!(0, queue_size);
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn basic_update_eligible_peers() {
    // setup a basic connectivity manager without starting its event loop

    let network_context = Arc::new(NetworkContext::new(
        NetworkId::Validator,
        RoleType::Validator,
        PeerId::random(),
    ));
    let (connection_reqs_tx, _connection_reqs_rx) =
        diem_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (_connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();
    let (_conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(0);
    let (_ticker_tx, ticker_rx) = channel::new_test::<()>(0);
    let trusted_peers = Arc::new(RwLock::new(HashMap::new()));
    let seed_addrs = HashMap::new();
    let seed_pubkeys = HashMap::new();
    let mut rng = StdRng::from_seed(TEST_SEED);

    let mut conn_mgr = ConnectivityManager::new(
        network_context,
        trusted_peers.clone(),
        seed_addrs,
        seed_pubkeys,
        ticker_rx,
        ConnectionRequestSender::new(connection_reqs_tx),
        connection_notifs_rx,
        conn_mgr_reqs_rx,
        FixedInterval::from_millis(100),
        300,  /* ms */
        None, /* connection limit */
    );

    // sample some example data

    let peer_id_a = PeerId::random();
    let peer_id_b = PeerId::random();

    let pubkey_1 = x25519::PrivateKey::generate(&mut rng).public_key();
    let pubkey_2 = x25519::PrivateKey::generate(&mut rng).public_key();

    let pubkeys_1: HashSet<_> = vec![pubkey_1].into_iter().collect();
    let pubkeys_2: HashSet<_> = vec![pubkey_2].into_iter().collect();
    let pubkeys_1_2: HashSet<_, _> = vec![pubkey_1, pubkey_2].into_iter().collect();

    let pubkeys_map_empty = HashMap::new();
    let pubkeys_map_1: HashMap<_, _> = vec![(peer_id_a, pubkeys_1.clone())].into_iter().collect();
    let pubkeys_map_2: HashMap<_, _> = vec![(peer_id_a, pubkeys_2), (peer_id_b, pubkeys_1.clone())]
        .into_iter()
        .collect();
    let pubkeys_map_1_2: HashMap<_, _> = vec![(peer_id_a, pubkeys_1_2), (peer_id_b, pubkeys_1)]
        .into_iter()
        .collect();

    // basic one peer one discovery source
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_1.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1);

    // same update does nothing
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_1.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1);

    // reset
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_empty.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_empty);

    // basic union across multiple sources
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_1.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1);
    conn_mgr.handle_update_eligible_peers(DiscoverySource::Config, pubkeys_map_2);
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1_2);

    // does nothing even if another source has same set
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_1_2.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1_2);
    conn_mgr.handle_update_eligible_peers(DiscoverySource::Config, pubkeys_map_1_2.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1_2);

    // since on-chain and config now contain the same sets, clearing one should do nothing.
    conn_mgr.handle_update_eligible_peers(DiscoverySource::Config, pubkeys_map_empty.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_1_2);

    // reset
    conn_mgr.handle_update_eligible_peers(DiscoverySource::OnChain, pubkeys_map_empty.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_empty);

    // empty update again does nothing
    conn_mgr.handle_update_eligible_peers(DiscoverySource::Config, pubkeys_map_empty.clone());
    assert_eq!(&*trusted_peers.read(), &pubkeys_map_empty);
}
