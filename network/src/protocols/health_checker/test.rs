// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer_manager::{
        self, conn_status_channel, ConnectionRequest, PeerManagerNotification, PeerManagerRequest,
    },
    protocols::rpc::InboundRpcRequest,
    ProtocolId,
};
use channel::{libra_channel, message_queues::QueueStyle};
use futures::sink::SinkExt;
use parity_multiaddr::Multiaddr;
use std::{num::NonZeroUsize, str::FromStr};
use tokio::runtime::Runtime;

const PING_TIMEOUT: Duration = Duration::from_millis(500);

fn setup_permissive_health_checker(
    rt: &mut Runtime,
    ping_failures_tolerated: u64,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    libra_channel::Receiver<PeerId, ConnectionRequest>,
    conn_status_channel::Sender,
    channel::Sender<()>,
) {
    let (ticker_tx, ticker_rx) = channel::new_test(0);

    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_reqs_tx, connection_reqs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (network_notifs_tx, network_notifs_rx) =
        libra_channel::new(QueueStyle::FIFO, NonZeroUsize::new(1).unwrap(), None);
    let (connection_notifs_tx, connection_notifs_rx) = conn_status_channel::new();

    let hc_network_tx = HealthCheckerNetworkSender::new(
        PeerManagerRequestSender::new(peer_mgr_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let hc_network_rx = HealthCheckerNetworkEvents::new(network_notifs_rx, connection_notifs_rx);

    let health_checker = HealthChecker::new(
        ticker_rx,
        hc_network_tx,
        hc_network_rx,
        PING_TIMEOUT,
        ping_failures_tolerated,
    );
    rt.spawn(health_checker.start());
    (
        peer_mgr_reqs_rx,
        network_notifs_tx,
        connection_reqs_rx,
        connection_notifs_tx,
        ticker_tx,
    )
}

fn setup_strict_health_checker(
    rt: &mut Runtime,
) -> (
    libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    libra_channel::Receiver<PeerId, ConnectionRequest>,
    conn_status_channel::Sender,
    channel::Sender<()>,
) {
    setup_permissive_health_checker(rt, 0 /* ping_failures_tolerated */)
}

async fn expect_ping(
    network_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) -> (Ping, oneshot::Sender<Result<Bytes, RpcError>>) {
    let req = network_reqs_rx.next().await.unwrap();
    let rpc_req = match req {
        PeerManagerRequest::SendRpc(_peer_id, rpc_req) => rpc_req,
        _ => panic!("Unexpected PeerManagerRequest: {:?}", req),
    };

    let protocol = rpc_req.protocol;
    let req_data = rpc_req.data;
    let res_tx = rpc_req.res_tx;

    assert_eq!(
        protocol,
        ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL)
    );

    match lcs::from_bytes(&req_data).unwrap() {
        HealthCheckerMsg::Ping(ping) => (ping, res_tx),
        msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
    }
}

async fn expect_ping_send_ok(
    network_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) {
    let (ping, res_tx) = expect_ping(network_reqs_rx).await;
    let res_data = lcs::to_bytes(&HealthCheckerMsg::Pong(Pong(ping.0))).unwrap();
    res_tx.send(Ok(res_data.into())).unwrap();
}

async fn expect_ping_send_notok(
    network_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) {
    let (_ping_msg, res_tx) = expect_ping(network_reqs_rx).await;
    // This mock ping request must fail.
    res_tx.send(Err(RpcError::TimedOut)).unwrap();
}

async fn expect_ping_timeout(
    network_reqs_rx: &mut libra_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) {
    let (_ping_msg, _res_tx) = expect_ping(network_reqs_rx).await;
    // Sleep for ping timeout plus a little bit.
    std::thread::sleep(PING_TIMEOUT + Duration::from_millis(100));
}

async fn send_inbound_ping(
    peer_id: PeerId,
    ping: u32,
    network_notifs_tx: &mut libra_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
) -> oneshot::Receiver<Result<Bytes, RpcError>> {
    let protocol = ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL);
    let data = lcs::to_bytes(&HealthCheckerMsg::Ping(Ping(ping)))
        .unwrap()
        .into();
    let (res_tx, res_rx) = oneshot::channel();
    let inbound_rpc_req = InboundRpcRequest {
        protocol,
        data,
        res_tx,
    };
    let key = (
        peer_id,
        ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL),
    );
    let (delivered_tx, delivered_rx) = oneshot::channel();
    network_notifs_tx
        .push_with_feedback(
            key.clone(),
            PeerManagerNotification::RecvRpc(peer_id, inbound_rpc_req),
            Some(delivered_tx),
        )
        .unwrap();
    delivered_rx.await.unwrap();
    res_rx
}

