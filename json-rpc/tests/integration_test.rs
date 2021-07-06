// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use diem_json_rpc_types::views::AccountTransactionsWithProofView;
use diem_types::transaction::AccountTransactionsWithProof;
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
        // no test after this one, as your scripts may not in allow list.
        // add test before above test
    ]
}
