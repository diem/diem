// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::peer_manager::PeerManagerRequest;
use core::str::FromStr;
use crypto::{signing, x25519};
use futures::{FutureExt, SinkExt, TryFutureExt};
use memsocket::MemorySocket;
use std::io;
use tokio::runtime::Runtime;

fn setup_conn_mgr(
    rt: &mut Runtime,
    seed_peer_id: PeerId,
    max_connection_attempts: Option<u32>,
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
    let (_, signing_public_key) = signing::generate_keypair();
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
            max_connection_attempts.map_or(u32::max_value(), |v| v),
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

async fn expect_disconnect_request<TSubstream>(
    peer_mgr_reqs_rx: &mut channel::Receiver<PeerManagerRequest<TSubstream>>,
    peer: PeerId,
    result: Result<(), PeerManagerError>,
) {
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::DisconnectPeer(p, error_tx) => {
            assert_eq!(peer, p);
            error_tx.send(result).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
}

async fn expect_dial_request<TSubstream>(
    peer_mgr_reqs_rx: &mut channel::Receiver<PeerManagerRequest<TSubstream>>,
    peer: PeerId,
    address: Multiaddr,
    result: Result<(), PeerManagerError>,
) {
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::DialPeer(p, addr, error_tx) => {
            assert_eq!(peer, p);
            assert_eq!(address, addr);
            error_tx.send(result).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
}

#[test]
fn addr_change() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, None);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
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
            seed_peer_id,
            seed_address.clone(),
            Ok(()),
        )
        .await;

        // Send request to connect to seed peer at old address. ConnectivityManager should not
        // dial, since we are already connected at the new address. The absence of another dial
        // attempt is hard to test explicitly. It will get implicitly tested if the dial
        // attempt arrives in place of some other expected message in the future.
        info!("Sending redundant request to connect to seed peer.");
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
        // Send request to connect to seed peer at new address.
        info!("Sending request to connect to seed peer at new address.");
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
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, None);

    // Fake peer manager and discovery.
    let f_peer_mgr = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
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
    let (mut peer_mgr_reqs_rx, _, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, None);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
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
        expect_disconnect_request(&mut peer_mgr_reqs_rx, seed_peer_id, Ok(())).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

// Tests that connectivity manager retries dials and disconnects on failure.
#[test]
fn retry_on_failure() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    let (mut peer_mgr_reqs_rx, _, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, None);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
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
            seed_peer_id,
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
        expect_disconnect_request(&mut peer_mgr_reqs_rx, seed_peer_id, Ok(())).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
// Tests that if we dial a an already connected peer or disconnect from an already disconnected
// peer, connectivity manager does not send any additional dial or disconnect requests.
fn no_op_requests() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    let (mut peer_mgr_reqs_rx, _, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, None);

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
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
            seed_peer_id,
            seed_address.clone(),
            Err(PeerManagerError::AlreadyConnected(seed_address.clone())),
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
            seed_peer_id,
            Err(PeerManagerError::NotConnected(seed_peer_id)),
        )
        .await;

        // Trigger connectivity check again. We don't expect connectivity manager to do
        // anything - if it does, the task should panic. That may not fail the test (right
        // now), but will be easily spotted by someone running the tests locally.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

// Tests that connectivity manager does not retry to connect to a peer after
// `max_connection_attempts`.
#[test]
fn no_retry_after_max_attempts() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let seed_peer_id = PeerId::random();
    let (mut peer_mgr_reqs_rx, _, mut conn_mgr_reqs_tx, mut ticker_tx) =
        setup_conn_mgr(&mut rt, seed_peer_id, Some(5));

    let events_f = async move {
        let seed_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        // Send request to connect to seed peer.
        info!("Sending request to connect to seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                seed_peer_id,
                vec![seed_address.clone()],
            ))
            .await
            .unwrap();

        // Trigger failures for first 5 connection attempts.
        for _ in 0..5 {
            // Trigger connectivity check.
            info!("Sending tick to trigger connectivity check");
            ticker_tx.send(()).await.unwrap();
            // Peer manager receives a request to connect to the seed peer. Error is returned.
            info!("Waiting to receive dial request");
            expect_dial_request(
                &mut peer_mgr_reqs_rx,
                seed_peer_id,
                seed_address.clone(),
                Err(PeerManagerError::IoError(io::Error::from(
                    io::ErrorKind::ConnectionRefused,
                ))),
            )
            .await;
        }

        // If we trigger another 5 ticks, no connection attempt is made.
        for _ in 0..5 {
            // Trigger connectivity check.
            info!("Sending tick to trigger connectivity check");
            ticker_tx.send(()).await.unwrap();
        }

        let other_peer_id = PeerId::random();
        let other_peer_keys = {
            let (_, signing_public_key) = signing::generate_keypair();
            let (_, identity_public_key) = x25519::generate_keypair();
            NetworkPublicKeys {
                identity_public_key,
                signing_public_key,
            }
        };
        let other_peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();

        // Add new eligible peer.
        info!("Sending request to make another peer eligible");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateEligibleNodes(
                [(other_peer_id, other_peer_keys)].iter().cloned().collect(),
            ))
            .await
            .unwrap();
        // Add address of new peer.
        info!("Sending request to connect to seed peer");
        conn_mgr_reqs_tx
            .send(ConnectivityRequest::UpdateAddresses(
                other_peer_id,
                vec![other_peer_address.clone()],
            ))
            .await
            .unwrap();
        // Trigger connectivity check.
        info!("Sending tick to trigger connectivity check");
        ticker_tx.send(()).await.unwrap();

        // Peer manager should receive a dial request for new peer. If a connection attempt to seed
        // peer was made on prior ticks, we would have seen that event first.
        info!("Waiting to receive dial request");
        expect_dial_request(
            &mut peer_mgr_reqs_rx,
            other_peer_id,
            other_peer_address.clone(),
            Ok(()),
        )
        .await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}
