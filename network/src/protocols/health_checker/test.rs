// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::{
    interface::{NetworkNotification, NetworkRequest},
    protocols::rpc::InboundRpcRequest,
    validator_network::HEALTH_CHECKER_RPC_PROTOCOL,
    ProtocolId,
};
use futures::sink::SinkExt;
use prost::Message as _;
use tokio::runtime::Runtime;

const PING_TIMEOUT: Duration = Duration::from_millis(500);

fn setup_permissive_health_checker(
    rt: &mut Runtime,
    ping_failures_tolerated: u64,
) -> (
    channel::Receiver<NetworkRequest>,
    channel::Sender<NetworkNotification>,
    channel::Sender<()>,
) {
    let (ticker_tx, ticker_rx) = channel::new_test(0);

    let (network_reqs_tx, network_reqs_rx) = channel::new_test(0);
    let (network_notifs_tx, network_notifs_rx) = channel::new_test(0);

    let hc_network_tx = HealthCheckerNetworkSender::new(network_reqs_tx);
    let hc_network_rx = HealthCheckerNetworkEvents::new(network_notifs_rx);

    let health_checker = HealthChecker::new(
        ticker_rx,
        hc_network_tx,
        hc_network_rx,
        PING_TIMEOUT,
        ping_failures_tolerated,
    );
    rt.spawn(health_checker.start());
    (network_reqs_rx, network_notifs_tx, ticker_tx)
}

fn setup_strict_health_checker(
    rt: &mut Runtime,
) -> (
    channel::Receiver<NetworkRequest>,
    channel::Sender<NetworkNotification>,
    channel::Sender<()>,
) {
    setup_permissive_health_checker(rt, 0 /* ping_failures_tolerated */)
}

async fn expect_ping(
    network_reqs_rx: &mut channel::Receiver<NetworkRequest>,
) -> (Ping, oneshot::Sender<Result<Bytes, RpcError>>) {
    let req = network_reqs_rx.next().await.unwrap();
    let rpc_req = match req {
        NetworkRequest::SendRpc(_peer_id, rpc_req) => rpc_req,
        _ => panic!("Unexpected NetworkRequest: {:?}", req),
    };

    let protocol = rpc_req.protocol;
    let req_data = rpc_req.data;
    let res_tx = rpc_req.res_tx;

    assert_eq!(
        protocol,
        ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL)
    );

    let req_msg = HealthCheckerMsg::decode(req_data.as_ref()).unwrap();
    match req_msg.message {
        Some(HealthCheckerMsg_oneof::Ping(ping_msg)) => (ping_msg, res_tx),
        _ => panic!("Unexpected HealthCheckerMsg: {:?}", req_msg),
    }
}

async fn expect_ping_send_ok(network_reqs_rx: &mut channel::Receiver<NetworkRequest>) {
    let (ping_msg, res_tx) = expect_ping(network_reqs_rx).await;
    let pong_msg = Pong {
        nonce: ping_msg.nonce,
    };
    let res_msg_enum = HealthCheckerMsg {
        message: Some(HealthCheckerMsg_oneof::Pong(pong_msg)),
    };
    let res_data = res_msg_enum.to_bytes().unwrap();
    res_tx.send(Ok(res_data)).unwrap();
}

async fn expect_ping_send_notok(network_reqs_rx: &mut channel::Receiver<NetworkRequest>) {
    let (_ping_msg, res_tx) = expect_ping(network_reqs_rx).await;
    // This mock ping request must fail.
    res_tx.send(Err(RpcError::TimedOut)).unwrap();
}

async fn expect_ping_timeout(network_reqs_rx: &mut channel::Receiver<NetworkRequest>) {
    let (_ping_msg, _res_tx) = expect_ping(network_reqs_rx).await;
    // Sleep for ping timeout plus a little bit.
    std::thread::sleep(PING_TIMEOUT + Duration::from_millis(100));
}

