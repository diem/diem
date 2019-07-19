// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::peer_manager::PeerManagerRequest;
use core::str::FromStr;
use crypto::x25519;
use futures::{FutureExt, SinkExt, TryFutureExt};
use memsocket::MemorySocket;
use nextgen_crypto::ed25519::compat;
use std::io;
use tokio::runtime::Runtime;
use tokio_retry::strategy::FixedInterval;

fn setup_conn_mgr(
    rt: &mut Runtime,
    seed_peer_id: PeerId,
) -> (
    channel::Receiver<PeerManagerRequest<MemorySocket>>,
    channel::Sender<PeerManagerNotification<MemorySocket>>,
    channel::Sender<ConnectivityRequest>,
    channel::Sender<()>,
) {
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx): (
        channel::Sender<PeerManagerRequest<MemorySocket>>,
        _,
    ) = channel::new_test(0);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = channel::new_test(0);
    let (conn_mgr_reqs_tx, conn_mgr_reqs_rx) = channel::new_test(0);
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let (_, signing_public_key) = compat::generate_keypair(None);
    let (_, identity_public_key) = x25519::generate_keypair();
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
    rt.spawn(conn_mgr.start().boxed().unit_error().compat());
    (
        peer_mgr_reqs_rx,
        peer_mgr_notifs_tx,
        conn_mgr_reqs_tx,
        ticker_tx,
    )
}

fn gen_peer() -> (PeerId, NetworkPublicKeys) {
    let peer_id = PeerId::random();
    let (_, signing_public_key) = compat::generate_keypair(None);
    let (_, identity_public_key) = x25519::generate_keypair();
    (
        peer_id,
        NetworkPublicKeys {
            identity_public_key,
            signing_public_key,
        },
    )
}

async fn expect_disconnect_request<'a, TSubstream>(
    peer_mgr_reqs_rx: &'a mut channel::Receiver<PeerManagerRequest<TSubstream>>,
    peer_mgr_notifs_tx: &'a mut channel::Sender<PeerManagerNotification<TSubstream>>,
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::LostPeer(peer_id, address))
            .await
            .unwrap();
    }
}

async fn expect_dial_request<'a, TSubstream>(
    peer_mgr_reqs_rx: &'a mut channel::Receiver<PeerManagerRequest<TSubstream>>,
    peer_mgr_notifs_tx: &'a mut channel::Sender<PeerManagerNotification<TSubstream>>,
    conn_mgr_reqs_tx: &'a mut channel::Sender<ConnectivityRequest>,
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(peer_id, address))
            .await
            .unwrap();
    }

    // Wait for dial queue to be empty. Without this, it's impossible to guarantee that a completed
    // dial is removed from a dial queue. We need this guarantee to see the effects of future
    // triggers for connectivity check.
    info!("Waiting for dial queue to be empty");
    loop {
        // Send request to connectivity manager to get dial queue size.
        let (queue_size_tx, queue_size_rx) = oneshot::channel();
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::GetDialQueueSize(queue_size_tx))
            .await
            .unwrap();
        if queue_size_rx.await.unwrap() == 0 {
            break;
        }
    }
}

#[test]
fn addr_change() {
    ::logger::try_init_for_testing();
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address,
            ))
            .await
            .unwrap();

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
    rt.block_on(f_peer_mgr.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn lost_connection() {
    ::logger::try_init_for_testing();
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address.clone(),
            ))
            .await
            .unwrap();

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
    rt.block_on(f_peer_mgr.boxed().unit_error().compat())
        .unwrap();
}

#[test]
fn disconnect() {
    ::logger::try_init_for_testing();
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
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    ::logger::try_init_for_testing();
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
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
// Tests that if we dial an already connected peer or disconnect from an already disconnected
// peer, connectivity manager does not send any additional dial or disconnect requests.
fn no_op_requests() {
    ::logger::try_init_for_testing();
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                seed_peer_id,
                seed_address.clone(),
            ))
            .await
            .unwrap();

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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::LostPeer(
                seed_peer_id,
                seed_address.clone(),
            ))
            .await
            .unwrap();

        // Trigger connectivity check again. We don't expect connectivity manager to do
        // anything - if it does, the task should panic. That may not fail the test (right
        // now), but will be easily spotted by someone running the tests locallly.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn backoff_on_failure() {
    ::logger::try_init_for_testing();
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
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_b,
                peer_b_address.clone(),
            ))
            .await
            .unwrap();

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
            assert!(elapsed.as_millis() <= 150);
        }
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}
