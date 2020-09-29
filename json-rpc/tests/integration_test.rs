// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use libra_crypto::hash::CryptoHash;
use libra_types::{
    account_config::coin1_tmp_tag,
    transaction::{Transaction, TransactionPayload},
};
use transaction_builder_generated::stdlib;

mod node;
mod testing;

#[test]
fn test_interface() {
    libra_logger::LibraLogger::init_for_testing();
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
            name: "currency info",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_currencies", json!([]));
                assert_eq!(
                    resp.result.unwrap(),
                    json!([
                      {
                        "burn_events_key": "06000000000000000000000000000000000000000a550c18",
                        "cancel_burn_events_key": "08000000000000000000000000000000000000000a550c18",
                        "code": "Coin1",
                        "exchange_rate_update_events_key": "09000000000000000000000000000000000000000a550c18",
                        "fractional_part": 100,
                        "mint_events_key": "05000000000000000000000000000000000000000a550c18",
                        "preburn_events_key": "07000000000000000000000000000000000000000a550c18",
                        "scaling_factor": 1000000,
                        "to_lbr_exchange_rate": 1.0,
                      },
                      {
                        "burn_events_key": "0b000000000000000000000000000000000000000a550c18",
                        "cancel_burn_events_key": "0d000000000000000000000000000000000000000a550c18",
                        "code": "LBR",
                        "exchange_rate_update_events_key": "0e000000000000000000000000000000000000000a550c18",
                        "fractional_part": 1000,
                        "mint_events_key": "0a000000000000000000000000000000000000000a550c18",
                        "preburn_events_key": "0c000000000000000000000000000000000000000a550c18",
                        "scaling_factor": 1000000,
                        "to_lbr_exchange_rate": 1.0
                      }
                    ])
                )
            },
        },
        Test {
            name: "block metadata",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_metadata", json!([]));
                let metadata = resp.result.unwrap();
                assert_eq!(metadata["chain_id"], resp.libra_chain_id);
                assert_eq!(metadata["timestamp"], resp.libra_ledger_timestampusec);
                assert_eq!(metadata["version"], resp.libra_ledger_version);
                assert_eq!(metadata["chain_id"], 4);
                assert_ne!(resp.libra_ledger_timestampusec, 0);
                assert_ne!(resp.libra_ledger_version, 0);
            },
        },
        Test {
            name: "unknown role type account",
            run: |env: &mut testing::Env| {
                let address = libra_types::account_config::libra_root_address().to_string();
                let resp = env.send("get_account", json!([address]));
                let mut result = resp.result.unwrap();
                // as we generate account auth key, ignore it in assertion
                assert_ne!(result["authentication_key"].as_str().unwrap(), "");
                result["authentication_key"] = json!(null);
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": null,
                        "balances": [],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": "02000000000000000000000000000000000000000a550c18",
                        "role": { "type": "unknown" },
                        "sent_events_key": "03000000000000000000000000000000000000000a550c18",
                        "sequence_number": 1
                    }),
                );
            },
        },
        Test {
            name: "designated_dealer role type account",
            run: |env: &mut testing::Env| {
                let address = libra_types::account_config::testnet_dd_account_address().to_string();
                let resp = env.send("get_account", json!([address]));
                let mut result = resp.result.unwrap();
                // as we generate account auth key, ignore it in assertion
                assert_ne!(result["authentication_key"].as_str().unwrap(), "");
                result["authentication_key"] = json!(null);
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": null,
                        "balances": [
                            {
                                "amount": 9223370036854775807 as u64,
                                "currency": "Coin1"
                            },
                            {
                                "amount": 0 as u64,
                                "currency": "LBR"
                            },
                        ],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": "0300000000000000000000000000000000000000000000dd",
                        "role": {
                            "type": "designated_dealer",
                            "base_url": "",
                            "compliance_key": "",
                            "expiration_time": 18446744073709551615 as u64,
                            "human_name": "moneybags",
                            "preburn_balances": [
                                {
                                    "amount": 0,
                                    "currency": "Coin1"
                                },
                            ],
                            "received_mint_events_key": "0000000000000000000000000000000000000000000000dd",
                            "compliance_key_rotation_events_key": "0100000000000000000000000000000000000000000000dd",
                            "base_url_rotation_events_key": "0200000000000000000000000000000000000000000000dd",
                        },
                        "sent_events_key": "0400000000000000000000000000000000000000000000dd",
                        "sequence_number": 2
                    }),
                );
            },
        },
        Test {
            name: "parent vasp role type account",
            run: |env: &mut testing::Env| {
                let account = &env.vasps[0];
                let address = account.address.to_string();
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": account.auth_key().to_string(),
                        "balances": [{"amount": 997000000000 as u64, "currency": "Coin1"}],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0200000000000000{}", address),
                        "role": {
                            "base_url": "",
                            "base_url_rotation_events_key": format!("0100000000000000{}", address),
                            "compliance_key": "",
                            "compliance_key_rotation_events_key": format!("0000000000000000{}", address),
                            "expiration_time": 18446744073709551615 as u64,
                            "human_name": "Novi 0",
                            "num_children": 1,
                            "type": "parent_vasp"
                        },
                        "sent_events_key": format!("0300000000000000{}", address),
                        "sequence_number": 1
                    }),
                );
            },
        },
        Test {
            name: "child vasp role type account",
            run: |env: &mut testing::Env| {
                let parent = &env.vasps[0];
                let account = &env.vasps[0].children[0];
                let address = account.address.to_string();
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": account.auth_key().to_string(),
                        "balances": [{"amount": 3000000000 as u64, "currency": "Coin1"}],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0000000000000000{}", address),
                        "role": {
                            "type": "child_vasp",
                            "parent_vasp_address": parent.address.to_string(),
                        },
                        "sent_events_key": format!("0100000000000000{}", address),
                        "sequence_number": 0
                    }),
                );
            },
        },
        Test {
            name: "peer to peer account transaction with events",
            run: |env: &mut testing::Env| {
                let prev_ledger_version = env.send("get_metadata", json!([])).libra_ledger_version;

                let txn = env.transfer_coins((0, 0), (1, 0), 200000);
                env.wait_for_txn(&txn);
                let txn_hex = hex::encode(lcs::to_bytes(&txn).expect("lcs txn failed"));

                let sender = &env.vasps[0].children[0];
                let receiver = &env.vasps[1].children[0];

                let resp = env.send(
                    "get_account_transaction",
                    json!([sender.address.to_string(), 0, true]),
                );
                let result = resp.result.unwrap();
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    true,
                    version > prev_ledger_version && version <= resp.libra_ledger_version
                );

                let gas_used = result["gas_used"].as_u64().expect("exist as u64");
                let txn_hash = Transaction::UserTransaction(txn.clone()).hash().to_hex();

                let script = match txn.payload() {
                    TransactionPayload::Script(s) => s,
                    _ => unreachable!(),
                };
                let script_hash = libra_crypto::HashValue::sha3_256_of(script.code()).to_hex();
                let script_bytes = hex::encode(lcs::to_bytes(script).unwrap());

                assert_eq!(
                    result,
                    json!({
                        "bytes": format!("00{}", txn_hex),
                        "events": [
                            {
                                "data": {
                                    "amount": {"amount": 200000 as u64, "currency": "Coin1"},
                                    "metadata": "",
                                    "receiver": receiver.address.to_string(),
                                    "sender": sender.address.to_string(),
                                    "type": "sentpayment"
                                },
                                "key": format!("0100000000000000{}", sender.address.to_string()),
                                "sequence_number": 0,
                                "transaction_version": version,
                            },
                            {
                                "data": {
                                    "amount": {"amount": 200000 as u64, "currency": "Coin1"},
                                    "metadata": "",
                                    "receiver": receiver.address.to_string(),
                                    "sender": sender.address.to_string(),
                                    "type": "receivedpayment"
                                },
                                "key": format!("0000000000000000{}", receiver.address.to_string()),
                                "sequence_number": 1,
                                "transaction_version": version
                            }
                        ],
                        "gas_used": gas_used,
                        "hash": txn_hash,
                        "transaction": {
                            "chain_id": 4,
                            "expiration_timestamp_secs": txn.expiration_timestamp_secs(),
                            "gas_currency": "Coin1",
                            "gas_unit_price": 0,
                            "max_gas_amount": 1000000,
                            "public_key": sender.public_key.to_string(),
                            "script": {
                                "amount": 200000,
                                "currency": "Coin1",
                                "metadata": "",
                                "metadata_signature": "",
                                "receiver": receiver.address.to_string(),
                                "type": "peer_to_peer_transaction"
                            },
                            "script_bytes": script_bytes,
                            "script_hash": script_hash,
                            "sender": sender.address.to_string(),
                            "sequence_number": 0,
                            "signature": hex::encode(txn.authenticator().signature_bytes()),
                            "signature_scheme": "Scheme::Ed25519",
                            "type": "user"
                        },
                        "version": version,
                        "vm_status": {"type": "executed"}
                    }),
                    "{:#}",
                    result,
                );
            },
        },
        Test {
            name: "peer to peer account failed error explanations",
            run: |env: &mut testing::Env| {
                env.allow_execution_failures(|env: &mut testing::Env| {
                    // transfer too many coins
                    env.transfer_coins((0, 0), (1, 0), 200000000000000);
                });

                let sender = &env.vasps[0].children[0];

                let resp = env.send(
                    "get_account_transaction",
                    json!([sender.address.to_string(), 1, true]),
                );
                let result = resp.result.unwrap();
                let vm_status = result["vm_status"].clone();
                assert_eq!(
                    vm_status,
                    json!({
                        "abort_code": 1288,
                        "explanation": {
                            "category": "LIMIT_EXCEEDED",
                            "category_description": " A limit on an amount, e.g. a currency, is exceeded. Example: withdrawal of money after account limits window\n is exhausted.",
                            "reason": "EINSUFFICIENT_BALANCE",
                            "reason_description": " The account does not hold a large enough balance in the specified currency"
                        },
                        "location": "00000000000000000000000000000001::LibraAccount",
                        "type": "move_abort"
                    })
                );
            },
        },
        Test {
            name: "invalid transaction submitted: mempool validation error",
            run: |env: &mut testing::Env| {
                env.allow_execution_failures(|env: &mut testing::Env| {
                    let txn1 = env.transfer_coins_txn((0, 0), (1, 0), 200);
                    let txn2 = env.transfer_coins_txn((0, 0), (1, 0), 200);
                    env.submit(&txn1);
                    let resp = env.submit(&txn2);
                    assert_eq!(
                        resp.error.expect("error").message,
                        "Server error: Mempool submission error: \"Failed to update gas price to 0\"".to_string(),
                    );
                    // wait submitted transaction executed, so that the following tests won't have
                    // problem when need to submit same account transaction.
                    env.wait_for_txn(&txn1);
                });
            },
        },
        Test {
            name: "preburn & burn events",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_preburn_script(coin1_tmp_tag(), 100);
                let txn = env.create_txn(&env.dd, script);
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();

                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data": {
                                "amount": {"amount": 100, "currency": "Coin1"},
                                "metadata": "",
                                "receiver": "000000000000000000000000000000dd",
                                "sender": "000000000000000000000000000000dd",
                                "type": "sentpayment"
                            },
                            "key": "0400000000000000000000000000000000000000000000dd",
                            "sequence_number": 2,
                            "transaction_version": version
                        },
                        {
                            "data": {
                                "amount": {"amount": 100, "currency": "Coin1"},
                                "preburn_address": "000000000000000000000000000000dd",
                                "type": "preburn"
                            },
                            "key": "07000000000000000000000000000000000000000a550c18",
                            "sequence_number": 0,
                            "transaction_version": version
                        }
                    ]),
                    "{}",
                    result["events"]
                );

                let burn_txn = env.create_txn(
                    &env.tc,
                    stdlib::encode_burn_script(coin1_tmp_tag(), 0, env.dd.address),
                );
                let result = env.submit_and_wait(burn_txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([{
                        "data":{
                            "amount":{"amount":100,"currency":"Coin1"},
                            "preburn_address":"000000000000000000000000000000dd",
                            "type":"burn"
                        },
                        "key":"06000000000000000000000000000000000000000a550c18",
                        "sequence_number":0,
                        "transaction_version":version
                    }]),
                    "{}",
                    result["events"]
                );
            },
        },
        Test {
            name: "cancel burn event",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_preburn_script(coin1_tmp_tag(), 100);
                let txn = env.create_txn(&env.dd, script);
                env.submit_and_wait(txn);

                let cancel_burn_txn = env.create_txn(
                    &env.tc,
                    stdlib::encode_cancel_burn_script(coin1_tmp_tag(), env.dd.address),
                );
                let result = env.submit_and_wait(cancel_burn_txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "amount":{"amount":100,"currency":"Coin1"},
                                "preburn_address":"000000000000000000000000000000dd",
                                "type":"cancelburn"
                            },
                            "key":"08000000000000000000000000000000000000000a550c18",
                            "sequence_number":0,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "amount":{"amount":100,"currency":"Coin1"},
                                "metadata":"",
                                "receiver":"000000000000000000000000000000dd",
                                "sender":"000000000000000000000000000000dd",
                                "type":"receivedpayment"
                            },
                            "key":"0300000000000000000000000000000000000000000000dd",
                            "sequence_number":1,
                            "transaction_version":version
                        }
                    ]),
                    "{}",
                    result["events"]
                );
            },
        },
        Test {
            name: "update exchange rate event",
            run: |env: &mut testing::Env| {
                let txn = env.create_txn(
                    &env.tc,
                    stdlib::encode_update_exchange_rate_script(coin1_tmp_tag(), 0, 1, 4),
                );
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([{
                        "data":{
                            "currency_code":"Coin1",
                            "new_to_lbr_exchange_rate":0.25,
                            "type":"to_lbr_exchange_rate_update"
                        },
                        "key":"09000000000000000000000000000000000000000a550c18",
                        "sequence_number":0,
                        "transaction_version":version
                    }]),
                    "{}",
                    result["events"]
                );
            },
        },
        Test {
            name: "mint & received mint events",
            run: |env: &mut testing::Env| {
                let txn = env.create_txn(
                    &env.tc,
                    stdlib::encode_tiered_mint_script(
                        coin1_tmp_tag(),
                        0,
                        env.dd.address,
                        1_000_000,
                        1,
                    ),
                );
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"Coin1"},
                                "destination_address":"000000000000000000000000000000dd",
                                "type":"receivedmint"
                            },
                            "key":"0000000000000000000000000000000000000000000000dd",
                            "sequence_number":1,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"Coin1"},
                                "type":"mint"
                            },
                            "key":"05000000000000000000000000000000000000000a550c18",
                            "sequence_number":1,
                            "transaction_version":version},
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"Coin1"},
                                "metadata":"",
                                "receiver":"000000000000000000000000000000dd",
                                "sender":"00000000000000000000000000000000",
                                "type":"receivedpayment"
                            },
                            "key":"0300000000000000000000000000000000000000000000dd",
                            "sequence_number":2,
                            "transaction_version":version

                        }
                    ]),
                    "{}",
                    result["events"]
                );
            },
        },
        Test {
            name: "rotate compliance key rotation events",
            run: |env: &mut testing::Env| {
                let private_key = generate_key::generate_key();
                let public_key: libra_crypto::ed25519::Ed25519PublicKey = (&private_key).into();
                let txn = env.create_txn(
                    &env.vasps[0],
                    stdlib::encode_rotate_dual_attestation_info_script(
                        b"http://hello.com".to_vec(),
                        public_key.to_bytes().to_vec(),
                    ),
                );
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                let rotated_seconds = result["events"][0]["data"]["time_rotated_seconds"]
                    .as_u64()
                    .unwrap();
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "new_base_url":"http://hello.com",
                                "time_rotated_seconds": rotated_seconds,
                                "type":"baseurlrotation"
                            },
                            "key": format!("0100000000000000{}", env.vasps[0].address.to_string()),
                            "sequence_number":0,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "new_compliance_public_key": hex::encode(public_key.to_bytes()),
                                "time_rotated_seconds": rotated_seconds,
                                "type":"compliancekeyrotation"
                            },
                            "key": format!("0000000000000000{}", env.vasps[0].address.to_string()),
                            "sequence_number":0,
                            "transaction_version":version
                        }
                    ]),
                    "{}",
                    result["events"]
                );
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
    ]
}
