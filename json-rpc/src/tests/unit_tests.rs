// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::{JsonRpcAsyncClient, JsonRpcBatch, JsonRpcResponse},
    errors::{JsonRpcError, ServerCode},
    runtime::bootstrap,
    tests::mock_db::MockLibraDB,
    views::{
        AccountStateWithProofView, BlockMetadata, BytesView, EventView, StateProofView,
        TransactionDataView, TransactionView,
    },
};
use anyhow::format_err;
use futures::{channel::mpsc::channel, StreamExt};
use hex;
use libra_config::utils;
use libra_crypto::{ed25519::Ed25519PrivateKey, HashValue, PrivateKey, Uniform};
use libra_temppath::TempPath;
use libra_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    event::EventKey,
    language_storage::TypeTag,
    ledger_info::LedgerInfoWithSignatures,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    proof::{SparseMerkleProof, TransactionAccumulatorProof},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, TransactionInfo, TransactionPayload, TransactionToCommit},
    vm_error::{StatusCode, VMStatus},
};
use libradb::{test_helper::arb_blocks_to_commit, LibraDB, LibraDBTrait};
use proptest::prelude::*;
use reqwest;
use serde_json::{self, Value};
use std::{
    collections::{BTreeMap, HashMap},
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
fn mock_db(
    blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
    account_state_with_proof: Option<AccountStateWithProof>,
) -> MockLibraDB {
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
        all_txns.extend(txns_to_commit.iter().map(|txn_to_commit| {
            (
                txn_to_commit.transaction().clone(),
                txn_to_commit.major_status(),
            )
        }));
    }

    let account_state_with_proof = if let Some(mut proof) = account_state_with_proof {
        if proof.blob.is_none() {
            let (_, blob) = all_accounts.iter().next().unwrap();
            proof.blob = Some(blob.clone());
        }
        vec![proof]
    } else {
        vec![]
    };
    if events.is_empty() {
        // mock the first event
        let mock_event = ContractEvent::new(
            EventKey::new_from_address(&AccountAddress::random(), 0),
            0,
            TypeTag::Bool,
            b"event_data".to_vec(),
        );
        events.push((version as u64, mock_event));
    }

    MockLibraDB {
        timestamp,
        version: version as u64,
        all_accounts,
        all_txns,
        events,
        account_state_with_proof,
    }
}

