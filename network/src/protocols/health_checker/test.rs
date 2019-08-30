// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{common::NegotiatedSubstream, peer_manager::PeerManagerRequest};
use futures::future::{FutureExt, TryFutureExt};
use memsocket::MemorySocket;
use parity_multiaddr::Multiaddr;
use std::str::FromStr;
use tokio::runtime::Runtime;

const PING_TIMEOUT: Duration = Duration::from_millis(500);

fn setup_permissive_health_checker(
    rt: &mut Runtime,
    ping_failures_tolerated: u64,
) -> (
    channel::Receiver<PeerManagerRequest<MemorySocket>>,
    channel::Sender<PeerManagerNotification<MemorySocket>>,
    channel::Sender<()>,
) {
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = channel::new_test(0);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = channel::new_test(0);
    let health_checker = HealthChecker::new(
        ticker_rx,
        PeerManagerRequestSender::new(peer_mgr_reqs_tx),
        peer_mgr_notifs_rx,
        PING_TIMEOUT,
        ping_failures_tolerated,
    );
    rt.spawn(health_checker.start().boxed().unit_error().compat());
    (peer_mgr_reqs_rx, peer_mgr_notifs_tx, ticker_tx)
}

fn setup_default_health_checker(
    rt: &mut Runtime,
) -> (
    channel::Receiver<PeerManagerRequest<MemorySocket>>,
    channel::Sender<PeerManagerNotification<MemorySocket>>,
    channel::Sender<()>,
) {
    let (ticker_tx, ticker_rx) = channel::new_test(0);
    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = channel::new_test(0);
    let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = channel::new_test(0);
    let health_checker = HealthChecker::new(
        ticker_rx,
        PeerManagerRequestSender::new(peer_mgr_reqs_tx),
        peer_mgr_notifs_rx,
        PING_TIMEOUT,
        0,
    );
    rt.spawn(health_checker.start().boxed().unit_error().compat());
    (peer_mgr_reqs_rx, peer_mgr_notifs_tx, ticker_tx)
}

async fn send_ping_expect_pong(substream: MemorySocket) {
    // Messages are length-prefixed. Wrap in a framed stream.
    let mut substream = Framed::new(substream.compat(), UviBytes::<Bytes>::default()).sink_compat();
    // Send ping.
    substream
        .send(Ping::default().to_bytes().unwrap())
        .await
        .unwrap();
    // Expect Pong.
    let _: Pong = read_proto(&mut substream).await.unwrap();
}

async fn expect_ping_send_ok(substream: MemorySocket) {
    // Messages are length-prefixed. Wrap in a framed stream.
    let mut substream = Framed::new(substream.compat(), UviBytes::<Bytes>::default()).sink_compat();
    // Read ping.
    let _: Ping = read_proto(&mut substream).await.unwrap();
    // Send Pong.
    substream
        .send(Pong::default().to_bytes().unwrap())
        .await
        .unwrap();
}

async fn expect_ping_send_notok(substream: MemorySocket) {
    // Messages are length-prefixed. Wrap in a framed stream.
    let mut substream = Framed::new(substream.compat(), UviBytes::<Bytes>::default()).sink_compat();
    // Read ping.
    let _: Ping = read_proto(&mut substream).await.unwrap();
    substream.close().await.unwrap();
}

async fn expect_ping_timeout(substream: MemorySocket) {
    // Messages are length-prefixed. Wrap in a framed stream.
    let mut substream = Framed::new(substream.compat(), UviBytes::<Bytes>::default()).sink_compat();
    // Read ping.
    let _: Ping = read_proto(&mut substream).await.unwrap();
    // Sleep for ping timeout plus a little bit.
    std::thread::sleep(PING_TIMEOUT + Duration::from_millis(100));
}

async fn open_substream_and_notify(
    peer_id: PeerId,
    peer_mgr_notifs_tx: &mut channel::Sender<PeerManagerNotification<MemorySocket>>,
) -> MemorySocket {
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();

    peer_mgr_notifs_tx
        .send(PeerManagerNotification::NewInboundSubstream(
            peer_id,
            NegotiatedSubstream {
                protocol: ProtocolId::from_static(PING_PROTOCOL_NAME),
                substream: listener_substream,
            },
        ))
        .await
        .unwrap();
    dialer_substream
}

