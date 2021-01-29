// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    peer_manager::{
        self, conn_notifs_channel, ConnectionRequest, PeerManagerNotification, PeerManagerRequest,
    },
    protocols::{
        network::{NewNetworkEvents, NewNetworkSender},
        rpc::InboundRpcRequest,
    },
    transport::ConnectionMetadata,
    ProtocolId,
};
use channel::{diem_channel, message_queues::QueueStyle};
use diem_config::{config::RoleType, network_id::NetworkId};
use diem_time_service::{MockTimeService, TimeService};
use futures::{executor::block_on, future};

const PING_INTERVAL: Duration = Duration::from_secs(1);
const PING_TIMEOUT: Duration = Duration::from_millis(500);

fn setup_permissive_health_checker(
    ping_failures_tolerated: u64,
) -> (
    HealthChecker,
    MockTimeService,
    diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    diem_channel::Receiver<PeerId, ConnectionRequest>,
    conn_notifs_channel::Sender,
) {
    let mock_time = TimeService::mock();

    let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
    let (connection_reqs_tx, connection_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
    let (network_notifs_tx, network_notifs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
    let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();

    let hc_network_tx = HealthCheckerNetworkSender::new(
        PeerManagerRequestSender::new(peer_mgr_reqs_tx),
        ConnectionRequestSender::new(connection_reqs_tx),
    );
    let hc_network_rx = HealthCheckerNetworkEvents::new(network_notifs_rx, connection_notifs_rx);
    let network_context =
        NetworkContext::new(NetworkId::Validator, RoleType::Validator, PeerId::ZERO);
    let health_checker = HealthChecker::new(
        Arc::new(network_context),
        mock_time.clone(),
        hc_network_tx,
        hc_network_rx,
        PING_INTERVAL,
        PING_TIMEOUT,
        ping_failures_tolerated,
    );
    (
        health_checker,
        mock_time.into_mock(),
        peer_mgr_reqs_rx,
        network_notifs_tx,
        connection_reqs_rx,
        connection_notifs_tx,
    )
}

fn setup_strict_health_checker() -> (
    HealthChecker,
    MockTimeService,
    diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    diem_channel::Receiver<PeerId, ConnectionRequest>,
    conn_notifs_channel::Sender,
) {
    setup_permissive_health_checker(0 /* ping_failures_tolerated */)
}

async fn expect_ping(
    network_reqs_rx: &mut diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) -> (Ping, oneshot::Sender<Result<Bytes, RpcError>>) {
    let req = network_reqs_rx.next().await.unwrap();
    let rpc_req = match req {
        PeerManagerRequest::SendRpc(_peer_id, rpc_req) => rpc_req,
        _ => panic!("Unexpected PeerManagerRequest: {:?}", req),
    };

    let protocol_id = rpc_req.protocol_id;
    let req_data = rpc_req.data;
    let res_tx = rpc_req.res_tx;

    assert_eq!(protocol_id, ProtocolId::HealthCheckerRpc,);

    match bcs::from_bytes(&req_data).unwrap() {
        HealthCheckerMsg::Ping(ping) => (ping, res_tx),
        msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
    }
}

async fn expect_ping_send_ok(
    network_reqs_rx: &mut diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) {
    let (ping, res_tx) = expect_ping(network_reqs_rx).await;
    let res_data = bcs::to_bytes(&HealthCheckerMsg::Pong(Pong(ping.0))).unwrap();
    res_tx.send(Ok(res_data.into())).unwrap();
}

async fn expect_ping_send_notok(
    network_reqs_rx: &mut diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
) {
    let (_ping_msg, res_tx) = expect_ping(network_reqs_rx).await;
    // This mock ping request must fail.
    res_tx.send(Err(RpcError::TimedOut)).unwrap();
}

async fn send_inbound_ping(
    peer_id: PeerId,
    ping: u32,
    network_notifs_tx: &mut diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
) -> oneshot::Receiver<Result<Bytes, RpcError>> {
    let protocol_id = ProtocolId::HealthCheckerRpc;
    let data = bcs::to_bytes(&HealthCheckerMsg::Ping(Ping(ping)))
        .unwrap()
        .into();
    let (res_tx, res_rx) = oneshot::channel();
    let inbound_rpc_req = InboundRpcRequest {
        protocol_id,
        data,
        res_tx,
    };
    let key = (peer_id, ProtocolId::HealthCheckerRpc);
    let (delivered_tx, delivered_rx) = oneshot::channel();
    network_notifs_tx
        .push_with_feedback(
            key,
            PeerManagerNotification::RecvRpc(peer_id, inbound_rpc_req),
            Some(delivered_tx),
        )
        .unwrap();
    delivered_rx.await.unwrap();
    res_rx
}

async fn expect_pong(res_rx: oneshot::Receiver<Result<Bytes, RpcError>>) {
    let res_data = res_rx.await.unwrap().unwrap();
    match bcs::from_bytes(&res_data).unwrap() {
        HealthCheckerMsg::Pong(_) => {}
        msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
    };
}

async fn expect_disconnect(
    expected_peer_id: PeerId,
    connection_reqs_rx: &mut diem_channel::Receiver<PeerId, ConnectionRequest>,
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
    connection_notifs_tx: &mut conn_notifs_channel::Sender,
) {
    let (delivered_tx, delivered_rx) = oneshot::channel();
    let notif = peer_manager::ConnectionNotification::NewPeer(
        ConnectionMetadata::mock(peer_id),
        NetworkContext::mock(),
    );
    connection_notifs_tx
        .push_with_feedback(peer_id, notif, Some(delivered_tx))
        .unwrap();
    delivered_rx.await.unwrap();
}

#[test]
fn outbound() {
    ::diem_logger::Logger::init_for_testing();
    let (
        health_checker,
        mock_time,
        mut network_reqs_rx,
        network_notifs_tx,
        _,
        mut connection_notifs_tx,
    ) = setup_strict_health_checker();

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        mock_time.advance_async(PING_INTERVAL).await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        mock_time.advance_async(PING_INTERVAL).await;

        // Health Checker should attempt to ping the new peer.
        expect_ping_send_ok(&mut network_reqs_rx).await;

        // Shutdown Health Checker.
        drop(network_notifs_tx);
        drop(connection_notifs_tx);
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn inbound() {
    ::diem_logger::Logger::init_for_testing();
    let (
        health_checker,
        _mock_time,
        _network_reqs_rx,
        mut network_notifs_tx,
        _,
        mut connection_notifs_tx,
    ) = setup_strict_health_checker();

    let test = async move {
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Receive ping from peer.
        let res_rx = send_inbound_ping(peer_id, 0, &mut network_notifs_tx).await;

        // HealthChecker should respond with a pong.
        expect_pong(res_rx).await;

        // Shutdown Health Checker.
        drop(network_notifs_tx);
        drop(connection_notifs_tx);
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn outbound_failure_permissive() {
    ::diem_logger::Logger::init_for_testing();
    let ping_failures_tolerated = 10;
    let (
        health_checker,
        mock_time,
        mut network_reqs_rx,
        network_notifs_tx,
        mut connection_reqs_rx,
        mut connection_notifs_tx,
    ) = setup_permissive_health_checker(ping_failures_tolerated);

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        mock_time.advance_async(PING_INTERVAL).await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        for _ in 0..=ping_failures_tolerated {
            mock_time.advance_async(PING_INTERVAL).await;
            // Health checker should send a ping request which fails.
            expect_ping_send_notok(&mut network_reqs_rx).await;
        }

        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;

        // Shutdown Health Checker.
        drop(network_notifs_tx);
        drop(connection_notifs_tx);
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn ping_success_resets_fail_counter() {
    ::diem_logger::Logger::init_for_testing();
    let failures_triggered = 10;
    let ping_failures_tolerated = 2 * 10;
    let (
        health_checker,
        mock_time,
        mut network_reqs_rx,
        network_notifs_tx,
        mut connection_reqs_rx,
        mut connection_notifs_tx,
    ) = setup_permissive_health_checker(ping_failures_tolerated);

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        mock_time.advance_async(PING_INTERVAL).await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        {
            for _ in 0..failures_triggered {
                mock_time.advance_async(PING_INTERVAL).await;
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(&mut network_reqs_rx).await;
            }
        }

        // Trigger successful ping. This should reset the counter of ping failures.
        {
            mock_time.advance_async(PING_INTERVAL).await;
            // Health checker should send a ping request which succeeds
            expect_ping_send_ok(&mut network_reqs_rx).await;
        }

        // We would then need to fail for more than `ping_failures_tolerated` times before
        // triggering disconnect.
        {
            for _ in 0..=ping_failures_tolerated {
                mock_time.advance_async(PING_INTERVAL).await;
                // Health checker should send a ping request which fails.
                expect_ping_send_notok(&mut network_reqs_rx).await;
            }
        }

        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;

        // Shutdown Health Checker.
        drop(network_notifs_tx);
        drop(connection_notifs_tx);
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn outbound_failure_strict() {
    ::diem_logger::Logger::init_for_testing();
    let (
        health_checker,
        mock_time,
        mut network_reqs_rx,
        network_notifs_tx,
        mut connection_reqs_rx,
        mut connection_notifs_tx,
    ) = setup_strict_health_checker();

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        mock_time.advance_async(PING_INTERVAL).await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        send_new_peer_notification(peer_id, &mut connection_notifs_tx).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        mock_time.advance_async(PING_INTERVAL).await;

        // Health checker should send a ping request which fails.
        expect_ping_send_notok(&mut network_reqs_rx).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut connection_reqs_rx).await;

        // Shutdown Health Checker.
        drop(network_notifs_tx);
        drop(connection_notifs_tx);
    };
    block_on(future::join(health_checker.start(), test));
}
