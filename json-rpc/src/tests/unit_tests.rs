// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    errors::{InvalidArguments, JsonRpcError, ServerCode},
    tests::{
        genesis::generate_genesis_state,
        utils::{test_bootstrap, MockLibraDB},
    },
};
use futures::{channel::mpsc::channel, StreamExt};
use libra_config::utils;
use libra_crypto::{ed25519::Ed25519PrivateKey, hash::CryptoHash, HashValue, PrivateKey, Uniform};
use libra_json_rpc_client::{
    views::{
        AccountStateWithProofView, AccountView, BlockMetadata, BytesView, EventView,
        StateProofView, TransactionDataView, TransactionView, VMStatusView,
    },
    JsonRpcAsyncClient, JsonRpcBatch, JsonRpcResponse, ResponseAsView,
};
use libra_json_rpc_types::{
    response::JsonRpcErrorResponse,
    views::{
        JSONRPC_LIBRA_CHAIN_ID, JSONRPC_LIBRA_LEDGER_TIMESTAMPUSECS, JSONRPC_LIBRA_LEDGER_VERSION,
    },
};

use libra_proptest_helpers::ValueGenerator;
use libra_types::{
    account_address::AccountAddress,
    account_config::{from_currency_code_string, AccountResource, FreezingBit, LBR_NAME},
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    chain_id::ChainId,
    contract_event::ContractEvent,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    mempool_status::{MempoolStatus, MempoolStatusCode},
    proof::{SparseMerkleProof, TransactionAccumulatorProof, TransactionInfoWithProof},
    test_helpers::transaction_test_helpers::get_test_signed_txn,
    transaction::{Transaction, TransactionInfo, TransactionPayload},
    vm_status::StatusCode,
};
use libradb::test_helper::arb_blocks_to_commit;
use move_core_types::{
    language_storage::TypeTag,
    move_resource::MoveResource,
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_vm_types::values::{Struct, Value};
use proptest::prelude::*;
use std::{
    cmp::{max, min},
    collections::HashMap,
    convert::TryFrom,
    str::FromStr,
    sync::Arc,
};
use storage_interface::DbReader;
use tokio::runtime::Runtime;
use vm_validator::{
    mocks::mock_vm_validator::MockVMValidator, vm_validator::TransactionValidation,
};

type JsonMap = HashMap<String, serde_json::Value>;

// returns MockLibraDB for unit-testing
fn mock_db() -> MockLibraDB {
    let mut gen = ValueGenerator::new();
    let blocks = gen.generate(arb_blocks_to_commit());
    let mut account_state_with_proof = gen.generate(any::<AccountStateWithProof>());

    let mut version = 1;
    let mut all_accounts = HashMap::new();
    let mut all_txns = vec![];
    let mut events = vec![];
    let mut timestamps = vec![0 as u64];

    for (txns_to_commit, ledger_info_with_sigs) in &blocks {
        for (idx, txn) in txns_to_commit.iter().enumerate() {
            timestamps.push(ledger_info_with_sigs.ledger_info().timestamp_usecs());
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
            let mut state = AccountState::try_from(&blob).unwrap();
            let freezing_bit = Value::struct_(Struct::pack(vec![Value::bool(false)], true))
                .value_as::<Struct>()
                .unwrap()
                .simple_serialize(&MoveStructLayout::new(vec![MoveTypeLayout::Bool]))
                .unwrap();
            state.insert(FreezingBit::resource_path(), freezing_bit);
            all_accounts.insert(address, AccountStateBlob::try_from(&state).unwrap());
        }

        // Record all transactions.
        all_txns.extend(txns_to_commit.iter().map(|txn_to_commit| {
            (
                txn_to_commit.transaction().clone(),
                txn_to_commit.status().clone(),
            )
        }));
    }

    if account_state_with_proof.blob.is_none() {
        let (_, blob) = all_accounts.iter().next().unwrap();
        account_state_with_proof.blob = Some(blob.clone());
    }

    let account_state_with_proof = vec![account_state_with_proof];

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

    let (genesis, _) = generate_genesis_state();
    MockLibraDB {
        version: version as u64,
        genesis,
        all_accounts,
        all_txns,
        events,
        account_state_with_proof,
        timestamps,
    }
}

#[test]
fn test_json_rpc_protocol() {
    let address = format!("0.0.0.0:{}", utils::get_available_port());
    let mock_db = mock_db();
    let mp_sender = channel(1024).0;
    let _runtime = test_bootstrap(address.parse().unwrap(), Arc::new(mock_db), mp_sender);
    let client = reqwest::blocking::Client::new();

    // check that only root path is accessible
    let url = format!("http://{}/fake_path", address);
    let resp = client.get(&url).send().unwrap();
    assert_eq!(resp.status(), 404);

    // only post method is allowed
    let url = format!("http://{}/v1", address);
    let resp = client.get(&url).send().unwrap();
    assert_eq!(resp.status(), 405);

    // empty payload is not allowed
    let resp = client.post(&url).send().unwrap();
    assert_eq!(resp.status(), 400);

    // For now /v1 and / are both supported
    {
        let url_v1 = format!("http://{}", address);
        let resp = client.post(&url_v1).send().unwrap();
        assert_eq!(resp.status(), 400);

        let url_v2 = format!("http://{}/v2", address);
        let resp = client.post(&url_v2).send().unwrap();
        assert_eq!(resp.status(), 404);
    }

    // non json payload
    let resp = client.post(&url).body("non json").send().unwrap();
    assert_eq!(resp.status(), 400);

    // invalid version of protocol
    let request = serde_json::json!({"jsonrpc": "1.0", "method": "add", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(error_code(resp), -32600);

    // invalid request id
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "add", "params": [1, 2], "id": true});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(error_code(resp), -32603);

    // invalid rpc method
    let request = serde_json::json!({"jsonrpc": "2.0", "method": "add", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(error_code(resp), -32601);

    // invalid arguments: too many arguments
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "get_account", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    let error_resp: JsonRpcErrorResponse = resp.json().unwrap();
    assert_eq!(error_resp.error.code, -32602);
    assert_eq!(error_resp.error.message, "Invalid params");

    let invalid_args: InvalidArguments = error_resp.error.as_invalid_arguments().unwrap();
    assert_eq!(invalid_args.required, 1);
    assert_eq!(invalid_args.optional, 0);
    assert_eq!(invalid_args.given, 2);

    // invalid arguments: not enough arguments
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "get_account", "params": [], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(error_code(resp), -32602);

    // invalid arguments: too many arguments for a method has optional arguments
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "get_metadata", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);

    let error_resp: JsonRpcErrorResponse = resp.json().unwrap();
    assert_eq!(error_resp.error.code, -32602);
    assert_eq!(error_resp.error.message, "Invalid params");

    let invalid_args: InvalidArguments = error_resp.error.as_invalid_arguments().unwrap();
    assert_eq!(invalid_args.required, 0);
    assert_eq!(invalid_args.optional, 1);
    assert_eq!(invalid_args.given, 2);

    // Response includes two mandatory field, regardless of errors
    let request =
        serde_json::json!({"jsonrpc": "2.0", "method": "get_account", "params": [1, 2], "id": 1});
    let resp = client.post(&url).json(&request).send().unwrap();
    assert_eq!(resp.status(), 200);
    let data: JsonMap = resp.json().unwrap();
    assert!(data.get(JSONRPC_LIBRA_LEDGER_VERSION).is_some());
    assert!(data.get(JSONRPC_LIBRA_LEDGER_TIMESTAMPUSECS).is_some());
    assert_eq!(
        data.get(JSONRPC_LIBRA_CHAIN_ID).expect("must have"),
        ChainId::test().id()
    );
}

#[test]
fn test_transaction_submission() {
    let (mp_sender, mut mp_events) = channel(1);
    let mock_db = mock_db();
    let port = utils::get_available_port();
    let address = format!("0.0.0.0:{}", port);
    let mut runtime = test_bootstrap(address.parse().unwrap(), Arc::new(mock_db), mp_sender);
    let client = JsonRpcAsyncClient::new(
        reqwest::Url::from_str(format!("http://{}:{}/v1", "127.0.0.1", port).as_str())
            .expect("invalid url"),
    );

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
            let status_code: StatusCode = error.as_status_code().unwrap();
            assert_eq!(status_code, StatusCode::SENDING_ACCOUNT_DOES_NOT_EXIST);
        } else {
            panic!("unexpected error format");
        }
    } else {
        panic!("expected error");
    }
}

