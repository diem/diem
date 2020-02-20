// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::bootstrap;
use libra_config::utils;
use reqwest;
use serde_json;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use storage_service::mocks::mock_storage_client::MockStorageReadClient;

type JsonMap = HashMap<String, serde_json::Value>;

#[test]
fn test_json_rpc_protocol() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(MockStorageReadClient));
    let client = reqwest::blocking::Client::new();

    // check that only root path is accessible
    let url = format!("http://{}/fake_path", address);
    let resp = client.get(&url).send().unwrap();
    assert_eq!(resp.status(), 404);

    // only post method is allowed
    let url = format!("http://{}", address);
    let resp = client.get(&url).send().unwrap();
    assert_eq!(resp.status(), 405);

    // empty payload is not allowed
    let resp = client.post(&url).send().unwrap();
    assert_eq!(resp.status(), 400);

    // non json payload
    let resp = client.post(&url).body("non json").send().unwrap();
    assert_eq!(resp.status(), 400);

    // invalid version of protocol
    let request = serde_json::json!({"jsonrpc": "1.0", "method": "add", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(fetch_error(resp), -32600);

    // invalid request id
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "add", "params": [1, 2], "id": true});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(fetch_error(resp), -32600);

    // invalid rpc method
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "add", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(fetch_error(resp), -32601);

    // invalid arguments
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(fetch_error(resp), -32000);
}

#[test]
fn test_get_account_state() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(MockStorageReadClient));
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    // single call
    let account = "0265b97523d042755f9a290fdc6e9febc9a86bdcd03b70c63a0349274e85e263";
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state", "params": [account], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    let data = fetch_result(resp.json().unwrap());
    assert_eq!(data.get("balance").unwrap().as_u64(), Some(100));
    assert_eq!(data.get("sequence_number").unwrap().as_u64(), Some(0));

    // batching call
    let accounts = vec![
        "1ad0e745c00a6ef4a8176ddb1a5e2fdc7bc92f6ac3a7955b3aa0e3211074174a",
        "f9841c24cf884559e79f20eb9926902eb4a072d1d78d40f11588fbc941d82150",
        "bc1330c1c4176f50c82c7a1b9e5295eaf1eeec87a627735f4d446a55d5844ead",
    ];
    let requests = accounts
        .iter()
        .enumerate()
        .map(|(id, address)|
            serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state", "params": [address], "id": id})
        )
        .collect();
    let resp = client
        .post(&url)
        .json(&serde_json::Value::Array(requests))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 200);
    match resp.json().unwrap() {
        serde_json::Value::Array(responses) => {
            let mut ids = HashSet::new();
            for response in responses {
                ids.insert(response.get("id").unwrap().as_u64().unwrap());
                let result: JsonMap = serde_json::from_value(
                    response.as_object().unwrap().get("result").unwrap().clone(),
                )
                .unwrap();
                assert_eq!(result.get("balance").unwrap().as_u64(), Some(100));
            }
            let ids_expected: HashSet<_> = vec![0, 1, 2].into_iter().collect();
            assert_eq!(ids, ids_expected);
        }
        _ => panic!("invalid server response format"),
    }
}

fn fetch_result(data: JsonMap) -> JsonMap {
    let result: JsonMap = serde_json::from_value(data.get("result").unwrap().clone()).unwrap();
    assert_eq!(
        data.get("jsonrpc").unwrap(),
        &serde_json::Value::String("2.0".to_string())
    );
    result
}

fn fetch_error(resp: reqwest::blocking::Response) -> i16 {
    let data: JsonMap = resp.json().unwrap();
    let error: JsonMap = serde_json::from_value(data.get("error").unwrap().clone()).unwrap();
    assert_eq!(
        data.get("jsonrpc").unwrap(),
        &serde_json::Value::String("2.0".to_string())
    );
    serde_json::from_value(error.get("code").unwrap().clone()).unwrap()
}