async fn expect_disconnect(
    peer_id: PeerId,
    peer_mgr_reqs_rx: &mut channel::Receiver<PeerManagerRequest<MemorySocket>>,
) {
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::DisconnectPeer(peer, ch) => {
            assert_eq!(peer, peer_id);
            ch.send(Ok(())).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
}

async fn expect_open_substream(
    peer_id: PeerId,
    peer_mgr_reqs_rx: &mut channel::Receiver<PeerManagerRequest<MemorySocket>>,
) -> MemorySocket {
    let (dialer_substream, listener_substream) = MemorySocket::new_pair();
    match peer_mgr_reqs_rx.next().await.unwrap() {
        PeerManagerRequest::OpenSubstream(peer, protocol, ch) => {
            assert_eq!(peer, peer_id);
            assert_eq!(protocol, PING_PROTOCOL_NAME);
            ch.send(Ok(dialer_substream)).unwrap();
        }
        _ => {
            panic!("unexpected request to peer manager");
        }
    }
    listener_substream
}

#[test]
fn outbound() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_default_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        let peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_id,
                peer_address.clone(),
            ))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should request for a new substream.
        let listener_substream = expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;

        // Health checker should send a ping request.
        expect_ping_send_ok(listener_substream).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn inbound() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (_, mut peer_mgr_notifs_tx, _) = setup_default_health_checker(&mut rt);

    let events_f = async move {
        let peer_id = PeerId::random();

        // Send notification about incoming Ping substream.
        let dialer_substream = open_substream_and_notify(peer_id, &mut peer_mgr_notifs_tx).await;

        // Send ping and expect pong in return.
        send_ping_expect_pong(dialer_substream).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn outbound_failure_permissive() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let ping_failures_tolerated = 10;
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        let peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_id,
                peer_address.clone(),
            ))
            .await
            .unwrap();

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        for _ in 0..=ping_failures_tolerated {
            ticker_tx.send(()).await.unwrap();
            // Health checker should request for a new substream.
            let listener_substream = expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;
            // Health checker should send a ping request which fails.
            expect_ping_send_notok(listener_substream).await;
        }
        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut peer_mgr_reqs_rx).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn ping_success_resets_fail_counter() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let failures_triggered = 10;
    let ping_failures_tolerated = 2 * 10;
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        let peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();
        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_id,
                peer_address.clone(),
            ))
            .await
            .unwrap();
        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        {
            for _ in 0..failures_triggered {
                ticker_tx.send(()).await.unwrap();
                // Health checker should request for a new substream.
                let listener_substream =
                    expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(listener_substream).await;
            }
        }
        // Trigger successful ping. This should reset the counter of ping failures.
        {
            ticker_tx.send(()).await.unwrap();
            // Health checker should request for a new substream.
            let listener_substream = expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;
            // Health checker should send a ping request which succeeds
            expect_ping_send_ok(listener_substream).await;
        }
        // We would then need to fail for more than `ping_failures_tolerated` times before
        // triggering disconnect.
        {
            for _ in 0..=ping_failures_tolerated {
                ticker_tx.send(()).await.unwrap();
                // Health checker should request for a new substream.
                let listener_substream =
                    expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(listener_substream).await;
            }
        }
        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut peer_mgr_reqs_rx).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn outbound_failure_strict() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_default_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        let peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_id,
                peer_address.clone(),
            ))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should request for a new substream.
        let listener_substream = expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;

        // Health checker should send a ping request which fails.
        expect_ping_send_notok(listener_substream).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut peer_mgr_reqs_rx).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}

#[test]
fn ping_timeout() {
    ::logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut peer_mgr_reqs_rx, mut peer_mgr_notifs_tx, mut ticker_tx) =
        setup_default_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        let peer_address = Multiaddr::from_str("/ip4/127.0.0.1/tcp/9090").unwrap();

        peer_mgr_notifs_tx
            .send(PeerManagerNotification::NewPeer(
                peer_id,
                peer_address.clone(),
            ))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should request for a new substream.
        let listener_substream = expect_open_substream(peer_id, &mut peer_mgr_reqs_rx).await;

        // Health checker should send a ping request which fails.
        expect_ping_timeout(listener_substream).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut peer_mgr_reqs_rx).await;
    };
    rt.block_on(events_f.boxed().unit_error().compat()).unwrap();
}