async fn expect_pong(res_rx: oneshot::Receiver<Result<Bytes, RpcError>>) {
    let res_data = res_rx.await.unwrap().unwrap();
    match lcs::from_bytes(&res_data).unwrap() {
        HealthCheckerMsg::Pong(_) => {}
        msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
    };
}

async fn expect_disconnect(
    expected_peer_id: PeerId,
    connection_reqs_rx: &mut libra_channel::Receiver<PeerId, ConnectionRequest>,
) {
    let req = connection_reqs_rx.next().await.unwrap();
    let (peer_id, res_tx) = match req {
        ConnectionRequest::DisconnectPeer(peer_id, res_tx) => (peer_id, res_tx),
        _ => panic!("Unexpected PeerManagerRequest: {:?}", req),
    };
    assert_eq!(peer_id, expected_peer_id);
    res_tx.send(Ok(())).unwrap();
}

async fn send_new_peer_notification(
    peer_id: PeerId,
    connection_notifs_tx: &mut conn_status_channel::Sender,
) {
    let (delivered_tx, delivered_rx) = oneshot::channel();
    connection_notifs_tx
        .push_with_feedback(
            peer_id,
            peer_manager::ConnectionStatusNotification::NewPeer(
                peer_id,
                Multiaddr::from_str("/ip6/::1/tcp/8081").unwrap(),
            ),
            Some(delivered_tx),
        )
        .unwrap();
    delivered_rx.await.unwrap();
}

#[test]
fn outbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, _, _, mut connection_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health Checker should attempt to ping the new peer.
        expect_ping_send_ok(&mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn inbound() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (_network_reqs_rx, mut network_notifs_tx, _, mut connection_notifs_tx, _ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Receive ping from peer.
        let res_rx = send_inbound_ping(peer_id, 0, &mut network_notifs_tx).await;

        // HealthChecker should respond with a pong.
        expect_pong(res_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn outbound_failure_permissive() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let ping_failures_tolerated = 10;
    let (mut network_reqs_rx, _, mut connection_reqs_rx, mut connection_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        for _ in 0..=ping_failures_tolerated {
            ticker_tx.send(()).await.unwrap();
            // Health checker should send a ping request which fails.
            expect_ping_send_notok(&mut network_reqs_rx).await;
        }

        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn ping_success_resets_fail_counter() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let failures_triggered = 10;
    let ping_failures_tolerated = 2 * 10;
    let (mut network_reqs_rx, _, mut connection_reqs_rx, mut connection_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;
        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        {
            for _ in 0..failures_triggered {
                ticker_tx.send(()).await.unwrap();
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(&mut network_reqs_rx).await;
            }
        }
        // Trigger successful ping. This should reset the counter of ping failures.
        {
            ticker_tx.send(()).await.unwrap();
            // Health checker should send a ping request which succeeds
            expect_ping_send_ok(&mut network_reqs_rx).await;
        }
        // We would then need to fail for more than `ping_failures_tolerated` times before
        // triggering disconnect.
        {
            for i in 0..=ping_failures_tolerated {
                info!("i: {}", i);
                ticker_tx.send(()).await.unwrap();
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(&mut network_reqs_rx).await;
            }
        }
        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn outbound_failure_strict() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, _, mut connection_reqs_rx, mut connection_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should send a ping request which fails.
        expect_ping_send_notok(&mut network_reqs_rx).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn ping_timeout() {
    ::libra_logger::Logger::new().environment_only(true).init();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, _, mut connection_reqs_rx, mut connection_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should send a ping request which fails.
        expect_ping_timeout(&mut network_reqs_rx).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;
    };
    rt.block_on(events_f);
}