async fn send_inbound_ping(
    peer_id: PeerId,
    ping_msg: Ping,
    network_notifs_tx: &mut channel::Sender<NetworkNotification>,
) -> oneshot::Receiver<Result<Bytes, RpcError>> {
    let protocol = ProtocolId::from_static(HEALTH_CHECKER_RPC_PROTOCOL);
    let req_msg_enum = HealthCheckerMsg {
        message: Some(HealthCheckerMsg_oneof::Ping(ping_msg)),
    };
    let data = req_msg_enum.to_bytes().unwrap();
    let (res_tx, res_rx) = oneshot::channel();
    let inbound_rpc_req = InboundRpcRequest {
        protocol,
        data,
        res_tx,
    };
    network_notifs_tx
        .send(NetworkNotification::RecvRpc(peer_id, inbound_rpc_req))
        .await
        .unwrap();
    res_rx
}

async fn expect_pong(res_rx: oneshot::Receiver<Result<Bytes, RpcError>>) {
    let res_data = res_rx.await.unwrap().unwrap();
    let res_msg = HealthCheckerMsg::decode(res_data.as_ref()).unwrap();
    match res_msg.message {
        Some(HealthCheckerMsg_oneof::Pong(_)) => {}
        _ => panic!("Unexpected HealthCheckerMsg: {:?}", res_msg),
    }
}

async fn expect_disconnect(
    expected_peer_id: PeerId,
    network_reqs_rx: &mut channel::Receiver<NetworkRequest>,
) {
    let req = network_reqs_rx.next().await.unwrap();
    let (peer_id, res_tx) = match req {
        NetworkRequest::DisconnectPeer(peer_id, res_tx) => (peer_id, res_tx),
        _ => panic!("Unexpected NetworkRequest: {:?}", req),
    };
    assert_eq!(peer_id, expected_peer_id);
    res_tx.send(Ok(())).unwrap();
}

#[test]
fn outbound() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, mut network_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();

        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health Checker should attempt to ping the new peer.
        expect_ping_send_ok(&mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn inbound() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (_network_reqs_rx, mut network_notifs_tx, _ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();

        // Receive ping from peer.
        let ping_msg = Ping { nonce: 0 };
        let res_rx = send_inbound_ping(peer_id, ping_msg, &mut network_notifs_tx).await;

        // HealthChecker should respond with a pong.
        expect_pong(res_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn outbound_failure_permissive() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let ping_failures_tolerated = 10;
    let (mut network_reqs_rx, mut network_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();

        // Trigger pings to a peer. These should ping the newly added peer, but not disconnect from
        // it.
        for _ in 0..=ping_failures_tolerated {
            ticker_tx.send(()).await.unwrap();
            // Health checker should send a ping request which fails.
            expect_ping_send_notok(&mut network_reqs_rx).await;
        }

        // Health checker should disconnect from peer after tolerated number of failures
        expect_disconnect(peer_id, &mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn ping_success_resets_fail_counter() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let failures_triggered = 10;
    let ping_failures_tolerated = 2 * 10;
    let (mut network_reqs_rx, mut network_notifs_tx, mut ticker_tx) =
        setup_permissive_health_checker(&mut rt, ping_failures_tolerated);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();
        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();
        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();
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
        expect_disconnect(peer_id, &mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn outbound_failure_strict() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, mut network_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();

        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should send a ping request which fails.
        expect_ping_send_notok(&mut network_reqs_rx).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}

#[test]
fn ping_timeout() {
    ::libra_logger::try_init_for_testing();
    let mut rt = Runtime::new().unwrap();
    let (mut network_reqs_rx, mut network_notifs_tx, mut ticker_tx) =
        setup_strict_health_checker(&mut rt);

    let events_f = async move {
        // Trigger ping to a peer. This should do nothing.
        ticker_tx.send(()).await.unwrap();

        // Notify HealthChecker of new connected node.
        let peer_id = PeerId::random();

        network_notifs_tx
            .send(NetworkNotification::NewPeer(peer_id))
            .await
            .unwrap();

        // Trigger ping to a peer. This should ping the newly added peer.
        ticker_tx.send(()).await.unwrap();

        // Health checker should send a ping request which fails.
        expect_ping_timeout(&mut network_reqs_rx).await;

        // Health checker should disconnect from peer.
        expect_disconnect(peer_id, &mut network_reqs_rx).await;
    };
    rt.block_on(events_f);
}
