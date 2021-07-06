// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use diem_crypto::hash::HashValue;
use diem_json_rpc_types::views::{
    AccountTransactionsWithProofView, AccumulatorConsistencyProofView,
};
use diem_transaction_builder::stdlib::{
    self, encode_rotate_authentication_key_with_nonce_admin_script,
    encode_rotate_authentication_key_with_nonce_admin_script_function,
};
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    on_chain_config::DIEM_MAX_KNOWN_VERSION,
    proof::{AccumulatorConsistencyProof, TransactionAccumulatorSummary},
    transaction::{AccountTransactionsWithProof, ChangeSet, TransactionPayload, WriteSetPayload},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::{convert::TryFrom, str::FromStr};

mod node;
mod testing;

#[test]
fn test_interface() {
    diem_logger::DiemLogger::init_for_testing();
    let fullnode = node::Node::start().unwrap();
    fullnode.wait_for_jsonrpc_connectivity();

    let mut env = testing::Env::gen(fullnode.root_key.clone(), fullnode.url());
    // setup 2 vasps, it generates some transactions for tests, so that some query tests
    // can be very simple, change this will affect some tests result.
    env.init_vasps(2);
    for t in create_test_cases() {
        print!("run {}: ", t.name);
        (t.run)(&mut env);
        println!("success!");
    }
}

pub struct Test {
    pub name: &'static str,
    pub run: fn(&mut testing::Env),
}