#[test]
fn test_get_account() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1024);

    // test case 1: single call
    let (first_account, blob) = mock_db.all_accounts.iter().next().unwrap();
    let expected_resource = AccountState::try_from(blob).unwrap();

    let mut batch = JsonRpcBatch::default();
    batch.add_get_account_request(*first_account);
    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);
    let account = AccountView::optional_from_response(result)
        .unwrap()
        .expect("account does not exist");
    let account_balances: Vec<_> = account.balances.iter().map(|bal| bal.amount).collect();
    let expected_resource_balances: Vec<_> = expected_resource
        .get_balance_resources(&[from_currency_code_string(LBR_NAME).unwrap()])
        .unwrap()
        .iter()
        .map(|(_, bal_resource)| bal_resource.coin())
        .collect();
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
    let mut batch = JsonRpcBatch::default();
    let mut states = vec![];

    for (account, blob) in mock_db.all_accounts.iter() {
        if account == first_account {
            continue;
        }
        states.push(AccountState::try_from(blob).unwrap());
        batch.add_get_account_request(*account);
    }

    let responses = runtime.block_on(client.execute(batch)).unwrap();
    assert_eq!(responses.len(), states.len());

    for (idx, response) in responses.into_iter().enumerate() {
        let account = AccountView::optional_from_response(response.expect("error in response"))
            .unwrap()
            .expect("account does not exist");
        let account_balances: Vec<_> = account.balances.iter().map(|bal| bal.amount).collect();
        let expected_resource_balances: Vec<_> = states[idx]
            .get_balance_resources(&[from_currency_code_string(LBR_NAME).unwrap()])
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
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let (actual_version, actual_timestamp) = mock_db.get_latest_commit_metadata().unwrap();
    let mut batch = JsonRpcBatch::default();
    batch.add_get_metadata_request(None);

    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);

    let result_view = BlockMetadata::from_response(result).unwrap();
    assert_eq!(result_view.version, actual_version);
    assert_eq!(result_view.timestamp, actual_timestamp);
}

