// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

// use proptest::prelude::*;
use warp::test::WsClient;

use diem_config::config::StreamConfig;

use crate::{
    stream_rpc::{
        connection::{ClientConnection, ConnectionManager},
        json_rpc::JsonRpcResponse,
        transport::{sse::get_sse_routes, websocket::get_websocket_routes},
    },
    tests::utils::{mock_db, MockDiemDB},
};

pub fn make_ws_config(fetch_size: u64, queue_size: usize, poll_interval_ms: u64) -> StreamConfig {
    StreamConfig {
        enabled: true,
        subscription_fetch_size: fetch_size,
        send_queue_size: queue_size,
        poll_interval_ms,
    }
}

pub async fn connect_to_ws(
    db: Arc<MockDiemDB>,
    config: StreamConfig,
) -> (WsClient, ConnectionManager) {
    let (routes, cm) = get_websocket_routes(&config, 1024 * 10, db.clone());
    let ws_client = warp::test::ws()
        .path("/v1/stream/ws")
        .header("user-agent", "diem-client-sdk-python / 0.1.22")
        .header("Content-Length", "500")
        .handshake(routes)
        .await
        .expect("handshake");
    (ws_client, cm)
}

pub async fn ws_test_setup(
    fetch_size: u64,
    queue_size: usize,
    poll_interval_ms: u64,
) -> (Arc<MockDiemDB>, WsClient, ConnectionManager) {
    diem_logger::DiemLogger::init_for_testing();
    let db = Arc::new(mock_db());
    let config = make_ws_config(fetch_size, queue_size, poll_interval_ms);
    let (ws_client, cm) = connect_to_ws(db.clone(), config).await;
    (db, ws_client, cm)
}

pub async fn verify_ok(client: &mut WsClient) -> u64 {
    let msg = client.recv().await.unwrap();
    println!("Message: {:?}", &msg);
    let resp: JsonRpcResponse =
        serde_json::from_str(msg.to_str().expect("ok msg response")).unwrap();
    let res = resp.result.expect("ok response result");
    assert_eq!(
        res.get("status")
            .expect("ok response status")
            .as_str()
            .unwrap(),
        "OK"
    );
    res.get("transaction_version")
        .expect("OK transaction version")
        .as_u64()
        .expect("ok transaction version to u64")
}

pub fn num_clients(cm: &ConnectionManager) -> usize {
    cm.clients.read().len()
}

pub fn get_latest_client(cm: &ConnectionManager) -> ClientConnection {
    cm.get_client((num_clients(cm) - 1) as u64)
        .expect("no clients exist")
}

pub fn num_tasks(client: &ClientConnection) -> usize {
    client.running_tasks.tasks.lock().len()
}

pub async fn close_ws(mut ws_client: WsClient) {
    ws_client.send(warp::ws::Message::close()).await;
    ws_client.recv_closed().await.expect("connection closed");
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
}