fn create_test_cases() -> Vec<Test> {
    vec![
        Test {
            name: "Upgrade diem version",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_update_diem_version_script(
                    0,
                    DIEM_MAX_KNOWN_VERSION.major + 1,
                );
                let txn = env.create_txn(&env.root, script);
                env.submit_and_wait(txn);
            },
        },
        Test {
            name: "upgrade event & newepoch",
            run: |env: &mut testing::Env| {
                let write_set = ChangeSet::new(create_common_write_set(), vec![]);
                let txn = env.create_txn_by_payload(
                    &env.root,
                    TransactionPayload::WriteSet(WriteSetPayload::Direct(write_set)),
                );
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                let committed_time = result["events"][0]["data"]["committed_timestamp_secs"]
                    .as_u64()
                    .unwrap();
                assert!(committed_time != 0);
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "type": "admintransaction",
                                "committed_timestamp_secs": committed_time,
                            },
                            "key": "01000000000000000000000000000000000000000a550c18",
                            "sequence_number": 0,
                            "transaction_version": version
                        },
                        {
                            "data":{
                                "epoch": 3,
                                "type": "newepoch"
                            },
                            "key": "04000000000000000000000000000000000000000a550c18",
                            "sequence_number": 2,
                            "transaction_version": version
                        }
                    ]),
                    "{}",
                    result["events"]
                );
            },
        },
        Test {
            name: "get_account_transactions_with_proofs",
            run: |env: &mut testing::Env| {
                let sender = &env.vasps[0].children[0];
                let response = env.send(
                    "get_account_transactions_with_proofs",
                    json!([sender.address.to_string(), 0, 1000, false]),
                );
                // Just check that the responses deserialize correctly, we'll let
                // the verifying client smoke tests handle the proof checking.
                let value = response.result.unwrap();
                let view = serde_json::from_value::<AccountTransactionsWithProofView>(value).unwrap();
                let _txns = AccountTransactionsWithProof::try_from(&view).unwrap();
            },
        },
        Test {
            name: "no unknown events so far",
            run: |env: &mut testing::Env| {
                let response = env.send("get_transactions", json!([0, 1000, true]));
                let txns = response.result.unwrap();
                for txn in txns.as_array().unwrap() {
                    for event in txn["events"].as_array().unwrap() {
                        let event_type = event["data"]["type"].as_str().unwrap();
                        assert_ne!(event_type, "unknown", "{}", event);
                    }
                }
            },
        },
        Test {
            name: "multi-agent payment script meets dual attestation limit",
            run: |env: &mut testing::Env| {
                let sender = env.vasps[0].children[0].clone();
                let receiver = env.vasps[1].children[0].clone();
                let sender_balance = env.get_balance(&sender, "XUS");
                let receiver_balance = env.get_balance(&receiver, "XUS");

                let limit = env.get_metadata()["dual_attestation_limit"].as_u64().unwrap();
                let amount = limit + 1;
                let txn = env.multi_agent_payment_txn((0, 0), (1, 0), amount);

                let txn_view = env.submit_and_wait(txn);

                let events = txn_view["events"].as_array().unwrap();
                assert_eq!(events.len(), 2);
                assert_eq!(events[0]["data"]["type"], "sentpayment");
                assert_eq!(events[1]["data"]["type"], "receivedpayment");
                for event in events.iter() {
                    assert_eq!(event["data"]["amount"], json!({"amount": amount, "currency": "XUS"}));
                    assert_eq!(event["data"]["sender"], format!("{:x}", &sender.address));
                    assert_eq!(event["data"]["receiver"], format!("{:x}", &receiver.address));
                }

                assert_eq!(sender_balance - amount, env.get_balance(&sender, "XUS"));
                assert_eq!(receiver_balance + amount, env.get_balance(&receiver, "XUS"));
            },
        },
        Test {
            name: "multi-agent transaction with rotate_authentication_key_with_nonce_admin script function",
            run: |env: &mut testing::Env| {
                let root = env.root.clone();
                let account = env.vasps[0].children[0].clone();
                let private_key = generate_key::generate_key();
                let public_key: diem_crypto::ed25519::Ed25519PublicKey = (&private_key).into();
                let txn = env.create_multi_agent_txn(
                    &root,
                    vec![&account],
                    encode_rotate_authentication_key_with_nonce_admin_script_function(
                        0, public_key.to_bytes().to_vec()),
                );
                env.submit_and_wait(txn.clone());
                let resp = env.send(
                    "get_account_transaction",
                    json!([root.address.to_string(), 3, true]),
                );
                let result = resp.result.unwrap();
                let script = match txn.payload() {
                    TransactionPayload::ScriptFunction(s) => s,
                    _ => unreachable!(),
                };
                let script_hash = diem_crypto::HashValue::zero().to_hex();
                let script_bytes = hex::encode(bcs::to_bytes(script).unwrap());
                assert_eq!(result["vm_status"], json!({"type": "executed"}));
                assert_eq!(
                    result["transaction"],
                    json!({
                        "type": "user",
                        "sender": format!("{:x}", &root.address),
                        "signature_scheme": "Scheme::Ed25519",
                        "signature": hex::encode(txn.authenticator().sender().signature_bytes()),
                        "public_key": root.public_key.to_string(),
                        "secondary_signers": [ format!("{:x}", &account.address) ],
                        "secondary_signature_schemes": [ "Scheme::Ed25519" ],
                        "secondary_signatures": [ hex::encode(txn.authenticator().secondary_signers()[0].signature_bytes())],
                        "secondary_public_keys": [ account.public_key.to_string() ],
                        "sequence_number": 3,
                        "chain_id": 4,
                        "max_gas_amount": 1000000,
                        "gas_unit_price": 0,
                        "gas_currency": "XUS",
                        "expiration_timestamp_secs": txn.expiration_timestamp_secs(),
                        "script_hash": script_hash,
                        "script_bytes": script_bytes,
                        "script": {
                            "type": "script_function",
                            "arguments_bcs": vec![ "0000000000000000", &hex::encode(bcs::to_bytes(&public_key).unwrap())],
                            "type_arguments": [],
                            "module_address": "00000000000000000000000000000001",
                            "module_name": "AccountAdministrationScripts",
                            "function_name": "rotate_authentication_key_with_nonce_admin"
                        },
                    }),
                );
            },
        },
        Test {
            name: "multi-agent transaction with rotate_authentication_key_with_nonce_admin script",
            run: |env: &mut testing::Env| {
                let root = env.root.clone();
                let account = env.vasps[1].children[0].clone();
                let private_key = generate_key::generate_key();
                let public_key: diem_crypto::ed25519::Ed25519PublicKey = (&private_key).into();
                let txn = env.create_multi_agent_txn(
                    &root,
                    vec![&account],
                    TransactionPayload::Script(encode_rotate_authentication_key_with_nonce_admin_script(
                        0, public_key.to_bytes().to_vec())),
                );
                env.submit_and_wait(txn.clone());
                let resp = env.send(
                    "get_account_transaction",
                    json!([root.address.to_string(), 4, true]),
                );
                let result = resp.result.unwrap();
                let script = match txn.payload() {
                    TransactionPayload::Script(s) => s,
                    _ => unreachable!(),
                };
                let script_hash = diem_crypto::HashValue::sha3_256_of(script.code()).to_hex();
                let script_bytes = hex::encode(bcs::to_bytes(script).unwrap());
                assert_eq!(result["vm_status"], json!({"type": "executed"}));
                assert_eq!(
                    result["transaction"],
                    json!({
                        "type": "user",
                        "sender": format!("{:x}", &root.address),
                        "signature_scheme": "Scheme::Ed25519",
                        "signature": hex::encode(txn.authenticator().sender().signature_bytes()),
                        "public_key": root.public_key.to_string(),
                        "secondary_signers": [ format!("{:x}", &account.address) ],
                        "secondary_signature_schemes": [ "Scheme::Ed25519" ],
                        "secondary_signatures": [ hex::encode(txn.authenticator().secondary_signers()[0].signature_bytes())],
                        "secondary_public_keys": [ account.public_key.to_string() ],
                        "sequence_number": 4,
                        "chain_id": 4,
                        "max_gas_amount": 1000000,
                        "gas_unit_price": 0,
                        "gas_currency": "XUS",
                        "expiration_timestamp_secs": txn.expiration_timestamp_secs(),
                        "script_hash": script_hash,
                        "script_bytes": script_bytes,
                        "script": {
                            "type_arguments": [],
                            "arguments": [
                                "{U64: 0}",
                                format!("{{U8Vector: 0x{}}}", public_key.to_string()),
                            ],
                            "code": hex::encode(script.code()),
                            "type": "rotate_authentication_key_with_nonce_admin"
                        },
                    }),
                );
            },
        },
        Test {
            name: "get_accumulator_consistency_proof",
            run: |env: &mut testing::Env| {
                // batch request
                let resp = env.send_request(json!([
                    {"jsonrpc": "2.0", "method": "get_metadata", "params": [], "id": 1},
                    // leave both params empty to get the full accumulator summary
                    {"jsonrpc": "2.0", "method": "get_accumulator_consistency_proof", "params": [], "id": 2},
                ]));

                // extract both responses
                let resps: Vec<serde_json::Value> = serde_json::from_value(resp).expect("should be valid serde_json::Value");
                let metadata = &resps.iter().find(|g| g["id"] == 1).unwrap()["result"];
                let proof_view = &resps.iter().find(|g| g["id"] == 2).unwrap()["result"];

                // get the root hash and version from the metadata response
                let metadata_root_hash = HashValue::from_str(metadata["accumulator_root_hash"].as_str().unwrap()).unwrap();
                let version = metadata["version"].as_u64().unwrap();

                // parse the consistency proof and build the accumulator
                let proof_view = serde_json::from_value::<AccumulatorConsistencyProofView>(proof_view.clone()).unwrap();
                let proof = AccumulatorConsistencyProof::try_from(&proof_view).unwrap();
                let accumulator = TransactionAccumulatorSummary::try_from_genesis_proof(proof, version).unwrap();

                // root hash from metadata and the computed root hash from the
                // accumulator summary should match
                assert_eq!(metadata_root_hash, accumulator.root_hash());
            },
        },
        // no test after this one, as your scripts may not in allow list.
        // add test before above test
    ]
}

fn create_common_write_set() -> WriteSet {
    WriteSetMut::new(vec![(
        AccessPath::new(
            AccountAddress::new([
                0xc4, 0xc6, 0x3f, 0x80, 0xc7, 0x4b, 0x11, 0x26, 0x3e, 0x42, 0x1e, 0xbf, 0x84, 0x86,
                0xa4, 0xe3,
            ]),
            vec![0x01, 0x21, 0x7d, 0xa6, 0xc6, 0xb3, 0xe1, 0x9f, 0x18],
        ),
        WriteOp::Value(vec![0xca, 0xfe, 0xd0, 0x0d]),
    )])
    .freeze()
    .unwrap()
}
