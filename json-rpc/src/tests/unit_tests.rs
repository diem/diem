// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::ServerCode,
    runtime::check_latest_ledger_info_timestamp,
    tests::utils::{
        create_database_client_and_runtime, create_db_and_runtime, mock_db, test_bootstrap,
        MockDiemDB,
    },
    util::{sdk_info_from_user_agent, SdkInfo, SdkLang, SdkVersion},
    views::VMStatusView,
};
use diem_client::{views::TransactionDataView, BlockingClient, MethodRequest};
use diem_config::{config::DEFAULT_CONTENT_LENGTH_LIMIT, utils};
use diem_crypto::{ed25519::Ed25519PrivateKey, hash::CryptoHash, HashValue, PrivateKey, Uniform};
use diem_metrics::get_all_metrics;
use diem_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    chain_id::ChainId,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    proof::{SparseMerkleProof, TransactionAccumulatorProof, TransactionInfoWithProof},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, TransactionInfo, TransactionPayload},
    vm_status::StatusCode,
};
use futures::{channel::mpsc::channel, StreamExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::{
    cmp::{max, min},
    convert::TryFrom,
    ops::Sub,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use storage_interface::DbReader;
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

use serde_json::json;

#[test]
fn test_cors() {
    let (_mock_db, _runtime, url, _) = create_db_and_runtime();

    let origin = "test";

    let client = reqwest::blocking::Client::new();
    let request = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("origin", origin)
        .header("access-control-headers", "content-type")
        .header("access-control-request-method", "POST");
    let resp = request.send().unwrap();
    assert_eq!(resp.status(), 200);

    let cors_header = resp.headers().get("access-control-allow-origin").unwrap();
    assert_eq!(cors_header, origin);
}

#[test]
fn test_json_rpc_http_errors() {
    let (_mock_db, _runtime, url, _) = create_db_and_runtime();

    let client = reqwest::blocking::Client::new();

    // check that only root path is accessible
    let fake_path = format!("{}/fake_path", url);
    let resp = client.get(&fake_path).send().unwrap();
    assert_eq!(resp.status(), 404);

    // only post method is allowed
    let resp = client.get(&url).send().unwrap();
    assert_eq!(resp.status(), 405, "{}", url);

    // empty payload is not allowed
    let resp = client.post(&url).send().unwrap();
    assert_eq!(resp.status(), 400);

    // For now /v1 and / are both supported
    let url_v1 = format!("{}/v1", url);
    let resp = client.post(&url_v1).send().unwrap();
    assert_eq!(resp.status(), 400);

    let resp = client.post(&url).body("non json").send().unwrap();
    assert_eq!(resp.status(), 400);

    let resp = client
        .post(&url)
        .json(&json!({ "id": gen_string(DEFAULT_CONTENT_LENGTH_LIMIT) }))
        .send()
        .unwrap();
    assert_eq!(resp.status(), 413); // 413 is Payload Too Large
}

#[test]
fn test_json_rpc_protocol_invalid_requests() {
    let (mock_db, _runtime, url, _) = create_db_and_runtime();
    let client = reqwest::blocking::Client::new();
    let version = mock_db.version;
    let timestamp = mock_db.get_block_timestamp(version).unwrap();
    let calls = vec![
        (
            "invalid protocol version",
            json!({"jsonrpc": "1.0", "method": "get_metadata", "id": 1}),
            json!({
                "error": {
                    "code": -32600, "data": null, "message": "Invalid Request",
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "invalid request format: invalid id type",
            json!({"jsonrpc": "2.0", "method": "get_metadata", "id": true}),
            json!({
                "error": {
                    "code": -32604, "data": null, "message": "Invalid request format",
                },
                "id": null,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "invalid request format: request is not an object",
            json!(true),
            json!({
                "error": {
                    "code": -32604, "data": null, "message": "Invalid request format",
                },
                "id": null,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "method not found",
            json!({"jsonrpc": "2.0", "method": "add", "id": 1}),
            json!({
                "error": {
                    "code": -32601, "data": null, "message": "Method not found",
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "method not given",
            json!({"jsonrpc": "2.0", "id": 1}),
            json!({
                "error": {
                    "code": -32601, "data": null, "message": "Method not found",
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "jsonrpc not given",
            json!({"method": "get_metadata", "id": 1}),
            json!({
                "error": {
                    "code": -32600, "data": null, "message": "Invalid Request",
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "invalid arguments: too many arguments",
            json!({"jsonrpc": "2.0", "method": "get_currencies", "params": [1, 2], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_currencies'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "invalid arguments: not enough arguments",
            json!({"jsonrpc": "2.0", "method": "get_events", "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "invalid arguments: too many arguments for a method has optional arguments",
            json!({"jsonrpc": "2.0", "method": "get_metadata", "params": [1, 2], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_metadata'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_metadata invalid arguments: version is too large",
            json!({"jsonrpc": "2.0", "method": "get_metadata", "params": [version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account invalid param data type",
            json!({"jsonrpc": "2.0", "method": "get_account", "params": [false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account invalid account address str",
            json!({"jsonrpc": "2.0", "method": "get_account", "params": ["helloworld"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account: invalid version param type",
            json!({"jsonrpc": "2.0", "method": "get_account", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", true], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "submit invalid data",
            json!({"jsonrpc": "2.0", "method": "submit", "params": ["helloworld"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'submit'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions: invalid start_version param",
            json!({"jsonrpc": "2.0", "method": "get_transactions", "params": ["helloworld", 1, true], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions: start_version is too big, returns empty array",
            json!({"jsonrpc": "2.0", "method": "get_transactions", "params": [version+1, 1, true], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": []
            }),
        ),
        (
            "get_transactions: invalid limit param",
            json!({"jsonrpc": "2.0", "method": "get_transactions", "params": [1, false, true], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions: invalid include_events param",
            json!({"jsonrpc": "2.0", "method": "get_transactions", "params": [1, 10, "true"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions_with_proofs: invalid start_version param",
            json!({"jsonrpc": "2.0", "method": "get_transactions_with_proofs", "params": ["helloworld", 1, true], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_transactions_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions_with_proofs: start_version is too big, returns empty array",
            json!({"jsonrpc": "2.0", "method": "get_transactions_with_proofs", "params": [version+1, 1, false], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": null
            }),
        ),
        (
            "get_transactions_with_proofs: invalid limit param",
            json!({"jsonrpc": "2.0", "method": "get_transactions_with_proofs", "params": [1, false, false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_transactions_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_transactions_with_proofs: limit is too big",
            json!({"jsonrpc": "2.0", "method": "get_transactions_with_proofs", "params": [1, 1001, false], "id": 1}),
            json!({
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: page size = 1001, exceed limit 1000",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events: invalid event_key type",
            json!({"jsonrpc": "2.0", "method": "get_events", "params": [false, 1, 10], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events: event_key is not hex-encoded string",
            json!({"jsonrpc": "2.0", "method": "get_events", "params": ["helloworld", 1, 10], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events: invalid start param",
            json!({"jsonrpc": "2.0", "method": "get_events", "params": ["13000000000000000000000000000000000000000a550c18", false, 1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events: start param is too big",
            json!({"jsonrpc": "2.0", "method": "get_events", "params": ["13000000000000000000000000000000000000000a550c18", version+1, 1], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": []
            }),
        ),
        (
            "get_events: invalid limit param",
            json!({"jsonrpc": "2.0", "method": "get_events", "params": ["13000000000000000000000000000000000000000a550c18", 1, "invalid"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events_with_proofs: invalid event_key type",
            json!({"jsonrpc": "2.0", "method": "get_events_with_proofs", "params": [false, 1, 10], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events_with_proofs: event_key is not hex-encoded string",
            json!({"jsonrpc": "2.0", "method": "get_events_with_proofs", "params": ["helloworld", 1, 10], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events_with_proofs: invalid start param",
            json!({"jsonrpc": "2.0", "method": "get_events_with_proofs", "params": ["13000000000000000000000000000000000000000a550c18", false, 1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_events_with_proofs: invalid limit param",
            json!({"jsonrpc": "2.0", "method": "get_events_with_proofs", "params": ["13000000000000000000000000000000000000000a550c18", 1, "invalid"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_events_with_proofs'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_event_by_version_with_proof: version is too large",
            json!({"jsonrpc": "2.0", "method": "get_event_by_version_with_proof", "params": ["13000000000000000000000000000000000000000a550c18", version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_resources: malformed_addr",
            json!({"jsonrpc": "2.0", "method": "get_resources", "params": ["0", version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_resources'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
            }),
        ),
        (
            "get_account_transaction: invalid account",
            json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": ["invalid", 1, false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transaction'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transaction: invalid sequence",
            json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", false, false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transaction'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transaction: invalid include_event",
            json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 1, "false"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transaction'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transaction: seq number is too big",
            json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", version+1, false], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": null
            }),
        ),
        (
            "get_account_transactions: invalid account",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions", "params": ["invalid", 1, 2, false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transactions: invalid start param",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", false, 2, false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transactions: start param is too big",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions", "params": ["0000000000000000000000000A550C18", version+1, 2, false], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": []
            }),
        ),
        (
            "get_account_transactions: invalid limit param",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 1, "invalid", false], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transactions: invalid include_event",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 1, 5, "false"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_transactions'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transactions_with_proofs: start param is too big",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions_with_proofs", "params": ["0000000000000000000000000A550C18", version+1, 2, false], "id": 1}),
            json!({
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version,
                "result": {
                    "serialized_txns_with_proofs": []
                }
            }),
        ),
        (
            "get_account_transactions_with_proofs: limit is too large",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions_with_proofs", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 1, 1001, false], "id": 1}),
            json!({
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: page size = 1001, exceed limit 1000",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_transactions_with_proofs: ledger_version is too large",
            json!({"jsonrpc": "2.0", "method": "get_account_transactions_with_proofs", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 1, 5, false, version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param ledger_version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_state_proof: invalid known_version",
            json!({"jsonrpc": "2.0", "method": "get_state_proof", "params": ["invalid"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_state_proof'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_state_proof: version is too large",
            json!({"jsonrpc": "2.0", "method": "get_state_proof", "params": [version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_accumulator_consistency_proof: ledger_version is too large",
            json!({"jsonrpc": "2.0", "method": "get_accumulator_consistency_proof", "params": [5, version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param ledger_version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_accumulator_consistency_proof: client_known_version is greater than ledger_version",
            json!({"jsonrpc": "2.0", "method": "get_accumulator_consistency_proof", "params": [version+1, version], "id": 1}),
            json!({
                "error": {
                    "code": -32600,
                    "message": format!("Invalid Request: client_known_version({}) should be <= ledger_version({})", version+1, version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: invalid account address",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["invalid", 1, 1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_state_with_proof'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: invalid version",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", "invalid", null], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_state_with_proof'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: version > ledger version",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", version, version-1], "id": 1}),
            json!({
                "error": {
                    "code": -32600,
                    "message": format!("Invalid Request: version({}) should <= ledger version({})",version, version-1),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: ledger version is too large",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", 0, version+1], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param ledger_version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: invalid ledger version",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", version, "invalid"], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": "Invalid params for method 'get_account_state_with_proof'",
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
            "get_account_state_with_proof: version is too large",
            json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": ["e1b3d22871989e9fd9dc6814b2f4fc41", version+1, null], "id": 1}),
            json!({
                "error": {
                    "code": -32602,
                    "message": format!("Invalid param version should be <= known latest version {}", version),
                    "data": null
                },
                "id": 1,
                "jsonrpc": "2.0",
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
        (
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
                "diem_chain_id": ChainId::test().id(),
                "diem_ledger_timestampusec": timestamp,
                "diem_ledger_version": version
            }),
        ),
    ];
    for (name, request, expected) in calls {
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let headers = resp.headers().clone();
        assert_eq!(
            headers.get("X-Diem-Chain-Id").unwrap().to_str().unwrap(),
            "4"
        );
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
}

#[test]
fn test_no_params_request_is_valid() {
    let (_mock_db, _runtime, url, _) = create_db_and_runtime();
    let client = reqwest::blocking::Client::new();
    let request = json!({"jsonrpc": "2.0", "method": "get_currencies", "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    let resp_json: serde_json::Value = resp.json().unwrap();
    assert!(resp_json.get("result").is_some(), "{}", resp_json);
    assert!(resp_json.get("error").is_none(), "{}", resp_json);
}

#[test]
fn test_metrics() {
    let (_mock_db, _runtime, url, _) = create_db_and_runtime();
    let calls = vec![
        (
            "success single call",
            json!({"jsonrpc": "2.0", "method": "get_currencies", "id": 1}),
        ),
        (
            "success batch call",
            json!([
                {"jsonrpc": "2.0", "method": "get_currencies", "id": 1},
                {"jsonrpc": "2.0", "method": "get_currencies", "id": 2}
            ]),
        ),
        (
            "invalid param call",
            json!({"jsonrpc": "2.0", "method": "get_currencies", "params": ["invalid"], "id": 1}),
        ),
    ];
    let client = reqwest::blocking::ClientBuilder::new()
        .user_agent("diem-client-sdk-python / 2.11.15")
        .build()
        .expect("Client::new()");
    for (_name, request) in calls {
        let _ = client.post(&url).json(&request).send();
    }

    let metrics = get_all_metrics();
    let expected_metrics = vec![
        // rpc request count
        "diem_client_service_rpc_requests_count{type=single}",
        "diem_client_service_rpc_requests_count{type=batch}",
        // rpc request latency
        "diem_client_service_rpc_request_latency_seconds{type=single}",
        "diem_client_service_rpc_request_latency_seconds{type=batch}",
        // method request count
        "diem_client_service_requests_count{method=get_currencies,result=success,sdk_lang=python,sdk_ver=2.11.15,type=single}",
        // method latency
        "diem_client_service_method_latency_seconds{method=get_currencies,type=single}",
        "diem_client_service_method_latency_seconds{method=get_currencies,type=batch}",
        // invalid params
        "diem_client_service_invalid_requests_count{errortype=invalid_params,method=get_currencies,sdk_lang=python,sdk_ver=2.11.15,type=single}",
    ];

    for name in expected_metrics {
        assert!(
            metrics.contains_key(name),
            "metrics {} not found in {:?}",
            name,
            metrics
        );
    }
}

#[test]
fn test_transaction_submission() {
    let (mp_sender, mut mp_events) = channel(1);
    let mock_db = mock_db();
    let port = utils::get_available_port();
    let address = format!("0.0.0.0:{}", port);
    let runtime = test_bootstrap(address.parse().unwrap(), Arc::new(mock_db), mp_sender);
    let client = BlockingClient::new(format!("http://127.0.0.1:{}/v1", port));

    // future that mocks shared mempool execution
    runtime.spawn(async move {
        let validator = MockVMValidator;
        while let Some((txn, cb)) = mp_events.next().await {
            let vm_status = validator.validate_transaction(txn).unwrap().status();
            let result = if vm_status.is_some() {
                (MempoolStatus::new(MempoolStatusCode::VmError), vm_status)
            } else {
                (MempoolStatus::new(MempoolStatusCode::Accepted), None)
            };
            cb.send(Ok(result)).unwrap();
        }
    });

    // closure that checks transaction submission for given account
    let txn_submission = move |sender| {
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let txn = get_test_signed_txn(sender, 0, &privkey, privkey.public_key(), None);
        client.submit(&txn)
    };

    // check successful submission
    let sender = AccountAddress::new([9; AccountAddress::LENGTH]);
    txn_submission(sender).unwrap();

    // check vm error submission
    let sender = AccountAddress::new([0; AccountAddress::LENGTH]);
    let response = txn_submission(sender);

    let error = response.unwrap_err();
    let error = error.json_rpc_error().unwrap();
    assert_eq!(error.code, ServerCode::VmValidationError as i16);
    let status_code: StatusCode = error.as_status_code().unwrap();
    assert_eq!(status_code, StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST);
}

#[test]
fn test_get_account() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    // test case 1: single call
    let (first_account, blob) = mock_db.all_accounts.iter().next().unwrap();
    let expected_resource = AccountState::try_from(blob).unwrap();

    let account = client
        .get_account(*first_account)
        .unwrap()
        .into_inner()
        .unwrap();
    let account_balances: Vec<_> = account.balances.iter().map(|bal| bal.amount).collect();
    let expected_resource_balances: Vec<_> = expected_resource
        .get_balance_resources()
        .unwrap()
        .iter()
        .map(|(_, bal_resource)| bal_resource.coin())
        .collect();
    assert_eq!(account.address, *first_account);
    assert_eq!(account_balances, expected_resource_balances);
    assert_eq!(
        account.sequence_number,
        expected_resource
            .get_account_resource()
            .unwrap()
            .unwrap()
            .sequence_number()
    );

    // test case 2: batch call
    let mut batch = Vec::new();
    let mut states = vec![];

    for (account, blob) in mock_db.all_accounts.iter() {
        if account == first_account {
            continue;
        }
        states.push(AccountState::try_from(blob).unwrap());
        batch.push(MethodRequest::get_account(*account));
    }

    let responses = client.batch(batch).unwrap();
    assert_eq!(responses.len(), states.len());

    for (idx, response) in responses.into_iter().enumerate() {
        let account = response.unwrap().into_inner().unwrap_get_account().unwrap();
        let account_balances: Vec<_> = account.balances.iter().map(|bal| bal.amount).collect();
        let expected_resource_balances: Vec<_> = states[idx]
            .get_balance_resources()
            .unwrap()
            .iter()
            .map(|(_, bal_resource)| bal_resource.coin())
            .collect();
        assert_eq!(account_balances, expected_resource_balances);
        assert_eq!(
            account.sequence_number,
            states[idx]
                .get_account_resource()
                .unwrap()
                .unwrap()
                .sequence_number()
        );
    }
}

#[test]
fn test_get_metadata_latest() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let (actual_version, actual_timestamp) = mock_db.get_latest_commit_metadata().unwrap();

    let metadata = client.get_metadata().unwrap().into_inner();
    assert_eq!(metadata.version, actual_version);
    assert_eq!(metadata.timestamp, actual_timestamp);
}

#[test]
fn test_get_metadata() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let metadata = client.get_metadata_by_version(1).unwrap().into_inner();
    assert_eq!(metadata.version, 1);
    assert_eq!(metadata.timestamp, mock_db.timestamps[1]);
}

#[test]
fn test_limit_batch_size() {
    let (_, client, _runtime) = create_database_client_and_runtime();

    let mut batch = Vec::new();

    for i in 0..21 {
        batch.push(MethodRequest::get_metadata_by_version(i));
    }

    let ret = client.batch(batch).unwrap_err();
    let error = ret.json_rpc_error().unwrap();
    let expected = "JsonRpcError { code: -32600, message: \"Invalid Request: batch size = 21, exceed limit 20\", data: None }";
    assert_eq!(format!("{:?}", error), expected)
}

#[test]
fn test_get_events_page_limit() {
    let (_, client, _runtime) = create_database_client_and_runtime();

    let ret = client
        .get_events(
            EventKey::from_hex("13000000000000000000000000000000000000000a550c18").unwrap(),
            0,
            1001,
        )
        .unwrap_err();

    let error = ret.json_rpc_error().unwrap();
    let expected = "JsonRpcError { code: -32600, message: \"Invalid Request: page size = 1001, exceed limit 1000\", data: None }";
    assert_eq!(format!("{:?}", error), expected)
}

#[test]
fn test_get_transactions_page_limit() {
    let (_, client, _runtime) = create_database_client_and_runtime();

    let ret = client.get_transactions(0, 1001, false).unwrap_err();
    let error = ret.json_rpc_error().unwrap();
    let expected = "JsonRpcError { code: -32600, message: \"Invalid Request: page size = 1001, exceed limit 1000\", data: None }";
    assert_eq!(format!("{:?}", error), expected)
}

#[test]
fn test_get_events() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let event_index = 0;
    let mock_db_events = mock_db.events;
    let (first_event_version, first_event) = mock_db_events[event_index].clone();
    let event_key = first_event.key();

    let events = client
        .get_events(
            *event_key,
            first_event.sequence_number(),
            first_event.sequence_number() + 10,
        )
        .unwrap()
        .into_inner();

    let fetched_event = &events[event_index];
    assert_eq!(
        fetched_event.sequence_number,
        first_event.sequence_number(),
        "Seq number wrong"
    );
    assert_eq!(
        fetched_event.transaction_version, first_event_version,
        "Tx version wrong"
    );
}

#[test]
fn test_get_transactions() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let version = mock_db.get_latest_version().unwrap();
    let page = 800usize;

    for base_version in (0..version)
        .map(u64::from)
        .take(page)
        .collect::<Vec<_>>()
        .into_iter()
    {
        let txns = client
            .get_transactions(base_version, page as u64, true)
            .unwrap()
            .into_inner();

        for (i, view) in txns.iter().enumerate() {
            let version = base_version + i as u64;
            assert_eq!(view.version, version);
            let (tx, status) = &mock_db.all_txns[version as usize];
            assert_eq!(view.hash, tx.hash());

            // Check we returned correct events
            let expected_events = mock_db
                .events
                .iter()
                .filter(|(v, _)| *v == view.version)
                .map(|(_, e)| e)
                .collect::<Vec<_>>();

            assert_eq!(expected_events.len(), view.events.len());
            assert_eq!(VMStatusView::from(status), view.vm_status);

            for (i, event_view) in view.events.iter().enumerate() {
                let expected_event = expected_events.get(i).expect("Expected event didn't find");
                assert_eq!(event_view.sequence_number, expected_event.sequence_number());
                assert_eq!(event_view.transaction_version, version);
                assert_eq!(event_view.key, *expected_event.key());
                // TODO: check event_data
            }

            match tx {
                Transaction::BlockMetadata(t) => match view.transaction {
                    TransactionDataView::BlockMetadata { timestamp_usecs } => {
                        assert_eq!(t.clone().into_inner().1, timestamp_usecs);
                    }
                    _ => panic!("Returned value doesn't match!"),
                },
                Transaction::GenesisTransaction(_) => match view.transaction {
                    TransactionDataView::WriteSet { .. } => {}
                    _ => panic!("Returned value doesn't match!"),
                },
                Transaction::UserTransaction(t) => match &view.transaction {
                    TransactionDataView::UserTransaction {
                        sender,
                        script_hash,
                        chain_id,
                        ..
                    } => {
                        assert_eq!(t.sender(), *sender);
                        assert_eq!(&t.chain_id().id(), chain_id);
                        // TODO: verify every field
                        if let TransactionPayload::Script(s) = t.payload() {
                            assert_eq!(*script_hash, HashValue::sha3_256_of(s.code()));
                        }
                    }
                    _ => panic!("Returned value doesn't match!"),
                },
            }
        }
    }
}

#[test]
fn test_get_account_transaction() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    for (acc, blob) in mock_db.all_accounts.iter() {
        let ar = AccountResource::try_from(blob).unwrap();
        for seq in 1..ar.sequence_number() {
            let tx_view = client
                .get_account_transaction(*acc, seq, true)
                .unwrap()
                .into_inner()
                .unwrap();

            let (expected_tx, expected_status) = mock_db
                .all_txns
                .iter()
                .find_map(|(t, status)| {
                    if let Ok(x) = t.as_signed_user_txn() {
                        if x.sender() == *acc && x.sequence_number() == seq {
                            assert_eq!(tx_view.hash, t.hash());
                            return Some((x, status));
                        }
                    }
                    None
                })
                .expect("Couldn't find tx");

            // Check we returned correct events
            let expected_events = mock_db
                .events
                .iter()
                .filter(|(ev, _)| *ev == tx_view.version)
                .map(|(_, e)| e)
                .collect::<Vec<_>>();

            assert_eq!(tx_view.events.len(), expected_events.len());

            // check VM status
            assert_eq!(tx_view.vm_status, VMStatusView::from(expected_status));

            for (i, event_view) in tx_view.events.iter().enumerate() {
                let expected_event = expected_events.get(i).expect("Expected event didn't find");
                assert_eq!(event_view.sequence_number, expected_event.sequence_number());
                assert_eq!(event_view.transaction_version, tx_view.version);
                assert_eq!(event_view.key, *expected_event.key());
                // TODO: check event_data
            }

            let tx_data_view = tx_view.transaction;

            // Always user transaction
            match tx_data_view {
                TransactionDataView::UserTransaction {
                    sender,
                    sequence_number,
                    script_hash,
                    ..
                } => {
                    assert_eq!(*acc, sender);
                    assert_eq!(seq, sequence_number);

                    if let TransactionPayload::Script(s) = expected_tx.payload() {
                        assert_eq!(script_hash, HashValue::sha3_256_of(s.code()));
                    }
                }
                _ => panic!("wrong type"),
            }
        }
    }
}

#[test]
fn test_get_account_transactions() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    for (acc, blob) in mock_db.all_accounts.iter() {
        let total = AccountResource::try_from(blob).unwrap().sequence_number();

        let tx_views = client
            .get_account_transactions(*acc, 0, max(1, min(1000, total * 2)), true)
            .unwrap()
            .into_inner();
        assert_eq!(tx_views.len() as u64, total);
    }
}
#[test]
// Check that if version and ledger_version parameters are None, then the server returns the latest
// known state.
fn test_get_account_state_with_proof_null_versions() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let account = get_first_account_from_mock_db(&mock_db);
    let received_proof = client
        .get_account_state_with_proof(account, None, None)
        .unwrap()
        .into_inner();
    let expected_proof = get_first_state_proof_from_mock_db(&mock_db);

    // Check latest version returned, when no version specified
    assert_eq!(received_proof.version, expected_proof.version);
}

#[test]
fn test_get_account_state_with_proof() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let account = get_first_account_from_mock_db(&mock_db);

    let received_proof = client
        .get_account_state_with_proof(account, Some(0), Some(0))
        .unwrap()
        .into_inner();
    let expected_proof = get_first_state_proof_from_mock_db(&mock_db);
    let expected_blob = expected_proof.blob.as_ref().unwrap();
    let expected_sm_proof = expected_proof.proof.transaction_info_to_account_proof();
    let expected_txn_info_with_proof = expected_proof.proof.transaction_info_with_proof();

    //version
    assert_eq!(received_proof.version, expected_proof.version);

    // blob
    let account_blob: AccountStateBlob = bcs::from_bytes(&received_proof.blob.unwrap()).unwrap();
    assert_eq!(account_blob, *expected_blob);

    // proof
    let sm_proof: SparseMerkleProof<AccountStateBlob> =
        bcs::from_bytes(&received_proof.proof.transaction_info_to_account_proof).unwrap();
    assert_eq!(sm_proof, *expected_sm_proof);
    let txn_info: TransactionInfo =
        bcs::from_bytes(&received_proof.proof.transaction_info).unwrap();
    let li_proof: TransactionAccumulatorProof =
        bcs::from_bytes(&received_proof.proof.ledger_info_to_transaction_info_proof).unwrap();
    let txn_info_with_proof = TransactionInfoWithProof::new(li_proof, txn_info);
    assert_eq!(txn_info_with_proof, *expected_txn_info_with_proof);
}

#[test]
fn test_get_state_proof() {
    let (mock_db, client, _runtime) = create_database_client_and_runtime();

    let version = mock_db.version;
    let proof = client.get_state_proof(version).unwrap().into_inner();
    let li: LedgerInfoWithSignatures = bcs::from_bytes(&proof.ledger_info_with_signatures).unwrap();
    assert_eq!(li.ledger_info().version(), version);
}

#[test]
fn test_get_network_status() {
    let (_mock_db, client, _runtime) = create_database_client_and_runtime();

    let connected_peers = client.get_network_status().unwrap().into_inner();
    // expect no connected peers when no network is running
    assert_eq!(connected_peers, 0);
}

#[test]
fn test_health_check() {
    let (_mock_db, _runtime, url, _) = create_db_and_runtime();

    let client = reqwest::blocking::Client::new();
    let healthy_url = format!("{}/-/healthy", url);
    let resp = client.get(&healthy_url).send().unwrap();
    assert_eq!(resp.status(), 200);

    let healthy_url = format!(
        "{}/-/healthy?duration={}",
        url,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let resp = client.get(&healthy_url).send().unwrap();
    assert_eq!(resp.status(), 200);
}

#[test]
fn test_sdk_info_from_user_agent() {
    // Invalid user agents:
    assert_eq!(sdk_info_from_user_agent(None), SdkInfo::default());
    assert_eq!(sdk_info_from_user_agent(Some("")), SdkInfo::default());
    // If we have a bad language, don't trust the version
    assert_eq!(
        sdk_info_from_user_agent(Some("very-custom-unreal / 1.1.1")),
        SdkInfo::default()
    );
    // If we have a bad version, don't trust the language
    assert_eq!(
        sdk_info_from_user_agent(Some("diem-client-sdk-python / 0.12.223")),
        SdkInfo::default()
    );
    assert_eq!(
        sdk_info_from_user_agent(Some("diem-client-sdk-python / 105.12.22")),
        SdkInfo::default()
    );

    // Valid user agents:
    assert_eq!(
        sdk_info_from_user_agent(Some("diem-client-SdK-JaVa/ 3.21.09")),
        SdkInfo {
            language: SdkLang::Java,
            version: SdkVersion {
                major: 3,
                minor: 21,
                patch: 9
            }
        }
    );
    assert_eq!(
        sdk_info_from_user_agent(Some("diem-client-SdK-cpp/3.21.09")),
        SdkInfo {
            language: SdkLang::Cpp,
            version: SdkVersion {
                major: 3,
                minor: 21,
                patch: 9
            }
        }
    );
    assert_eq!(
        sdk_info_from_user_agent(Some("diem-client-sdk-python / 0.1.22")),
        SdkInfo {
            language: SdkLang::Python,
            version: SdkVersion {
                major: 0,
                minor: 1,
                patch: 22
            }
        }
    );
}

#[test]
fn test_check_latest_ledger_info_timestamp() {
    let now = SystemTime::now();
    let ledger_latest_timestamp_lack = 10;

    let ledger_latest_timestamp = now
        .sub(Duration::from_secs(ledger_latest_timestamp_lack))
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;

    assert!(check_latest_ledger_info_timestamp(
        ledger_latest_timestamp_lack - 1,
        ledger_latest_timestamp,
        now
    )
    .is_err());
    assert!(check_latest_ledger_info_timestamp(
        ledger_latest_timestamp_lack + 1,
        ledger_latest_timestamp,
        now
    )
    .is_ok());
}

/// Returns the first account address stored in the given mock database.
fn get_first_account_from_mock_db(mock_db: &MockDiemDB) -> AccountAddress {
    *mock_db
        .all_accounts
        .keys()
        .next()
        .expect("mock DB missing account")
}

/// Returns the first account_state_with_proof stored in the given mock database.
fn get_first_state_proof_from_mock_db(mock_db: &MockDiemDB) -> AccountStateWithProof {
    mock_db
        .account_state_with_proof
        .get(0)
        .expect("mock DB missing account state with proof")
        .clone()
}

fn gen_string(len: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(len)
        .map(char::from)
        .collect()
}
