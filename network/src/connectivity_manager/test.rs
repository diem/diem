// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::peer::DisconnectReason;
use crate::peer_manager::PeerManagerRequest;
use channel::message_queues::QueueStyle;
use core::str::FromStr;
use futures::SinkExt;
use libra_crypto::{ed25519::compat, test_utils::TEST_SEED, x25519};
use rand::{rngs::StdRng, SeedableRng};
use std::io;
use std::num::NonZeroUsize;
use tokio::runtime::Runtime;
use tokio_retry::strategy::FixedInterval;

fn setup_conn_mgr(
    rt: &mut Runtime,
    seed_peer_id: PeerId,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    channel::Sender<ConnectivityRequest>,
    channel::Sender<()>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(0);
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (_, signing_public_key) = compat::generate_keypair(&mut rng);
    let (_, identity_public_key) = x25519::compat::generate_keypair(&mut rng);
    let conn_mgr = {
        ConnectivityManager::new(
            Arc::new(RwLock::new(
                vec![(
                    seed_peer_id,
                    NetworkPublicKeys {
                        identity_public_key,
                        signing_public_key,
                    },
                )]
                .into_iter()
                .collect(),
            )),
            ticker_rx,
            PeerManagerRequestSender::new(peer_mgr_reqs_tx),
            peer_mgr_notifs_rx,
            conn_mgr_reqs_rx,
            FixedInterval::from_millis(100),
            300, /* ms */
        )
    };
    rt.spawn(conn_mgr.start());
    (
        peer_mgr_reqs_rx,
        peer_mgr_notifs_tx,
        conn_mgr_reqs_tx,
        ticker_tx,
    )
}

fn gen_peer() -> (PeerId, NetworkPublicKeys) {
    let peer_id = PeerId::random();
    let mut rng = StdRng::from_seed(TEST_SEED);
    let (_, signing_public_key) = compat::generate_keypair(&mut rng);
    let (_, identity_public_key) = x25519::compat::generate_keypair(&mut rng);
    (
        peer_id,
        NetworkPublicKeys {
            identity_public_key,
            signing_public_key,
        },
    )
}

async fn get_dial_queue_size(conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>) -> usize {
    let (queue_size_tx, queue_size_rx) = oneshot::channel();
    conn_mgr_reqs_tx
        .send(ConnectivityRequest::GetDialQueueSize(queue_size_tx))
        .await
        .unwrap();
    queue_size_rx.await.unwrap()
}

async fn send_notification_await_delivery(
    peer_mgr_notifs_tx: &mut libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    peer_id: PeerId,
    notif: PeerManagerNotification,
) {
    let (delivered_tx, delivered_rx) = oneshot::channel();
    peer_mgr_notifs_tx
        .push_with_feedback((peer_id, ProtocolId::default()), notif, Some(delivered_tx))
        .unwrap();
    delivered_rx.await.unwrap();
}