#[test]
fn test_get_metadata() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let mut batch = JsonRpcBatch::default();
    batch.add_get_metadata_request(Some(1));

    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);

    let result_view = BlockMetadata::from_response(result).unwrap();
    assert_eq!(result_view.version, 1);
    assert_eq!(result_view.timestamp, mock_db.timestamps[1]);
}

#[test]
fn test_limit_batch_size() {
    let (_, client, mut runtime) = create_database_client_and_runtime(1);

    let mut batch = JsonRpcBatch::default();

    for i in 0..21 {
        batch.add_get_metadata_request(Some(i));
    }

    let ret = runtime.block_on(client.execute(batch));
    assert!(ret.is_err());
    let expected = "JsonRpcError JsonRpcError { code: -32600, message: \"Invalid Request\", data: Some(ExceedSizeLimit(ExceedSizeLimit { limit: 20, size: 21, name: \"batch size\" })) }";
    assert_eq!(ret.unwrap_err().to_string(), expected)
}

#[test]
fn test_get_events_page_limit() {
    let (_, client, mut runtime) = create_database_client_and_runtime(1);

    let mut batch = JsonRpcBatch::default();

    batch.add_get_events_request("event_key".to_string(), 0, 1001);

    let ret = runtime.block_on(client.execute(batch)).unwrap().remove(0);
    assert!(ret.is_err());
    let expected = "JsonRpcError { code: -32600, message: \"Invalid Request\", data: Some(ExceedSizeLimit(ExceedSizeLimit { limit: 1000, size: 1001, name: \"page size\" })) }";
    assert_eq!(ret.unwrap_err().to_string(), expected)
}

