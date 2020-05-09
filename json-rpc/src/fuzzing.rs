// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::bootstrap;
use libra_config::utils;
use libra_mempool::mocks::MockSharedMempool;
use libra_proptest_helpers::ValueGenerator;
use libra_temppath::TempPath;
use libra_types::transaction::SignedTransaction;
use libradb::{test_helper::arb_mock_genesis, LibraDB};
use reqwest::blocking::Client;
use std::sync::Arc;
use storage_interface::DbWriter;

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
    let db = LibraDB::new_for_test(&tmp_dir);
    // initialize DB with baseline ledger info - otherwise server will fail on attempting to retrieve initial ledger info
    let (genesis_txn_to_commit, ledger_info_with_sigs) =
        ValueGenerator::new().generate(arb_mock_genesis());
    db.save_transactions(&[genesis_txn_to_commit], 0, Some(&ledger_info_with_sigs))
        .unwrap();

    let port = utils::get_available_port();
    let address = format!("0.0.0.0:{}", port);
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(db), smp.ac_client);

    let client = Client::new();
    let url = format!("http://{}", address);
    let json_request = match serde_json::from_slice::<serde_json::Value>(data) {
        Err(_) => {
            // should not throw error or panic on invalid fuzzer inputs
            if cfg!(test) {
                panic!();
            }
            return;
        }
        Ok(request) => request,
    };

    let response = client
        .post(&url) // address
        .json(&json_request)
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
    if response.get("error").is_some() && cfg!(test) {
        panic!(
            "Unexpected error payload in JSON RPC response: {}",
            response
        );
    }

    // only proceed to check successful response for input from `generate_corpus`
    if !cfg!(test) {
        return;
    }

    let result = response.get("result").unwrap_or_else(|| {
        panic!(
            "Received JSON RPC response with no result payload. Full response: {}",
            response
        )
    });
    let response_id: u64 = serde_json::from_value(
        response
            .get("id")
            .unwrap_or_else(|| panic!("failed to get ID from response: {}", response))
            .clone(),
    )
    .unwrap_or_else(|_| panic!("Failed to deserialize ID from: {}", response));
    assert_eq!(response_id, 1, "mismatch ID in JSON RPC: {}", response);
    assert!(
        *result == serde_json::Value::Null,
        "Received unexpected result payload from txn submission: {:?}",
        response
    );
}
