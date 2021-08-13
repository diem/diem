// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    stream_rpc::tests::util::{
        close_ws, connect_to_ws, get_latest_client, next_message, num_clients, num_tasks, timeout,
        verify_ok, ws_test_setup,
    },
    tests::utils::create_db_and_runtime,
};
use diem_json_rpc_types::stream::response::StreamJsonRpcResponse;
use futures::{SinkExt, StreamExt};
use reqwest::Url;
use serde_json::json;
use std::convert::TryFrom;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{http, Message},
};

#[tokio::test]
async fn test_websocket_call_and_response() {
    let (mock_db, config) = ws_test_setup(5, 10, 100, 1000).await;

    let calls = vec![
        (
            "subscribe_to_transactions: invalid request",
            json!({"id": "client-generated-id", "method": "subscribe_to_transactions", "params": {"starting_version": "not a number"}, "jsonrpc": "2.0"}).to_string(),
            json!({"jsonrpc":"2.0","id":"client-generated-id","error":{"code":-32602,"message":"Invalid params for method 'subscribe_to_transactions'","data":null}}).to_string(),
        ),
        (
            "subscribe_to_events: get events data",
            json!({"id": "client-generated-id", "method": "subscribe_to_events", "params": {"event_key": 1337}, "jsonrpc": "2.0"}).to_string(),
            json!({"jsonrpc":"2.0","id":"client-generated-id","error":{"code":-32602,"message":"Invalid params for method 'subscribe_to_events'","data":null}}).to_string(),
        ),
    ];

    for (name, request, expected) in calls {
        let (mut ws_client, cm) = connect_to_ws(mock_db.clone(), &config, None).await;
        assert_eq!(num_clients(&cm), 1);

        ws_client.send_text(request).await;
        let result = next_message(&mut ws_client, name).await;
        assert_eq!(result.to_str().unwrap(), expected);
    }
}

#[tokio::test]
async fn test_invalid_params() {
    let (mock_db, config) = ws_test_setup(5, 10, 100, 1000).await;

    let endpoint_names = vec!["subscribe_to_transactions", "subscribe_to_events"];

    for endpoint_name in endpoint_names {
        let name = format!("{}: invalid param", &endpoint_name);
        let request = json!({"id": "client-generated-id", "method": &endpoint_name, "params": {"not_a_param": 0}, "jsonrpc": "2.0"}).to_string();
        let expected = json!({"jsonrpc":"2.0","id":"client-generated-id","error":{"code":-32602,"message":format!("Invalid params for method '{}'", &endpoint_name),"data":null}}).to_string();
        let (mut ws_client, _cm) = connect_to_ws(mock_db.clone(), &config, None).await;
        ws_client.send_text(request).await;
        let result = next_message(&mut ws_client, &name).await;
        assert_eq!(result.to_str().unwrap(), expected);

        let name = format!("{}: malformed json", &endpoint_name);
        let request = "{\"id\": \"client-generated-id\" ";
        let expected = json!({"jsonrpc":"2.0","error":{"code":-32604,"message":"Invalid request format","data":null}}).to_string();
        let (mut ws_client, _cm) = connect_to_ws(mock_db.clone(), &config, None).await;
        ws_client.send_text(request).await;
        let result = next_message(&mut ws_client, &name).await;
        assert_eq!(result.to_str().unwrap(), expected);
    }
}

#[tokio::test]
async fn test_websocket_fetching_data() {
    let (mock_db, config) = ws_test_setup(5, 10, 100, 1000).await;

    let (_, ev) = mock_db.events.get(0).unwrap();
    let first_event_key = ev.key().to_string();
    let num_events = mock_db
        .events
        .iter()
        .filter(|(_, ev)| ev.key().to_string() == first_event_key)
        .count();

    let calls = vec![
        (
            "subscribe_to_transactions: get transaction data",
            json!({"id": "client-generated-id", "method": "subscribe_to_transactions", "params": {"starting_version": 0}, "jsonrpc": "2.0"}),
            mock_db.all_txns.len(),
        ),
        (
            "subscribe_to_events: get events data",
            json!({"id": "client-generated-id", "method": "subscribe_to_events", "params": {"event_key": first_event_key, "event_seq_num": 0}, "jsonrpc": "2.0"}),
            num_events,
        ),
    ];

    for (name, request, expected_number) in calls {
        let (mut ws_client, cm) = connect_to_ws(mock_db.clone(), &config, None).await;
        assert_eq!(num_clients(&cm), 1);

        ws_client.send_text(request.to_string()).await;
        let sub_result = next_message(&mut ws_client, name).await;
        let _transaction_version = verify_ok(sub_result, name);

        let client = get_latest_client(&cm);
        assert_eq!(num_tasks(&client), 1);

        for i in 0..(expected_number) {
            let msg = next_message(&mut ws_client, &format!("{} get message {}", &name, i)).await;
            let resp: StreamJsonRpcResponse =
                serde_json::from_str(msg.to_str().expect("response")).unwrap();
            assert!(resp.error.is_none());
            assert_eq!(resp.id.unwrap().to_string(), "\"client-generated-id\"");
        }

        close_ws(ws_client, name).await;

        assert_eq!(num_clients(&cm), 0);
    }
}