#[test]
fn test_get_transactions_page_limit() {
    let (_, client, mut runtime) = create_database_client_and_runtime(1);

    let mut batch = JsonRpcBatch::default();
    batch.add_get_transactions_request(0, 1001, false);

    let ret = runtime.block_on(client.execute(batch)).unwrap().remove(0);
    assert!(ret.is_err());
    let expected = "JsonRpcError { code: -32600, message: \"Invalid Request\", data: Some(ExceedSizeLimit(ExceedSizeLimit { limit: 1000, size: 1001, name: \"page size\" })) }";
    assert_eq!(ret.unwrap_err().to_string(), expected)
}

#[test]
fn test_get_events() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let event_index = 0;
    let mock_db_events = mock_db.events;
    let (first_event_version, first_event) = mock_db_events[event_index].clone();
    let event_key = hex::encode(first_event.key().as_bytes());

    let mut batch = JsonRpcBatch::default();
    batch.add_get_events_request(
        event_key,
        first_event.sequence_number(),
        first_event.sequence_number() + 10,
    );
    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);

    let events = EventView::vec_from_response(result).unwrap();
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
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let version = mock_db.get_latest_version().unwrap();
    let page = 800usize;

    for base_version in (0..version)
        .map(u64::from)
        .take(page)
        .collect::<Vec<_>>()
        .into_iter()
    {
        let mut batch = JsonRpcBatch::default();
        batch.add_get_transactions_request(base_version, page as u64, true);
        let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);
        let txns = TransactionView::vec_from_response(result).unwrap();

        for (i, view) in txns.iter().enumerate() {
            let version = base_version + i as u64;
            assert_eq!(view.version, version);
            let (tx, status) = &mock_db.all_txns[version as usize];
            assert_eq!(view.hash, tx.hash().to_hex());

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
                        assert_eq!(&t.sender().to_string(), sender);
                        assert_eq!(&t.chain_id().id(), chain_id);
                        // TODO: verify every field
                        if let TransactionPayload::Script(s) = t.payload() {
                            assert_eq!(script_hash, &HashValue::sha3_256_of(s.code()).to_hex());
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
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    for (acc, blob) in mock_db.all_accounts.iter() {
        let ar = AccountResource::try_from(blob).unwrap();
        for seq in 1..ar.sequence_number() {
            let mut batch = JsonRpcBatch::default();
            batch.add_get_account_transaction_request(*acc, seq, true);

            let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);
            let tx_view = TransactionView::optional_from_response(result)
                .unwrap()
                .expect("Transaction didn't exists!");

            let (expected_tx, expected_status) = mock_db
                .all_txns
                .iter()
                .find_map(|(t, status)| {
                    if let Ok(x) = t.as_signed_user_txn() {
                        if x.sender() == *acc && x.sequence_number() == seq {
                            assert_eq!(tx_view.hash, t.hash().to_hex());
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
                        assert_eq!(script_hash, HashValue::sha3_256_of(s.code()).to_hex());
                    }
                }
                _ => panic!("wrong type"),
            }
        }
    }
}

#[test]
fn test_get_account_transactions() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    for (acc, blob) in mock_db.all_accounts.iter() {
        let total = AccountResource::try_from(blob).unwrap().sequence_number();

        let mut batch = JsonRpcBatch::default();
        batch.add_get_account_transactions_request(*acc, 0, max(1, min(1000, total * 2)), true);

        let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);
        let tx_views = TransactionView::vec_from_response(result).unwrap();
        assert_eq!(tx_views.len() as u64, total);
    }
}
#[test]
// Check that if version and ledger_version parameters are None, then the server returns the latest
// known state.
fn test_get_account_state_with_proof_null_versions() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let account = get_first_account_from_mock_db(&mock_db);
    let mut batch = JsonRpcBatch::default();
    batch.add_get_account_state_with_proof_request(account, None, None);

    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);

    let received_proof = AccountStateWithProofView::from_response(result).unwrap();
    let expected_proof = get_first_state_proof_from_mock_db(&mock_db);

    // Check latest version returned, when no version specified
    assert_eq!(received_proof.version, expected_proof.version);
}

