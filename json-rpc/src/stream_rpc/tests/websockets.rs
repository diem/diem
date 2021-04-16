// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    stream_rpc::{
        json_rpc::JsonRpcResponse,
        tests::util::{
            close_ws, connect_to_ws, get_latest_client, num_clients, num_tasks, verify_ok,
            ws_test_setup,
        },
    },
    tests::utils::create_db_and_runtime,
};
use serde_json::json;
use storage_interface::DbReader;

#[tokio::test]
async fn test_websocket_getting_transactions() {
    let (_db, mut ws_client, cm) = ws_test_setup(5, 10, 300).await;

    let json_str = serde_json::json!({"id": "client-generated-id", "method": "subscribe_to_transactions", "params": {"starting_version": 0}, "jsonrpc": "2.0"}).to_string();
    ws_client.send_text(json_str).await;
    let transaction_version = verify_ok(&mut ws_client).await;

    assert_eq!(num_clients(&cm), 1);
    let client = get_latest_client(&cm);
    assert_eq!(num_tasks(&client), 1);

    let mut current_version = 0;
    for _ in 0..(transaction_version - 1) {
        let msg = ws_client.recv().await.unwrap();
        let resp: JsonRpcResponse = serde_json::from_str(msg.to_str().expect("response")).unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.id.unwrap().to_string(), "\"client-generated-id\"");
        let resp_version = resp
            .result
            .expect("response result")
            .get("version")
            .expect("response result version")
            .as_u64()
            .expect("response result version to u64");
        assert_eq!(current_version, resp_version);
        current_version += 1;
    }

    assert_eq!(current_version, transaction_version - 1);

    close_ws(ws_client).await;

    assert_eq!(num_clients(&cm), 0);
    assert_eq!(num_tasks(&client), 0);
}

#[test]
fn test_websocket_requests() {
    /*
    let (mock_db, _runtime, url, _) = create_db_and_runtime();
    let client = reqwest::blocking::Client::new();
    let version = mock_db.version;
    let timestamp = mock_db.get_block_timestamp(version).unwrap();

    let calls = vec![(
        "get_account_state_with_proof: ledger version is too large",
        json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", null, version+1], "id": 1}),
        json!({
            "error": {
                "code": -32602,
                "message": format!("Invalid param ledger_version should be <= known latest version {}", version),
                "data": null
            },
            "id": 1,
            "jsonrpc": "2.0",
            "diem_ledger_timestampusec": timestamp,
            "diem_ledger_version": version
        }),
    )];
    for (name, request, expected) in calls {
        // connect_to_ws()
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let headers = resp.headers().clone();
        assert_eq!(
            headers
                .get("X-Diem-Ledger-Version")
                .unwrap()
                .to_str()
                .unwrap(),
            version.to_string()
        );
        assert_eq!(
            headers
                .get("X-Diem-Ledger-TimestampUsec")
                .unwrap()
                .to_str()
                .unwrap(),
            timestamp.to_string()
        );

        let resp_json: serde_json::Value = resp.json().unwrap();
        assert_eq!(expected, resp_json, "test: {}", name);
    }
        */
}