#[test]
fn test_transaction_submission() {
    let (mp_sender, mut mp_events) = channel(1);
    let tmp_dir = TempPath::new();
    let db = LibraDB::new(&tmp_dir);
    let port = utils::get_available_port();
    let address = format!("0.0.0.0:{}", port);
    let mut runtime = bootstrap(address.parse().unwrap(), Arc::new(db), mp_sender);
    let client = JsonRpcAsyncClient::new(reqwest::Client::new(), "0.0.0.0", port);

    // future that mocks shared mempool execution
    runtime.spawn(async move {
        let validator = MockVMValidator;
        while let Some((txn, cb)) = mp_events.next().await {
            let vm_status = validator.validate_transaction(txn).await.unwrap().status();
            let result = if vm_status.is_some() {
                (MempoolStatus::new(MempoolStatusCode::VmError), vm_status)
            } else {
                (MempoolStatus::new(MempoolStatusCode::Accepted), None)
            };
            cb.send(Ok(result)).unwrap();
        }
    });

    // closure that checks transaction submission for given account
    let mut txn_submission = move |sender| {
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let txn = get_test_signed_txn(sender, 0, &privkey, privkey.public_key(), None);
        let mut batch = JsonRpcBatch::default();
        batch.add_submit_request(txn).unwrap();
        runtime.block_on(client.execute(batch)).unwrap()
    };

    // check successful submission
    let sender = AccountAddress::new([9; AccountAddress::LENGTH]);
    assert!(txn_submission(sender)[0].as_ref().unwrap() == &JsonRpcResponse::SubmissionResponse);

    // check vm error submission
    let sender = AccountAddress::new([0; AccountAddress::LENGTH]);
    let response = &txn_submission(sender)[0];

    if let Err(e) = response {
        if let Some(error) = e.downcast_ref::<JsonRpcError>() {
            assert_eq!(error.code, ServerCode::VmValidationError as i16);
            let vm_error: VMStatus =
                serde_json::from_value(error.data.as_ref().unwrap().clone()).unwrap();
            assert_eq!(
                vm_error.major_status,
                StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST
            );
        } else {
            panic!("unexpected error format");
        }
    } else {
        panic!("expected error");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_get_account_state(blocks in arb_blocks_to_commit()) {
        // set up MockLibraDB
        let mock_db = mock_db(blocks, None);

        // set up JSON RPC
        let port = utils::get_available_port();
        let address = format!("0.0.0.0:{}", port);
        let mp_sender = channel(1024).0;
        let mut rt = bootstrap(address.parse().unwrap(), Arc::new(mock_db.clone()), mp_sender);
        let client = JsonRpcAsyncClient::new(reqwest::Client::new(), "0.0.0.0", port);

        // test case 1: single call
        let (first_account, blob) = mock_db.all_accounts.iter().next().unwrap();
        let expected_resource = AccountState::try_from(blob).unwrap();

        let mut batch = JsonRpcBatch::default();
        batch.add_get_account_state_request(*first_account);
        let result = rt.block_on(client.execute(batch)).unwrap().remove(0).unwrap();
        match result {
            JsonRpcResponse::AccountResponse(account) => {
                assert_eq!(account.balance, expected_resource.get_balance_resource().unwrap().unwrap().coin());
                assert_eq!(account.sequence_number, expected_resource.get_account_resource().unwrap().unwrap().sequence_number());
            }
            _ => panic!("unexpected response")
        }

        // test case 2: batch call
        let mut batch = JsonRpcBatch::default();
        let mut states = vec![];

        for (account, blob) in mock_db.all_accounts.iter() {
            if account == first_account {
                continue;
            }
            states.push(AccountState::try_from(blob).unwrap());
            batch.add_get_account_state_request(*account);
        }

        let responses = rt.block_on(client.execute(batch)).unwrap();
        assert_eq!(responses.len(), states.len());

        for (idx, response) in responses.into_iter().enumerate() {
            match response.unwrap() {
                JsonRpcResponse::AccountResponse(account) => {
                    assert_eq!(account.balance, states[idx].get_balance_resource().unwrap().unwrap().coin());
                    assert_eq!(account.sequence_number, states[idx].get_account_resource().unwrap().unwrap().sequence_number());
                }
                _ => panic!("unexpected response")
            }
        }
    }

    #[test]
    fn test_get_metadata(blocks in arb_blocks_to_commit()) {
        // set up MockLibraDB
        let mock_db = mock_db(blocks, None);

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
        let mock_db = mock_db(blocks, None);

        // set up JSON RPC
        let address = format!("0.0.0.0:{}", utils::get_available_port());
        let _runtime = bootstrap(address.parse().unwrap(), Arc::new(mock_db.clone()), channel(1).0);
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}", address);

        let event_index = 0;
        let mock_db_events = mock_db.events;
        let (first_event_version, first_event) = mock_db_events[event_index].clone();
        let event_key = hex::encode(first_event.key().as_bytes());
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_events", "params": [event_key, first_event.sequence_number(), first_event.sequence_number() + 10], "id": 1});
        let resp = client.post(&url).json(&request).send().unwrap();
        let events: Vec<EventView> = fetch_data(resp);

        let fetched_event = &events[event_index];
        assert_eq!(fetched_event.sequence_number, first_event.sequence_number(), "Seq number wrong");
        assert_eq!(fetched_event.transaction_version, first_event_version, "Tx version wrong");
    }

    #[test]
    fn test_get_transactions(blocks in arb_blocks_to_commit()) {
        test_get_transactions_impl(blocks);
    }

    #[test]
    fn test_get_account_transaction(blocks in arb_blocks_to_commit()) {
        test_get_account_transaction_impl(blocks);
    }

    #[test]
    fn test_get_account_state_with_proof(
        blocks in arb_blocks_to_commit(),
        account_state_with_proof in any::<AccountStateWithProof>(),
    ) {
        test_get_account_state_with_proof_impl(blocks, account_state_with_proof);
    }
}

fn test_get_transactions_impl(blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>) {
    // set up MockLibraDB
    let mock_db = mock_db(blocks, None);

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
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_transactions", "params": [base_version, page as u64, true], "id": 1});
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
                    let (tx, status) = &mock_db.all_txns[version as usize];

                    let view: TransactionView =
                        serde_json::from_value(response.clone()).expect("Invalid result");

                    // Check we returned correct events
                    let expected_events = mock_db
                        .events
                        .iter()
                        .filter(|(v, _)| *v == view.version)
                        .map(|(_, e)| e)
                        .collect::<Vec<_>>();

                    assert_eq!(expected_events.len(), view.events.len());
                    assert_eq!(status, &view.vm_status);

                    for (i, event_view) in view.events.iter().enumerate() {
                        let expected_event =
                            expected_events.get(i).expect("Expected event didn't find");
                        assert_eq!(event_view.sequence_number, expected_event.sequence_number());
                        assert_eq!(event_view.transaction_version, version);
                        assert_eq!(
                            event_view.key.0,
                            BytesView::from(expected_event.key().as_bytes()).0
                        );
                        // TODO: check event_data
                    }

                    match tx {
                        Transaction::BlockMetadata(t) => match view.transaction {
                            TransactionDataView::BlockMetadata { timestamp_usecs } => {
                                assert_eq!(t.clone().into_inner().unwrap().1, timestamp_usecs);
                            }
                            _ => panic!("Returned value doesn't match!"),
                        },
                        Transaction::WaypointWriteSet(_) => match view.transaction {
                            TransactionDataView::WriteSet { .. } => {}
                            _ => panic!("Returned value doesn't match!"),
                        },
                        Transaction::UserTransaction(t) => match view.transaction {
                            TransactionDataView::UserTransaction {
                                sender,
                                script_hash,
                                ..
                            } => {
                                assert_eq!(t.sender().to_string(), sender);
                                // TODO: verify every field
                                if let TransactionPayload::Script(s) = t.payload() {
                                    assert_eq!(
                                        script_hash,
                                        HashValue::from_sha3_256(s.code()).to_hex()
                                    );
                                }
                            }
                            _ => panic!("Returned value doesn't match!"),
                        },
                    }
                }
            }
            _ => panic!("invalid server response format"),
        }
    }
}

