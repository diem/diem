// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    runtime::bootstrap,
    tests::mock_db::MockLibraDB,
    views::{AccountView, BlockMetadata, TransactionDataView, TransactionView},
};
use futures::{channel::mpsc::channel, StreamExt};
use hex;
use libra_config::utils;
use libra_crypto::ed25519::*;
use libra_temppath::TempPath;
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_config::AccountResource,
    crypto_proxies::LedgerInfoWithSignatures,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, TransactionToCommit},
};
use libradb::{test_helper::arb_blocks_to_commit, LibraDB, LibraDBTrait};
use proptest::prelude::*;
use reqwest;
use serde_json::{self, Value};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

type JsonMap = HashMap<String, serde_json::Value>;

#[test]
fn test_json_rpc_protocol() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let mp_sender = channel(1024).0;
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(db), mp_sender);
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

// returns MockLibraDB for unit-testing
fn mock_db(blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>) -> MockLibraDB {
    let mut version = 0;
    let mut all_accounts = BTreeMap::new();
    let mut all_txns = vec![];
    let mut events = vec![];
    let mut timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64;

    for (txns_to_commit, _ledger_info_with_sigs) in &blocks {
        timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        for (idx, txn) in txns_to_commit.iter().enumerate() {
            events.extend(
                txn.events()
                    .iter()
                    .map(|e| ((idx + version) as u64, e.clone())),
            );
        }
        version += txns_to_commit.len();
        let mut account_states = HashMap::new();
        // Get the ground truth of account states.
        txns_to_commit.iter().for_each(|txn_to_commit| {
            account_states.extend(txn_to_commit.account_states().clone())
        });

        // Record all account states.
        for (address, blob) in account_states.into_iter() {
            all_accounts.insert(address.clone(), blob.clone());
        }

        // Record all transactions.
        all_txns.extend(
            txns_to_commit
                .iter()
                .map(|txn_to_commit| txn_to_commit.transaction().clone()),
        );
    }

    MockLibraDB {
        timestamp,
        version: version as u64,
        all_accounts,
        all_txns,
        events,
    }
}

#[test]
fn test_transaction_submission() {
    let (mp_sender, mut mp_events) = channel(1);
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let runtime = bootstrap(address.parse().unwrap(), Arc::new(db), mp_sender);
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    // future that mocks shared mempool execution
    runtime.spawn(async move {
        let validator = MockVMValidator;
        while let Some((txn, cb)) = mp_events.next().await {
            let vm_status = validator.validate_transaction(txn).await.unwrap();
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
        let keypair = compat::generate_keypair(None);
        let txn = get_test_signed_txn(sender, 0, &keypair.0, keypair.1, None);
        let payload = hex::encode(lcs::to_bytes(&txn).unwrap());
        let request =
            serde_json::json!({"jsonrpc": "2.0", "method": "submit", "params": [payload], "id": 1});
        client.post(&url).json(&request).send().unwrap()
    };

    // check successful submission
    let sender = AccountAddress::new([9; ADDRESS_LENGTH]);
    let data: JsonMap = txn_submission(sender).json().unwrap();
    assert_eq!(data.get("result").unwrap(), &serde_json::Value::Null);

    // check vm error submission
    let sender = AccountAddress::new([0; ADDRESS_LENGTH]);
    assert_eq!(fetch_error(txn_submission(sender)), -32000);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_get_account_state(blocks in arb_blocks_to_commit()) {
        // set up MockLibraDB
        let mock_db = mock_db(blocks);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let mp_sender = channel(1024).0;
        let _runtime = bootstrap(address.parse().unwrap(), Arc::new(mock_db.clone()), mp_sender);
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        // test case 1: single call
        let (first_account, blob) = mock_db.all_accounts.iter().next().unwrap();
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state", "params": [first_account], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let expected_resource = AccountResource::try_from(blob).unwrap();
        let data = fetch_result(resp.json().unwrap());
        assert_eq!(data.get("balance").unwrap().as_u64(), Some(expected_resource.balance()));
        assert_eq!(data.get("sequence_number").unwrap().as_u64(), Some(expected_resource.sequence_number()));

        // test case 2: batch call
        let mut batch_accounts = vec![];
        let mut ids_expected = HashSet::new();
        let mut id = 0;
        let mut account_resources = HashMap::new();

        for (account, blob) in mock_db.all_accounts.iter() {
            if account == first_account {
                continue;
            }
            batch_accounts.push(account);
            account_resources.insert(id, AccountResource::try_from(blob).unwrap());
            ids_expected.insert(id);
            id += 1;
            if id == 3 { break; }
        }
        let requests = batch_accounts
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
                    let id = response.get("id").unwrap().as_u64().unwrap();
                    ids.insert(id);
                    let result: JsonMap = serde_json::from_value(
                        response.as_object().unwrap().get("result").unwrap().clone(),
                    )
                    .unwrap();
                    let account_resource = account_resources.get(&id).unwrap();
                    assert_eq!(result.get("balance").unwrap().as_u64(), Some(account_resource.balance()));

                    // deserialize
                    let result_view: AccountView =
                        serde_json::from_value(response.get("result").unwrap().clone()).unwrap();
                    let expected_view = AccountView::new(account_resource);
                    assert_eq!(result_view, expected_view);
                    assert_eq!(result_view.balance(), account_resource.balance());
                }
                assert_eq!(ids, ids_expected);
            }
            _ => panic!("invalid server response format"),
        }
    }

    #[test]
    fn test_get_metadata(blocks in arb_blocks_to_commit()) {
        // set up MockLibraDB
        let mock_db = mock_db(blocks);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let _runtime = bootstrap(address.parse().unwrap(), Arc::new(mock_db.clone()), channel(1).0);
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        let (actual_version, actual_timestamp) = mock_db.get_latest_commit_metadata().unwrap();

        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_metadata", "params": [], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let resp_json: JsonMap = resp.json().unwrap();

        // deserialize
        let result_view: BlockMetadata = serde_json::from_value(resp_json.get("result").unwrap().clone()).unwrap();
        assert_eq!(result_view.version, actual_version);
        assert_eq!(result_view.timestamp, actual_timestamp);

        let data = fetch_result(resp_json);
        assert_eq!(data.get("version").unwrap().as_u64(), Some(actual_version));
        assert_eq!(data.get("timestamp").unwrap().as_u64(), Some(actual_timestamp));
    }


    #[test]
    fn test_get_events(blocks in arb_blocks_to_commit()) {
        // set up MockLibraDB
        let mock_db = mock_db(blocks);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let _runtime = bootstrap(address.parse().unwrap(), Arc::new(mock_db.clone()), channel(1).0);
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        let event_key = hex::encode(mock_db.events.iter().next().unwrap().1.key().as_bytes());
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_events", "params": [event_key, 0, 10], "id": 1});
        let _resp = client.post(&url).json(&request).send().unwrap();
        // TODO: flaky tests
        // let mut events: Vec<EventView> = fetch_data(resp);
        // let event = events.remove(0);
        // assert_eq!(event.sequence_number, 0);
        // assert_eq!(event.transaction_version, 0);
    }
    #[test]
    fn test_get_transactions(blocks in arb_blocks_to_commit()) {
        test_get_transactions_impl(blocks);
    }

    #[test]
    fn test_get_account_transaction(blocks in arb_blocks_to_commit()) {
        test_get_account_transaction_impl(blocks);
    }
}

