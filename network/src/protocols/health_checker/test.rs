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
use diem_time_service::{MockTimeService, TimeService};
use futures::{executor::block_on, future};

const PING_INTERVAL: Duration = Duration::from_secs(1);
const PING_TIMEOUT: Duration = Duration::from_millis(500);

struct TestHarness {
    mock_time: MockTimeService,
    peer_mgr_reqs_rx: diem_channel::Receiver<(PeerId, ProtocolId), PeerManagerRequest>,
    peer_mgr_notifs_tx: diem_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>,
    connection_reqs_rx: diem_channel::Receiver<PeerId, ConnectionRequest>,
    connection_notifs_tx: conn_notifs_channel::Sender,
}

impl TestHarness {
    fn new_permissive(ping_failures_tolerated: u64) -> (Self, HealthChecker) {
        ::diem_logger::Logger::init_for_testing();
        let mock_time = TimeService::mock();

        let (peer_mgr_reqs_tx, peer_mgr_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
        let (connection_reqs_tx, connection_reqs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
        let (peer_mgr_notifs_tx, peer_mgr_notifs_rx) = diem_channel::new(QueueStyle::FIFO, 1, None);
        let (connection_notifs_tx, connection_notifs_rx) = conn_notifs_channel::new();

        let hc_network_tx = HealthCheckerNetworkSender::new(
            PeerManagerRequestSender::new(peer_mgr_reqs_tx),
            ConnectionRequestSender::new(connection_reqs_tx),
        );
        let hc_network_rx =
            HealthCheckerNetworkEvents::new(peer_mgr_notifs_rx, connection_notifs_rx);
        let health_checker = HealthChecker::new(
            NetworkContext::mock(),
            mock_time.clone(),
            hc_network_tx,
            hc_network_rx,
            PING_INTERVAL,
            PING_TIMEOUT,
            ping_failures_tolerated,
        );

        (
            Self {
                mock_time: mock_time.into_mock(),
                peer_mgr_reqs_rx,
                peer_mgr_notifs_tx,
                connection_reqs_rx,
                connection_notifs_tx,
            },
            health_checker,
        )
    }

    fn new_strict() -> (Self, HealthChecker) {
        Self::new_permissive(0 /* ping_failures_tolerated */)
    }

    async fn trigger_ping(&self) {
        self.mock_time.advance_async(PING_INTERVAL).await;
    }

    async fn expect_ping(&mut self) -> (Ping, oneshot::Sender<Result<Bytes, RpcError>>) {
        let req = self.peer_mgr_reqs_rx.next().await.unwrap();
        let rpc_req = match req {
            PeerManagerRequest::SendRpc(_peer_id, rpc_req) => rpc_req,
            _ => panic!("Unexpected PeerManagerRequest: {:?}", req),
        };

        let protocol_id = rpc_req.protocol_id;
        let req_data = rpc_req.data;
        let res_tx = rpc_req.res_tx;

        assert_eq!(protocol_id, ProtocolId::HealthCheckerRpc);

        match bcs::from_bytes(&req_data).unwrap() {
            HealthCheckerMsg::Ping(ping) => (ping, res_tx),
            msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
        }
    }

    async fn expect_ping_send_ok(&mut self) {
        let (ping, res_tx) = self.expect_ping().await;
        let res_data = bcs::to_bytes(&HealthCheckerMsg::Pong(Pong(ping.0))).unwrap();
        res_tx.send(Ok(res_data.into())).unwrap();
    }

    async fn expect_ping_send_not_ok(&mut self) {
        let (_ping_msg, res_tx) = self.expect_ping().await;
        // This mock ping request must fail.
        res_tx.send(Err(RpcError::TimedOut)).unwrap();
    }

    async fn send_inbound_ping(
        &mut self,
        peer_id: PeerId,
        ping: u32,
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
        self.peer_mgr_notifs_tx
            .push_with_feedback(
                key,
                PeerManagerNotification::RecvRpc(peer_id, inbound_rpc_req),
                Some(delivered_tx),
            )
            .unwrap();
        delivered_rx.await.unwrap();
        res_rx
    }

    async fn expect_disconnect(&mut self, expected_peer_id: PeerId) {
        let req = self.connection_reqs_rx.next().await.unwrap();
        let (peer_id, res_tx) = match req {
            ConnectionRequest::DisconnectPeer(peer_id, res_tx) => (peer_id, res_tx),
            _ => panic!("Unexpected ConnectionRequest: {:?}", req),
        };
        assert_eq!(peer_id, expected_peer_id);
        res_tx.send(Ok(())).unwrap();
    }

    async fn send_new_peer_notification(&mut self, peer_id: PeerId) {
        let (delivered_tx, delivered_rx) = oneshot::channel();
        let notif = peer_manager::ConnectionNotification::NewPeer(
            ConnectionMetadata::mock(peer_id),
            NetworkContext::mock(),
        );
        self.connection_notifs_tx
            .push_with_feedback(peer_id, notif, Some(delivered_tx))
            .unwrap();
        delivered_rx.await.unwrap();
    }
}

async fn expect_pong(res_rx: oneshot::Receiver<Result<Bytes, RpcError>>) {
    let res_data = res_rx.await.unwrap().unwrap();
    match bcs::from_bytes(&res_data).unwrap() {
        HealthCheckerMsg::Pong(_) => {}
        msg => panic!("Unexpected HealthCheckerMsg: {:?}", msg),
    };
}

#[test]
fn outbound() {
    let (mut harness, health_checker) = TestHarness::new_strict();

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        harness.trigger_ping().await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::new([0x42; PeerId::LENGTH]);
        harness.send_new_peer_notification(peer_id).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        harness.trigger_ping().await;

        // Health Checker should attempt to ping the new peer.
        harness.expect_ping_send_ok().await;
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn inbound() {
    let (mut harness, health_checker) = TestHarness::new_strict();

    let test = async move {
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::new([0x42; PeerId::LENGTH]);
        harness.send_new_peer_notification(peer_id).await;

        // Receive ping from peer.
        let res_rx = harness.send_inbound_ping(peer_id, 0).await;

        // HealthChecker should respond with a pong.
        expect_pong(res_rx).await;
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn outbound_failure_permissive() {
    let ping_failures_tolerated = 10;
    let (mut harness, health_checker) = TestHarness::new_permissive(ping_failures_tolerated);

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        harness.trigger_ping().await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::new([0x42; PeerId::LENGTH]);
        harness.send_new_peer_notification(peer_id).await;

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        for _ in 0..=ping_failures_tolerated {
            // Health checker should send a ping request which fails.
            harness.trigger_ping().await;
            harness.expect_ping_send_not_ok().await;
        }

        // Health checker should disconnect from peer after tolerated number of failures
        harness.expect_disconnect(peer_id).await;
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn ping_success_resets_fail_counter() {
    let failures_triggered = 10;
    let ping_failures_tolerated = 2 * 10;
    let (mut harness, health_checker) = TestHarness::new_permissive(ping_failures_tolerated);

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        harness.trigger_ping().await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::new([0x42; PeerId::LENGTH]);
        harness.send_new_peer_notification(peer_id).await;

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        {
            for _ in 0..failures_triggered {
                // Health checker should send a ping request which fails.
                harness.trigger_ping().await;
                harness.expect_ping_send_not_ok().await;
            }
        }

        // Trigger successful ping. This should reset the counter of ping failures.
        {
            // Health checker should send a ping request which succeeds
            harness.trigger_ping().await;
            harness.expect_ping_send_ok().await;
        }

        // We would then need to fail for more than `ping_failures_tolerated` times before
        // triggering disconnect.
        {
            for _ in 0..=ping_failures_tolerated {
                // Health checker should send a ping request which fails.
                harness.trigger_ping().await;
                harness.expect_ping_send_not_ok().await;
            }
        }

        // Health checker should disconnect from peer after tolerated number of failures
        harness.expect_disconnect(peer_id).await;
    };
    block_on(future::join(health_checker.start(), test));
}

#[test]
fn outbound_failure_strict() {
    let (mut harness, health_checker) = TestHarness::new_strict();

    let test = async move {
        // Trigger ping to a peer. This should do nothing.
        harness.trigger_ping().await;

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::new([0x42; PeerId::LENGTH]);
        harness.send_new_peer_notification(peer_id).await;

        // Trigger ping to a peer. This should ping the newly added peer.
        harness.trigger_ping().await;

        // Health checker should send a ping request which fails.
        harness.expect_ping_send_not_ok().await;

        // Health checker should disconnect from peer.
        harness.expect_disconnect(peer_id).await;
    };
    block_on(future::join(health_checker.start(), test));
}