#[test]
fn test_get_state_proof() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(
        address.parse().unwrap(),
        Arc::new(mock_db(vec![], None)),
        channel(1024).0,
    );
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    let version = 10;
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_state_proof", "params": [version], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    let proof: StateProofView = fetch_data(resp);
    let li: LedgerInfoWithSignatures =
        lcs::from_bytes(&proof.ledger_info_with_signatures.into_bytes().unwrap()).unwrap();
    assert_eq!(li.ledger_info().version(), version);
}

fn test_get_account_transaction_impl(
    blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
) {
    // set up MockLibraDB
    let mock_db = mock_db(blocks, None);

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
            let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_transaction", "params": [p_addr, seq, true], "id": 1});
            let resp = client.post(&url).json(&request).send().unwrap();
            assert_eq!(resp.status(), 200);

            let data: Option<TransactionView> = fetch_data(resp);
            let tx_view = data.expect("Transaction didn't exists!");

            let (expected_tx, expected_status) = mock_db
                .all_txns
                .iter()
                .find_map(|(t, status)| {
                    if let Ok(x) = t.as_signed_user_txn() {
                        if x.sender() == *acc && x.sequence_number() == seq {
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

            // check VM major status
            assert_eq!(&tx_view.vm_status, expected_status);

            for (i, event_view) in tx_view.events.iter().enumerate() {
                let expected_event = expected_events.get(i).expect("Expected event didn't find");
                assert_eq!(event_view.sequence_number, expected_event.sequence_number());
                assert_eq!(event_view.transaction_version, tx_view.version);
                assert_eq!(
                    event_view.key.0,
                    BytesView::from(expected_event.key().as_bytes()).0
                );
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
                    assert_eq!(acc.to_string(), sender);
                    assert_eq!(seq, sequence_number);

                    if let TransactionPayload::Script(s) = expected_tx.payload() {
                        assert_eq!(script_hash, HashValue::from_sha3_256(s.code()).to_hex());
                    }
                }
                _ => panic!("wrong type"),
            }
        }
    }
}

fn test_get_account_state_with_proof_impl(
    blocks: Vec<(Vec<TransactionToCommit>, LedgerInfoWithSignatures)>,
    account_state_with_proof: AccountStateWithProof,
) {
    // set up MockLibraDB
    let mock_db = mock_db(blocks, Some(account_state_with_proof));

    // set up JSON RPC
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let _runtime = bootstrap(
        address.parse().unwrap(),
        Arc::new(mock_db.clone()),
        channel(1).0,
    );
    let client = reqwest::blocking::Client::new();
    let url = format!("http://{}", address);

    let account = mock_db
        .all_accounts
        .keys()
        .next()
        .expect("mock DB missing account");
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "get_account_state_with_proof", "params": [account, 0, 0], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);

    let received_proof: AccountStateWithProofView = fetch_data(resp);
    let expected_proof = mock_db
        .account_state_with_proof
        .get(0)
        .expect("mock DB missing account state with proof");
    let expected_blob = expected_proof.blob.as_ref().unwrap();
    let expected_sm_proof = expected_proof.proof.transaction_info_to_account_proof();
    let expected_txn_info = expected_proof.proof.transaction_info();
    let expected_li_proof = expected_proof.proof.ledger_info_to_transaction_info_proof();

    //version
    assert_eq!(received_proof.version, expected_proof.version);

    // blob
    let account_blob: AccountStateBlob =
        lcs::from_bytes(&received_proof.blob.unwrap().into_bytes().unwrap()).unwrap();
    assert_eq!(account_blob, *expected_blob);

    // proof
    let sm_proof: SparseMerkleProof = lcs::from_bytes(
        &received_proof
            .proof
            .transaction_info_to_account_proof
            .into_bytes()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(sm_proof, *expected_sm_proof);
    let txn_info: TransactionInfo =
        lcs::from_bytes(&received_proof.proof.transaction_info.into_bytes().unwrap()).unwrap();
    assert_eq!(txn_info, *expected_txn_info);
    let li_proof: TransactionAccumulatorProof = lcs::from_bytes(
        &received_proof
            .proof
            .ledger_info_to_transaction_info_proof
            .into_bytes()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(li_proof, *expected_li_proof);
}

fn fetch_data<T>(response: reqwest::blocking::Response) -> T
where
    T: serde::de::DeserializeOwned,
{
    let data: JsonMap = response.json().unwrap();
    serde_json::from_value(data.get("result").unwrap().clone()).unwrap()
}

fn fetch_result(data: JsonMap) -> Value {
    let result: Value = serde_json::from_value(
        data.get("result")
            .ok_or_else(|| format_err!("Unexpected response: {:?}", data))
            .unwrap()
            .clone(),
    )
    .unwrap();
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