fn test_get_transactions_impl(blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>) {
    // set up MockLibraDB
    let mock_db = mock_db(blocks);

    // set up JSON RPC
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(
        address.parse().unwrap(),
        Arc::new(mock_db.clone()),
        channel(1).0,
    );
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    let version = mock_db.get_latest_version().unwrap();
    let page = 800usize;

    for base_version in (0..version).map(u64::from).take(page).collect::<Vec<_>>() {
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_transactions", "params": [base_version, page as u64, false], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let data = fetch_result(resp.json().unwrap());
        match data {
            serde_json::Value::Array(responses) => {
                for (i, response) in responses.iter().enumerate() {
                    let version = base_version + i as u64;
                    assert_eq!(
                        response
                            .as_object()
                            .unwrap()
                            .get("version")
                            .unwrap()
                            .as_u64(),
                        Some(version)
                    );
                    let tx = &mock_db.all_txns[version as usize];
                    let view = response.as_object().unwrap().get("transaction").unwrap();
                    match tx {
                        Transaction::BlockMetadata(t) => {
                            assert_eq!(
                                "blockmetadata",
                                view.get("type").unwrap().as_str().unwrap()
                            );
                            assert_eq!(
                                t.clone().into_inner().unwrap().1,
                                view.get("timestamp_usecs").unwrap().as_u64().unwrap()
                            );
                        }
                        Transaction::WriteSet(_) => {
                            assert_eq!("writeset", view.get("type").unwrap().as_str().unwrap());
                        }
                        Transaction::UserTransaction(t) => {
                            assert_eq!("user", view.get("type").unwrap().as_str().unwrap());
                            assert_eq!(
                                t.sender().to_string(),
                                view.get("sender").unwrap().as_str().unwrap()
                            );
                            // TODO: verify every field
                        }
                    }
                }
            }
            _ => panic!("invalid server response format"),
        }
    }
}

fn test_get_account_transaction_impl(
    blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) {
    // set up MockLibraDB
    let mock_db = mock_db(blocks);

    // set up JSON RPC
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(
        address.parse().unwrap(),
        Arc::new(mock_db.clone()),
        channel(1).0,
    );
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    for (acc, blob) in mock_db.all_accounts.iter() {
        let ar = AccountResource::try_from(blob).unwrap();
        for seq in 1..ar.sequence_number() {
            let p_addr = String::from(acc);
            let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": [p_addr, seq, false], "id": 1});
            let resp = client.post(&url).json(&request).send().unwrap();
            assert_eq!(resp.status(), 200);

            let data: Option<TransactionView> = fetch_data(resp);
            let view = data.unwrap().transaction;
            // Always user transaction
            match view {
                TransactionDataView::UserTransaction {
                    sender,
                    sequence_number,
                    ..
                } => {
                    assert_eq!(acc.to_string(), sender);
                    assert_eq!(seq, sequence_number);
                }
                _ => panic!("wrong type"),
            }
        }
    }
}

fn fetch_data<T>(response: reqwest::blocking::Response) -> T
where
    T: serde::de::DeserializeOwned,
{
    let data: JsonMap = response.json().unwrap();
    serde_json::from_value(data.get("result").unwrap().clone()).unwrap()
}

fn fetch_result(data: JsonMap) -> Value {
    let result: Value = serde_json::from_value(data.get("result").unwrap().clone()).unwrap();
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
