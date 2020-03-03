// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::runtime::bootstrap;
use libra_config::utils;
use libra_temppath::TempPath;
use libra_types::{
    account_address::AccountAddress, account_config::AccountResource,
    crypto_proxies::LedgerInfoWithSignatures, transaction::TransactionToCommit,
};
use libradb::{test_helper::arb_blocks_to_commit, LibraDB};
use proptest::prelude::*;
use reqwest;
use serde_json;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

type JsonMap = HashMap<String, serde_json::Value>;

#[test]
fn test_json_rpc_protocol() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let _runtime = bootstrap(address.parse().unwrap(), Arc::new(db));
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

fn setup_db(
    blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) -> (Arc<LibraDB>, BTreeMap<AccountAddress, AccountResource>) {
    // set up db
    let tmp_dir = TempPath::new();
    let db = Arc::new(LibraDB::new(&tmp_dir));
    let mut version = 0;
    let mut all_accounts = BTreeMap::new();
    let mut all_txns = vec![];

    for (txns_to_commit, ledger_info_with_sigs) in &blocks {
        db.save_transactions(
            &txns_to_commit.clone(),
            version,
            Some(&ledger_info_with_sigs.clone()),
        )
        .unwrap();
        version += txns_to_commit.len() as u64;
        let mut account_states = HashMap::new();
        // Get the ground truth of account states.
        txns_to_commit.iter().for_each(|txn_to_commit| {
            account_states.extend(txn_to_commit.account_states().clone())
        });

        // Record all account states.
        for (address, blob) in account_states.iter() {
            all_accounts.insert(address.clone(), AccountResource::try_from(blob).unwrap());
        }

        // Record all transactions.
        all_txns.extend(
            txns_to_commit
                .iter()
                .map(|txn_to_commit| txn_to_commit.transaction().clone()),
        );
    }

    (db, all_accounts)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_get_account_state(blocks in arb_blocks_to_commit()) {
        // set up LibraDB
        let (db, all_accounts) = setup_db(blocks);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let _runtime = bootstrap(address.parse().unwrap(), Arc::clone(&db));
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        // test case 1: single call
        let (first_account, account_resource) = all_accounts.iter().next().unwrap();
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state", "params": [first_account], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let data = fetch_result(resp.json().unwrap());
        assert_eq!(data.get("balance").unwrap().as_u64(), Some(account_resource.balance()));
        assert_eq!(data.get("sequence_number").unwrap().as_u64(), Some(account_resource.sequence_number()));

        // test case 2: batch call
        let mut batch_accounts = vec![];
        let mut ids_expected = HashSet::new();
        let mut id = 0;
        let mut account_resources = HashMap::new();

        for (account, account_resource) in all_accounts.iter() {
            if account == first_account {
                continue;
            }
            batch_accounts.push(account);
            account_resources.insert(id, account_resource);
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
                    assert_eq!(result.get("balance").unwrap().as_u64(), Some(account_resources.get(&id).unwrap().balance()));
                }
                assert_eq!(ids, ids_expected);
            }
            _ => panic!("invalid server response format"),
        }
    }

    #[test]
    fn test_get_metadata(blocks in arb_blocks_to_commit()) {
        // set up LibraDB
        let (db, _all_accounts) = setup_db(blocks);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let _runtime = bootstrap(address.parse().unwrap(), Arc::clone(&db));
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        let (actual_version, actual_timestamp) = db.get_latest_commit_metadata().unwrap();

        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_metadata", "params": [], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        assert_eq!(resp.status(), 200);
        let data = fetch_result(resp.json().unwrap());
        assert_eq!(data.get("version").unwrap().as_u64(), Some(actual_version));
        assert_eq!(data.get("timestamp").unwrap().as_u64(), Some(actual_timestamp));
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