#[tokio::test]
async fn test_multiple_subscriptions_and_response() {
    let (mock_db, config) = ws_test_setup(5, 10, 100, 1000).await;

    let calls = vec![
        (
            "subscribe_to_transactions: invalid request",
            json!({"id": "client-generated-id", "method": "subscribe_to_transactions", "params": {"starting_version": "not a number"}, "jsonrpc": "2.0"}).to_string(),
            json!({"jsonrpc":"2.0","id":"client-generated-id","error":{"code":-32602,"message":"Invalid params for method 'subscribe_to_transactions'","data":null}}).to_string(),
        ),
        (
            "subscribe_to_events: get events data",
            json!({"id": "client-generated-id", "method": "subscribe_to_events", "params": {"event_key": 1337}, "jsonrpc": "2.0"}).to_string(),
            json!({"jsonrpc":"2.0","id":"client-generated-id","error":{"code":-32602,"message":"Invalid params for method 'subscribe_to_events'","data":null}}).to_string(),
        ),
    ];

    for (name, request, expected) in calls {
        let (mut ws_client, cm) = connect_to_ws(mock_db.clone(), &config, None).await;
        assert_eq!(num_clients(&cm), 1);

        ws_client.send_text(request).await;
        let result = next_message(&mut ws_client, name).await;
        assert_eq!(result.to_str().unwrap(), expected);
    }
}

#[tokio::test]
async fn test_websocket_can_connect_and_disconnect() {
    let (mock_db, config) = ws_test_setup(5, 10, 100, 1000).await;
    let (ws_client1, cm) = connect_to_ws(mock_db.clone(), &config, None).await;
    assert_eq!(num_clients(&cm), 1);
    let (ws_client2, _) = connect_to_ws(mock_db.clone(), &config, Some(cm.clone())).await;
    assert_eq!(num_clients(&cm), 2);
    close_ws(ws_client1, "").await;
    assert_eq!(num_clients(&cm), 1);
    close_ws(ws_client2, "").await;
    assert_eq!(num_clients(&cm), 0);
}

/// This is heavier test as it spins up a fullnode
/// It exists primarily to ensure that the warp routing is set up correctly,
/// and the stream endpoint is functional (i.e http->ws upgrade).
/// The test can not be async as `create_db_and_runtime` spawns its own runtime,
/// which conflicts with the one `#[tokio::test]` attempts to spawn.
#[test]
fn test_websocket_requests() {
    let (mock_db, runtime, url, _) = create_db_and_runtime();

    let parsed = Url::parse(&url).expect("Invalid websocket address");
    let wss_url = format!(
        "ws://{}:{}/v1/stream/ws",
        parsed.host().expect("Could not parse host"),
        parsed.port().expect("Could not parse port")
    );

    let calls = vec![(
        "subscribe_to_transactions: can subscribe to transactions",
        json!({"id": "client-generated-id-1", "method": "subscribe_to_transactions", "params": {"starting_version": 0}, "jsonrpc": "2.0"}),
        json!({"jsonrpc": "2.0", "id": "client-generated-id-1", "result": {"status": "OK", "transaction_version": mock_db.version}}),
    )];

    let _guard = runtime.enter();

    for (name, request, expected) in calls {
        let http_request = http::Request::builder()
            .method(http::method::Method::GET)
            .uri(http::uri::Uri::try_from(&wss_url).unwrap())
            .header("user-agent", "diem-client-sdk-python / 0.1.22")
            .header("Content-Length", "1500")
            .body(())
            .unwrap();

        let ws_stream = runtime.handle().block_on(timeout(
            500,
            async move {
                let (ws_stream, _) = connect_async(http_request)
                    .await
                    .expect("Could not create websocket connection");
                ws_stream
            },
            "create ws connection",
        ));

        let (mut tx, mut rx) = ws_stream.split();
        runtime
            .handle()
            .block_on(timeout(
                500,
                tx.send(Message::text(request.to_string())),
                "send request",
            ))
            .unwrap_or_else(|e| panic!("{}: Could not send websocket request. {:?}", &name, e));

        let response = runtime
            .handle()
            .block_on(timeout(500, rx.next(), "get next message"))
            .unwrap_or_else(|| panic!("{}: Expected 'Some' response", &name))
            .unwrap_or_else(|_| panic!("{}: Expected no error response", &name))
            .to_string();

        assert_eq!(
            serde_json::to_string(
                &serde_json::from_str::<StreamJsonRpcResponse>(&response).unwrap()
            )
            .unwrap(),
            expected.to_string(),
            "test: {}",
            name
        );
    }
}