#[test]
fn test_get_account_state_with_proof() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let account = get_first_account_from_mock_db(&mock_db);
    let mut batch = JsonRpcBatch::default();
    batch.add_get_account_state_with_proof_request(account, Some(0), Some(0));

    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);

    let received_proof = AccountStateWithProofView::from_response(result).unwrap();
    let expected_proof = get_first_state_proof_from_mock_db(&mock_db);
    let expected_blob = expected_proof.blob.as_ref().unwrap();
    let expected_sm_proof = expected_proof.proof.transaction_info_to_account_proof();
    let expected_txn_info_with_proof = expected_proof.proof.transaction_info_with_proof();

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
    let li_proof: TransactionAccumulatorProof = lcs::from_bytes(
        &received_proof
            .proof
            .ledger_info_to_transaction_info_proof
            .into_bytes()
            .unwrap(),
    )
    .unwrap();
    let txn_info_with_proof = TransactionInfoWithProof::new(li_proof, txn_info);
    assert_eq!(txn_info_with_proof, *expected_txn_info_with_proof);
}

#[test]
fn test_get_state_proof() {
    let (mock_db, client, mut runtime) = create_database_client_and_runtime(1024);

    let version = mock_db.version;
    let mut batch = JsonRpcBatch::default();
    batch.add_get_state_proof_request(version);
    let result = execute_batch_and_get_first_response(&client, &mut runtime, batch);
    let proof = StateProofView::from_response(result).unwrap();
    let li: LedgerInfoWithSignatures =
        lcs::from_bytes(&proof.ledger_info_with_signatures.into_bytes().unwrap()).unwrap();
    assert_eq!(li.ledger_info().version(), version);
}

#[test]
fn test_get_network_status() {
    let (_mock_db, client, mut runtime) = create_database_client_and_runtime(1);

    let mut batch = JsonRpcBatch::default();
    batch.add_get_network_status_request();

    if let JsonRpcResponse::NetworkStatusResponse(connected_peers) =
        execute_batch_and_get_first_response(&client, &mut runtime, batch)
    {
        // expect no connected peers when no network is running
        assert_eq!(connected_peers.as_u64().unwrap(), 0);
    } else {
        panic!("did not receive expected json rpc response");
    }
}

/// Creates and returns a MockLibraDB, JsonRpcAsyncClient and corresponding server Runtime tuple for
/// testing. The given channel_buffer specifies the buffer size of the mempool client sender channel.
fn create_database_client_and_runtime(
    channel_buffer: usize,
) -> (MockLibraDB, JsonRpcAsyncClient, Runtime) {
    let mock_db = mock_db();

    let host = "0.0.0.0";
    let port = utils::get_available_port();
    let address = format!("{}:{}", host, port);
    let mp_sender = channel(channel_buffer).0;

    let runtime = test_bootstrap(
        address.parse().unwrap(),
        Arc::new(mock_db.clone()),
        mp_sender,
    );
    let client = JsonRpcAsyncClient::new(
        reqwest::Url::from_str(format!("http://127.0.0.1:{}/v1", port).as_str())
            .expect("invalid url"),
    );

    (mock_db, client, runtime)
}

/// Returns the first account address stored in the given mock database.
fn get_first_account_from_mock_db(mock_db: &MockLibraDB) -> AccountAddress {
    *mock_db
        .all_accounts
        .keys()
        .next()
        .expect("mock DB missing account")
}

/// Returns the first account_state_with_proof stored in the given mock database.
fn get_first_state_proof_from_mock_db(mock_db: &MockLibraDB) -> AccountStateWithProof {
    mock_db
        .account_state_with_proof
        .get(0)
        .expect("mock DB missing account state with proof")
        .clone()
}

/// Executes the given JsonRPCBatch using the specified JsonRpcAsyncClient and Runtime, and returns
/// the first JsonRpcResponse produced for the batch.
fn execute_batch_and_get_first_response(
    client: &JsonRpcAsyncClient,
    runtime: &mut Runtime,
    batch: JsonRpcBatch,
) -> JsonRpcResponse {
    runtime
        .block_on(client.execute(batch))
        .unwrap()
        .remove(0)
        .unwrap()
}

fn error_code(resp: reqwest::blocking::Response) -> i16 {
    let err_resp: JsonRpcErrorResponse = resp.json().unwrap();
    err_resp.error.code
}