async fn expect_disconnect_request(
    peer_mgr_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    peer_mgr_notifs_tx: &mut libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    peer_id: PeerId,
    address: Multiaddr,
    result: Result<(), PeerManagerError>,
) {
    let success = result.is_ok();
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::DisconnectPeer(p, error_tx) => {
            assert_eq!(peer_id, p);
            error_tx.send(result).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
    if success {
        send_notification_await_delivery(
            peer_mgr_notifs_tx,
            peer_id,
            PeerManagerNotification::LostPeer(peer_id, address, DisconnectReason::Requested),
        )
        .await;
    }
}

async fn expect_dial_request(
    peer_mgr_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    peer_mgr_notifs_tx: &mut libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    conn_mgr_reqs_tx: &mut channel::Sender<ConnectivityRequest>,
    peer_id: PeerId,
    address: Multiaddr,
    result: Result<(), PeerManagerError>,
) {
    let success = result.is_ok();
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::DialPeer(p, addr, error_tx) => {
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
        send_notification_await_delivery(
            peer_mgr_notifs_tx,
            peer_id,
            PeerManagerNotification::NewPeer(peer_id, address),
        )
        .await;
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
fn addr_change() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send address of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;

        // Send request to connect to seed peer at old address. ConnectivityManager should not
        // dial, since we are already connected at the new address. The absence of another dial
        // attempt is hard to test explicitly. It will get implicitly tested if the dial
        // attempt arrives in place of some other expected message in the future.
        info!("Sending same address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        let seed_address_new = Multiaddr::from_str("/ip4/127.0.1.1/tcp/8080").unwrap();
        // Send new address of seed peer.
        info!("Sending new address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address_new.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // We expect the peer which changed its address to also disconnect.
        info!("Sending lost peer notification for seed peer at old address");
        send_notification_await_delivery(
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address,
                DisconnectReason::ConnectionLost,
            ),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager then receives a request to connect to the seed peer at new address.
        info!("Waiting to receive dial request to seed peer at new address");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address_new,
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn lost_connection() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send address of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;

        // Notify connectivity actor of loss of connection to seed_peer.
        info!("Sending LostPeer event to signal connection loss");
        send_notification_await_delivery(
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address.clone(),
                DisconnectReason::ConnectionLost,
            ),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer after loss of
        // connection.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}

#[test]
fn disconnect() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send address of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;

        // Send request to make seed peer ineligible.
        info!("Sending request to make seed peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(HashMap::new()))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(events_f);
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send address of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager again receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;

        // Send request to make seed peer ineligible.
        info!("Sending request to make seed peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(HashMap::new()))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to disconnect from the seed peer, which fails.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            seed_address.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::Interrupted,
            ))),
        )
        .await;

        // Trigger connectivity check again.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives another request to disconnect from the seed peer, which now
        // succeeds.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            seed_address.clone(),
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send address of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to connect to the seed peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_address.clone(),
            Err(PeerManagerError::AlreadyConnected(seed_address.clone())),
        )
        .await;

        // Send a delayed NewPeer notification.
        info!("Sending delayed NewPeer notification for seed peer");
        send_notification_await_delivery(
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            PeerManagerNotification::NewPeer(seed_peer_id, seed_address.clone()),
        )
        .await;

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Send request to make seed peer ineligible.
        info!("Sending request to make seed peer ineligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(HashMap::new()))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager receives a request to disconnect from the seed peer, which fails.
        info!("Waiting to receive disconnect request");
        expect_disconnect_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            seed_address.clone(),
            Err(PeerManagerError::NotConnected(seed_peer_id)),
        )
        .await;

        // Send delayed LostPeer notification for seed peer.
        send_notification_await_delivery(
            &mut peer_mgr_notifs_tx,
            seed_peer_id,
            PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address.clone(),
                DisconnectReason::ConnectionLost,
            ),
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    let events_f = async move {
        let (peer_a, peer_a_keys) = gen_peer();
        let peer_a_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
        let (peer_b, peer_b_keys) = gen_peer();
        let peer_b_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/8080").unwrap();

        info!("Sending list of eligible peers");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
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
                peer_a,
                vec![peer_a_address.clone()],
            ))
            .await
            .unwrap();
        // Send address of peer b.
        info!("Sending address of peer b");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                peer_b,
                vec![peer_b_address.clone()],
            ))
            .await
            .unwrap();

        // Send NewPeer notification for peer_b.
        info!("Sending NewPeer notification for peer b");
        send_notification_await_delivery(
            &mut peer_mgr_notifs_tx,
            peer_b,
            PeerManagerNotification::NewPeer(peer_b, peer_b_address.clone()),
        )
        .await;

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
                &mut peer_mgr_reqs_rx,
                &mut peer_mgr_notifs_tx,
                &mut conn_mgr_reqs_tx,
                peer_a,
                peer_a_address.clone(),
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        // For this test, the peer advertises multiple listen addresses. Assume
        // that the first addr fails to connect while the second addr succeeds.
        let seed_addr_1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let seed_addr_2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_addr_1.clone(), seed_addr_2.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_1.clone(),
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Since the last connection attempt failed for seed_addr_1, we should
        // attempt the next available listener address. In this case, the call
        // succeeds and we should connect to the peer.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_2.clone(),
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_addr_1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let seed_addr_2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_addr_1.clone(), seed_addr_2.clone()],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_1.clone(),
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
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_2.clone(),
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
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_1.clone(),
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
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    info!("Seed peer_id is {}", seed_peer_id.short_str());
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_addr_1 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9091").unwrap();
        let seed_addr_2 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();
        let seed_addr_3 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9092").unwrap();

        // Send addresses of seed peer.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![
                    seed_addr_1.clone(),
                    seed_addr_2.clone(),
                    seed_addr_3.clone(),
                ],
            ))
            .await
            .unwrap();

        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Assume that the first listen addr fails to connect.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_1,
            Err(PeerManagerError::IoError(io::Error::from(
                io::ErrorKind::ConnectionRefused,
            ))),
        )
        .await;

        let seed_addr_4 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9094").unwrap();
        let seed_addr_5 = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9095").unwrap();

        // The peer issues a new, smaller set of listen addrs.
        info!("Sending address of seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_addr_4.clone(), seed_addr_5.clone()],
            ))
            .await
            .unwrap();

        // Trigger another connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // After updating the addresses, we should dial the first new address,
        // seed_addr_4 in this case.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            &mut peer_mgr_notifs_tx,
            &mut conn_mgr_reqs_tx,
            seed_peer_id,
            seed_addr_4,
            Ok(()),
        )
        .await;
    };
    rt.block_on(f_peer_mgr);
}
