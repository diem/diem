// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::bootstrap;
use libra_config::utils;
use libra_mempool::mocks::MockSharedMempool;
use libra_proptest_helpers::ValueGenerator;
use libra_temppath::TempPath;
use libra_types::transaction::SignedTransaction;
use libradb::LibraDB;
use reqwest::blocking::Client;
use std::sync::Arc;

#[test]
fn test_json_rpc_service_fuzzer() {
    let mut gen = ValueGenerator::new();
    let data = generate_corpus(&mut gen);
    fuzzer(&data);
}

/// generate_corpus produces an arbitrary transaction to submit to JSON RPC service
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    // use proptest to generate a SignedTransaction
    let txn = gen.generate(proptest::arbitrary::any::<SignedTransaction>());
    let payload = hex::encode(lcs::to_bytes(&txn).unwrap());
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "submit", "params": [payload], "id": 1});
    serde_json::to_vec(&request).expect("failed to convert JSON to byte array")
}

pub fn fuzzer(data: &[u8]) {
    // set up mock Shared Mempool
    let smp = MockSharedMempool::new(None);

    // set up JSON RPC service
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let port = utils::get_available_port();
    let address = format!("0.0.0.0:{}", port);
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(db), smp.ac_client);

    let client = Client::new();
    let url = format!("http://{}", address);
    let request: serde_json::Value =
        serde_json::from_slice(data).expect("failed to convert byte array to JSON");
    let response = client
        .post(&url) // address
        .json(&request)
        .send()
        .expect("failed to send request to JSON RPC server");

    let response: serde_json::Value = response.json().expect("failed to convert response to JSON");

    let json_rpc_protocol = response.get("jsonrpc");
    assert_eq!(
        json_rpc_protocol,
        Some(&serde_json::Value::String("2.0".to_string())),
        "JSON RPC response with incorrect protocol: {:?}",
        json_rpc_protocol
    );
    assert!(
        response.get("error").is_none(),
        "expected no error in JSON RPC response",
    );

    if let Some(result) = response.get("result") {
        let response_id: u64 = serde_json::from_value(response.get("id").unwrap().clone())
            .expect("failed to get ID from response");
        assert_eq!(response_id, 1, "mismatch ID in JSON RPC");
        assert!(
            *result == serde_json::Value::Null,
            "Received unexpected result payload from txn submission: {:?}",
            result
        );
    } else {
        panic!("Received JSON RPC response with no result payload");
    }
}
