// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use diem_json_rpc_types::views::AccountTransactionsWithProofView;
use diem_transaction_builder::stdlib;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    on_chain_config::DIEM_MAX_KNOWN_VERSION,
    transaction::{AccountTransactionsWithProof, ChangeSet, TransactionPayload, WriteSetPayload},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::convert::TryFrom;

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
                let script =
                    stdlib::encode_update_diem_version_script(0, DIEM_MAX_KNOWN_VERSION.major + 1);
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
                let view =
                    serde_json::from_value::<AccountTransactionsWithProofView>(value).unwrap();
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
