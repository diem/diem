// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde_json::json;

use diem_crypto::hash::CryptoHash;
use diem_transaction_builder::stdlib;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::xus_tag,
    contract_event::EventWithProof,
    diem_id_identifier::DiemIdVaspDomainIdentifier,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::DIEM_MAX_KNOWN_VERSION,
    proof::TransactionAccumulatorRangeProof,
    transaction::{ChangeSet, Transaction, TransactionInfo, TransactionPayload, WriteSetPayload},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use std::{convert::TryInto, ops::Deref};

use diem_json_rpc_types::views::EventView;
use diem_transaction_builder::stdlib::{
    encode_rotate_authentication_key_with_nonce_admin_script,
    encode_rotate_authentication_key_with_nonce_admin_script_function,
};

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
            name: "currency info",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_currencies", json!([]));
                assert_eq!(
                    resp.result.unwrap(),
                    json!([
                        {
                            "burn_events_key": "06000000000000000000000000000000000000000a550c18",
                            "cancel_burn_events_key": "08000000000000000000000000000000000000000a550c18",
                            "code": "XUS",
                            "exchange_rate_update_events_key": "09000000000000000000000000000000000000000a550c18",
                            "fractional_part": 100,
                            "mint_events_key": "05000000000000000000000000000000000000000a550c18",
                            "preburn_events_key": "07000000000000000000000000000000000000000a550c18",
                            "scaling_factor": 1000000,
                            "to_xdx_exchange_rate": 1.0,
                        },
                        {
                            "burn_events_key": "0b000000000000000000000000000000000000000a550c18",
                            "cancel_burn_events_key": "0d000000000000000000000000000000000000000a550c18",
                            "code": "XDX",
                            "exchange_rate_update_events_key": "0e000000000000000000000000000000000000000a550c18",
                            "fractional_part": 1000,
                            "mint_events_key": "0a000000000000000000000000000000000000000a550c18",
                            "preburn_events_key": "0c000000000000000000000000000000000000000a550c18",
                            "scaling_factor": 1000000,
                            "to_xdx_exchange_rate": 1.0
                        }
                    ])
                )
            },
        },
        Test {
            name: "block metadata",
            run: |env: &mut testing::Env| {
                // batch request
                let resp = env.send_request(json!([
                    {"jsonrpc": "2.0", "method": "get_metadata", "params": [], "id": 1},
                    {"jsonrpc": "2.0", "method": "get_state_proof", "params": [0], "id": 2}
                ]));

                // extract both responses
                let resps: Vec<serde_json::Value> = serde_json::from_value(resp).expect("should be valid serde_json::Value");
                let metadata = &resps.iter().find(|g| g["id"] == 1).unwrap()["result"];
                let state_proof = &resps.iter().find(|g| g["id"] == 2).unwrap()["result"];

                // extract header and ensure they match in both responses
                let diem_chain_id = &resps[0]["diem_chain_id"];
                let diem_ledger_timestampusec = &resps[0]["diem_ledger_timestampusec"];
                let diem_ledger_version = &resps[0]["diem_ledger_version"];

                assert_eq!(diem_chain_id, &resps[1]["diem_chain_id"]);
                assert_eq!(diem_ledger_timestampusec, &resps[1]["diem_ledger_timestampusec"]);
                assert_eq!(diem_ledger_version, &resps[1]["diem_ledger_version"]);

                // parse metadata
                assert_eq!(&metadata["chain_id"], diem_chain_id);
                assert_eq!(&metadata["timestamp"], diem_ledger_timestampusec);
                assert_eq!(&metadata["version"], diem_ledger_version);
                assert_eq!(metadata["chain_id"], 4);
                // All genesis's start with closed publishing so this should be populated with a
                // list of allowed scripts and publishing off
                assert_ne!(metadata["script_hash_allow_list"], json!([]));
                assert_eq!(metadata["module_publishing_allowed"], false);
                assert_eq!(metadata["diem_version"], 3);
                assert_eq!(metadata["dual_attestation_limit"], 1000000000);
                assert_ne!(diem_ledger_timestampusec, 0);
                assert_ne!(diem_ledger_version, 0);

                // prove the accumulator_root_hash
                let info_hex = state_proof["ledger_info_with_signatures"].as_str().unwrap();
                let info:LedgerInfoWithSignatures = bcs::from_bytes(&hex::decode(&info_hex).unwrap()).unwrap();
                let expected_hash = info.deref().ledger_info().transaction_accumulator_hash().to_hex();
                assert_eq!(expected_hash, metadata["accumulator_root_hash"].as_str().unwrap());
            },
        },
        Test {
            name: "get metadata with older version parameter should not return version information",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_metadata", json!([1]));
                let metadata = resp.result.unwrap();
                // no data provided for the following fields when requesting older version
                assert_eq!(metadata["script_hash_allow_list"], json!(null));
                assert_eq!(metadata["module_publishing_allowed"], json!(null));
                assert_eq!(metadata["diem_version"], json!(null));
            },
        },
        Test {
            name: "account not found",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_account", json!(["d738a0b9851305dfe1d17707f0841dbc"]));
                assert!(resp.result.is_none());
            },
        },
        Test {
            name: "unknown role type account",
            run: |env: &mut testing::Env| {
                let address = format!("{:x}", diem_types::account_config::diem_root_address());
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
                        "sequence_number": 1,
                        "version": resp.diem_ledger_version,
                    }),
                );
            },
        },
        Test {
            name: "designated_dealer role type account no preburns",
            run: |env: &mut testing::Env| {
                let address = format!(
                    "{:x}",
                    diem_types::account_config::testnet_dd_account_address()
                );
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
                                "amount": 0_u64,
                                "currency": "XDX"
                            },
                            {
                                "amount": 9223370036854775807_u64,
                                "currency": "XUS"
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
                            "expiration_time": 18446744073709551615_u64,
                            "human_name": "moneybags",
                            "preburn_balances": [
                                {
                                    "amount": 0_u64,
                                    "currency": "XUS"
                                }
                            ],
                            "preburn_queues": [
                                {
                                    "preburns": [],
                                    "currency": "XUS",
                                }
                            ],
                            "received_mint_events_key": "0000000000000000000000000000000000000000000000dd",
                            "compliance_key_rotation_events_key": "0100000000000000000000000000000000000000000000dd",
                            "base_url_rotation_events_key": "0200000000000000000000000000000000000000000000dd",
                        },
                        "sent_events_key": "0400000000000000000000000000000000000000000000dd",
                        "sequence_number": 2,
                        "version": resp.diem_ledger_version,
                    }),
                );
            },
        },
        Test {
            name: "designated_dealer role type account with preburns",
            run: |env: &mut testing::Env| {
                let address =
                    diem_types::account_config::testnet_dd_account_address().to_hex();
                env.submit_and_wait(
                    env.create_txn(&env.dd, stdlib::encode_preburn_script(xus_tag(), 100))
                );
                env.submit_and_wait(
                    env.create_txn(&env.dd, stdlib::encode_preburn_script(xus_tag(), 50))
                );
                env.submit_and_wait(
                    env.create_txn(&env.dd, stdlib::encode_preburn_script(xus_tag(), 60))
                );
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
                                "amount": 0_u64,
                                "currency": "XDX"
                            },
                            {
                                "amount": 9223370036854775597_u64,
                                "currency": "XUS"
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
                            "expiration_time": 18446744073709551615_u64,
                            "human_name": "moneybags",
                            "preburn_balances": [
                                {
                                    "amount": 210_u64,
                                    "currency": "XUS"
                                }
                            ],
                            "preburn_queues": [
                                {
                                    "currency": "XUS",
                                    "preburns": [
                                        {
                                            "preburn": {
                                                "amount": 100_u64,
                                                "currency": "XUS",
                                            },
                                            "metadata": "",
                                        },
                                        {
                                            "preburn": {
                                                "amount": 50_u64,
                                                "currency": "XUS"
                                            },
                                            "metadata": "",
                                        },
                                        {
                                            "preburn": {
                                                "amount": 60_u64,
                                                "currency": "XUS"
                                            },
                                            "metadata": "",
                                        },
                                    ],
                                }
                            ],
                            "received_mint_events_key": "0000000000000000000000000000000000000000000000dd",
                            "compliance_key_rotation_events_key": "0100000000000000000000000000000000000000000000dd",
                            "base_url_rotation_events_key": "0200000000000000000000000000000000000000000000dd",
                        },
                        "sent_events_key": "0400000000000000000000000000000000000000000000dd",
                        "sequence_number": 5,
                        "version": resp.diem_ledger_version,
                    }),
                );
            },
        },
        Test {
            name: "parent vasp role type account",
            run: |env: &mut testing::Env| {
                let account = &env.vasps[0];
                let address = format!("{:x}", &account.address);
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": account.auth_key().to_string(),
                        "balances": [{"amount": 997000000000_u64, "currency": "XUS"}],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0200000000000000{}", address),
                        "role": {
                            "base_url": "",
                            "base_url_rotation_events_key": format!("0100000000000000{}", address),
                            "compliance_key": "",
                            "compliance_key_rotation_events_key": format!("0000000000000000{}", address),
                            "diem_id_domains": [],
                            "expiration_time": 18446744073709551615_u64,
                            "human_name": "Novi 0",
                            "num_children": 1,
                            "type": "parent_vasp",
                        },
                        "sent_events_key": format!("0300000000000000{}", address),
                        "sequence_number": 1,
                        "version": resp.diem_ledger_version,
                    }),
                );
            },
        },
        Test {
            name: "get account by version",
            run: |env: &mut testing::Env| {
                let account = &env.vasps[0];
                let address = format!("{:x}", &account.address);
                // The account txn is creating child VASP.
                let account_sequence = 0;

                let resp = env.send("get_account_transaction", json!([address, account_sequence, false]));
                let result = resp.result.unwrap();
                let prev_version: u64 = result["version"].as_u64().unwrap() - 1;
                let resp = env.send("get_account", json!([address, prev_version]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": account.auth_key().to_string(),
                        "balances": [{"amount": 1000000000000_u64, "currency": "XUS"}],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0200000000000000{}", address),
                        "role": {
                            "base_url": "",
                            "base_url_rotation_events_key": format!("0100000000000000{}", address),
                            "compliance_key": "",
                            "compliance_key_rotation_events_key": format!("0000000000000000{}", address),
                            "diem_id_domains": [],
                            "expiration_time": 18446744073709551615_u64,
                            "human_name": "Novi 0",
                            "num_children": 0,
                            "type": "parent_vasp"
                        },
                        "sent_events_key": format!("0300000000000000{}", address),
                        "sequence_number": 0,
                        "version": prev_version,
                    }),
                );
            },
        },
        Test {
            name: "child vasp role type account",
            run: |env: &mut testing::Env| {
                let parent = &env.vasps[0];
                let account = &env.vasps[0].children[0];
                let address = format!("{:x}", &account.address);
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": account.auth_key().to_string(),
                        "balances": [{"amount": 3000000000_u64, "currency": "XUS"}],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0000000000000000{}", address),
                        "role": {
                            "type": "child_vasp",
                            "parent_vasp_address": format!("{:x}", &parent.address),
                        },
                        "sent_events_key": format!("0100000000000000{}", address),
                        "sequence_number": 0,
                        "version": resp.diem_ledger_version,
                    }),
                );
            },
        },
        Test {
            name: "peer to peer account transaction with events",
            run: |env: &mut testing::Env| {
                let prev_ledger_version = env.send("get_metadata", json!([])).diem_ledger_version;

                let txn = env.transfer_coins((0, 0), (1, 0), 200000);
                env.wait_for_txn(&txn);
                let txn_hex = hex::encode(bcs::to_bytes(&txn).expect("bcs txn failed"));

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
                    version > prev_ledger_version && version <= resp.diem_ledger_version
                );

                let gas_used = result["gas_used"].as_u64().expect("exist as u64");
                let txn_hash = Transaction::UserTransaction(txn.clone()).hash().to_hex();

                let script = match txn.payload() {
                    TransactionPayload::Script(s) => s,
                    _ => unreachable!(),
                };
                let script_hash = diem_crypto::HashValue::sha3_256_of(script.code()).to_hex();
                let script_bytes = hex::encode(bcs::to_bytes(script).unwrap());

                assert_eq!(
                    result,
                    json!({
                        "bytes": format!("00{}", txn_hex),
                        "events": [
                            {
                                "data": {
                                    "amount": {"amount": 200000_u64, "currency": "XUS"},
                                    "metadata": "",
                                    "receiver": format!("{:x}", &receiver.address),
                                    "sender": format!("{:x}", &sender.address),
                                    "type": "sentpayment"
                                },
                                "key": format!("0100000000000000{:x}", &sender.address),
                                "sequence_number": 0,
                                "transaction_version": version,
                            },
                            {
                                "data": {
                                    "amount": {"amount": 200000_u64, "currency": "XUS"},
                                    "metadata": "",
                                    "receiver": format!("{:x}", &receiver.address),
                                    "sender": format!("{:x}", &sender.address),
                                    "type": "receivedpayment"
                                },
                                "key": format!("0000000000000000{:x}", &receiver.address),
                                "sequence_number": 1,
                                "transaction_version": version
                            }
                        ],
                        "gas_used": gas_used,
                        "hash": txn_hash,
                        "transaction": {
                            "chain_id": 4,
                            "expiration_timestamp_secs": txn.expiration_timestamp_secs(),
                            "gas_currency": "XUS",
                            "gas_unit_price": 0,
                            "max_gas_amount": 1000000,
                            "public_key": sender.public_key.to_string(),
                            "secondary_signers": [],
                            "secondary_signature_schemes": [],
                            "secondary_signatures": [],
                            "secondary_public_keys": [],
                            "script": {
                                "type": "peer_to_peer_with_metadata",
                                "type_arguments": [
                                    "XUS"
                                ],
                                "arguments": [
                                    format!("{{ADDRESS: {:?}}}", &receiver.address),
                                    "{U64: 200000}",
                                    "{U8Vector: 0x}",
                                    "{U8Vector: 0x}"
                                ],
                                "code": hex::encode(script.code()),
                                "amount": 200000,
                                "currency": "XUS",
                                "metadata": "",
                                "metadata_signature": "",
                                "receiver": format!("{:x}", &receiver.address),
                            },
                            "script_bytes": script_bytes,
                            "script_hash": script_hash,
                            "sender": format!("{:x}", &sender.address),
                            "sequence_number": 0,
                            "signature": hex::encode(txn.authenticator().sender().signature_bytes()),
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
                        "location": "00000000000000000000000000000001::DiemAccount",
                        "type": "move_abort"
                    })
                );
            },
        },
        Test {
            name: "re-submit transaction won't fail",
            run: |env: &mut testing::Env| {
                let txn1 = env.transfer_coins_txn((0, 0), (1, 0), 200);
                env.submit(&txn1);
                env.submit(&txn1);
                env.wait_for_txn(&txn1);
            },
        },
        Test {
            name: "invalid transaction submitted: mempool validation error",
            run: |env: &mut testing::Env| {
                env.allow_execution_failures(|env: &mut testing::Env| {
                    let txn1 = env.transfer_coins_txn((0, 0), (1, 0), 200);
                    let txn2 = env.transfer_coins_txn((0, 0), (1, 0), 300);
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
            name: "expired transaction submitted: An expired transaction with too new sequence number will still be rejected",
            run: |env: &mut testing::Env| {
                env.allow_execution_failures(|env: &mut testing::Env| {
                    let txn1 = {
                        let account1 = env.get_account(0, 0);
                        let account2 = env.get_account(1, 0);
                        let script = stdlib::encode_peer_to_peer_with_metadata_script(
                            xus_tag(),
                            account2.address,
                            100,
                            vec![],
                            vec![],
                        );
                        let seq = env
                            .get_account_sequence(account1.address.to_string())
                            .expect("account should exist onchain for create transaction");
                        diem_types::transaction::helpers::create_user_txn(
                            account1,
                            TransactionPayload::Script(script),
                            account1.address,
                            seq + 100,
                            1_000_000,
                            0,
                            diem_types::account_config::XUS_NAME.to_owned(),
                            -100_000_000,
                            diem_types::chain_id::ChainId::test(),
                        ).expect("user signed transaction")
                    };
                    let resp = env.submit(&txn1);
                    assert_eq!(
                        resp.error.expect("error").message,
                        "Server error: VM Validation error: TRANSACTION_EXPIRED".to_string(),
                    );
                });
            },
        },
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
            name: "preburn & burn events",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_preburn_script(xus_tag(), 100);
                let txn = env.create_txn(&env.dd, script.clone());
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();

                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data": {
                                "amount": {"amount": 100, "currency": "XUS"},
                                "metadata": "",
                                "receiver": "000000000000000000000000000000dd",
                                "sender": "000000000000000000000000000000dd",
                                "type": "sentpayment"
                            },
                            "key": "0400000000000000000000000000000000000000000000dd",
                            "sequence_number": 5,
                            "transaction_version": version
                        },
                        {
                            "data": {
                                "amount": {"amount": 100, "currency": "XUS"},
                                "preburn_address": "000000000000000000000000000000dd",
                                "type": "preburn"
                            },
                            "key": "07000000000000000000000000000000000000000a550c18",
                            "sequence_number": 3,
                            "transaction_version": version
                        }
                    ]),
                    "{}",
                    result["events"]
                );
                assert_eq!(
                    result["transaction"]["script"],
                    json!({
                        "type_arguments": [
                            "XUS"
                        ],
                        "arguments": [
                            "{U64: 100}",
                        ],
                        "code": hex::encode(script.code()),
                        "type": "preburn"
                    }),
                    "{}",
                    result["transaction"]
                );

                let script = stdlib::encode_burn_with_amount_script_function(
                    xus_tag(), 0, env.dd.address, 100
                );
                let burn_txn = env.create_txn_by_payload(
                    &env.tc,
                    script,
                );
                let result = env.submit_and_wait(burn_txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([{
                        "data":{
                            "amount":{"amount":100,"currency":"XUS"},
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
                assert_eq!(
                    result["transaction"]["script"],
                    json!({
                        "type_arguments": [
                            "XUS"
                        ],
                        "arguments_bcs": [
                            "0000000000000000",
                            "000000000000000000000000000000dd",
                            "6400000000000000",
                        ],
                        "type": "script_function",
                        "module_address":"00000000000000000000000000000001",
                        "module_name":"TreasuryComplianceScripts",
                        "function_name":"burn_with_amount",
                    }),
                    "{}",
                    result["transaction"]
                );
            },
        },
        Test {
            name: "cancel burn event",
            run: |env: &mut testing::Env| {
                let txn =
                    env.create_txn(&env.dd, stdlib::encode_preburn_script(xus_tag(), 100));
                env.submit_and_wait(txn);

                let script = stdlib::encode_cancel_burn_with_amount_script_function(xus_tag(), env.dd.address, 100);
                let cancel_burn_txn = env.create_txn_by_payload(
                    &env.tc,
                    script,
                );
                let result = env.submit_and_wait(cancel_burn_txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "amount":{"amount":100,"currency":"XUS"},
                                "preburn_address":"000000000000000000000000000000dd",
                                "type":"cancelburn"
                            },
                            "key":"08000000000000000000000000000000000000000a550c18",
                            "sequence_number":0,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "amount":{"amount":100,"currency":"XUS"},
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
                assert_eq!(
                    result["transaction"]["script"],
                    json!({
                        "type_arguments": [
                            "XUS"
                        ],
                        "arguments_bcs": [
                            "000000000000000000000000000000dd",
                            "6400000000000000",
                        ],
                        "type": "script_function",
                        "module_address":"00000000000000000000000000000001",
                        "module_name":"TreasuryComplianceScripts",
                        "function_name":"cancel_burn_with_amount",
                    }),
                    "{}",
                    result["transaction"]
                );
            },
        },
        Test {
            name: "update exchange rate event",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_update_exchange_rate_script(xus_tag(), 0, 1, 4);
                let txn = env.create_txn(&env.tc, script.clone());
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([{
                        "data":{
                            "currency_code":"XUS",
                            "new_to_xdx_exchange_rate":0.25,
                            "type":"to_xdx_exchange_rate_update"
                        },
                        "key":"09000000000000000000000000000000000000000a550c18",
                        "sequence_number":0,
                        "transaction_version":version
                    }]),
                    "{}",
                    result["events"]
                );
                assert_eq!(
                    result["transaction"]["script"],
                    json!({
                        "type_arguments": [
                            "XUS"
                        ],
                        "arguments": [
                            "{U64: 0}",
                            "{U64: 1}",
                            "{U64: 4}"
                        ],
                        "code": hex::encode(script.code()),
                        "type": "update_exchange_rate"
                    }),
                    "{}",
                    result["transaction"]
                );
            },
        },
        Test {
            name: "mint & received mint events",
            run: |env: &mut testing::Env| {
                let script = stdlib::encode_tiered_mint_script(
                    xus_tag(),
                    0,
                    env.dd.address,
                    1_000_000,
                    1,
                );
                let txn = env.create_txn(&env.tc, script.clone());
                let result = env.submit_and_wait(txn);
                let version = result["version"].as_u64().unwrap();
                assert_eq!(
                    result["events"],
                    json!([
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"XUS"},
                                "destination_address":"000000000000000000000000000000dd",
                                "type":"receivedmint"
                            },
                            "key":"0000000000000000000000000000000000000000000000dd",
                            "sequence_number":1,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"XUS"},
                                "type":"mint"
                            },
                            "key":"05000000000000000000000000000000000000000a550c18",
                            "sequence_number":1,
                            "transaction_version":version},
                        {
                            "data":{
                                "amount":{"amount":1000000,"currency":"XUS"},
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
                assert_eq!(
                    result["transaction"]["script"],
                    json!({
                        "type_arguments": [
                            "XUS"
                        ],
                        "arguments": [
                            "{U64: 0}",
                            "{ADDRESS: 000000000000000000000000000000DD}",
                            "{U64: 1000000}",
                            "{U64: 1}",
                        ],
                        "code": hex::encode(script.code()),
                        "type": "tiered_mint"
                    }),
                    "{}",
                    result["transaction"]
                );
            },
        },
        Test {
            name: "rotate compliance key rotation events",
            run: |env: &mut testing::Env| {
                let private_key = generate_key::generate_key();
                let public_key: diem_crypto::ed25519::Ed25519PublicKey = (&private_key).into();
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
                            "key": format!("0100000000000000{:x}", &env.vasps[0].address),
                            "sequence_number":0,
                            "transaction_version":version
                        },
                        {
                            "data":{
                                "new_compliance_public_key": hex::encode(public_key.to_bytes()),
                                "time_rotated_seconds": rotated_seconds,
                                "type":"compliancekeyrotation"
                            },
                            "key": format!("0000000000000000{:x}", &env.vasps[0].address),
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
            name: "create account event",
            run: |env: &mut testing::Env| {
                let response = env.send(
                    "get_events",
                    json!(["00000000000000000000000000000000000000000a550c18", 0, 3]),
                );
                let events = response.result.unwrap();
                assert_eq!(
                    events,
                    json!([
                        {
                            "data":{
                                "created_address":"0000000000000000000000000a550c18",
                                "role_id":0,
                                "type":"createaccount"
                            },
                            "key":"00000000000000000000000000000000000000000a550c18",
                            "sequence_number":0,
                            "transaction_version":0
                        },
                        {
                            "data":{
                                "created_address":"0000000000000000000000000b1e55ed",
                                "role_id":1,
                                "type":"createaccount"
                            },
                            "key":"00000000000000000000000000000000000000000a550c18",
                            "sequence_number":1,
                            "transaction_version":0
                        },
                        {
                            "data":{
                                "created_address":"b5b333aabbf92e78524e2129b722eaca",
                                "role_id":3,
                                "type":"createaccount"
                            },
                            "key":"00000000000000000000000000000000000000000a550c18",
                            "sequence_number":2,
                            "transaction_version":0
                        }
                    ]),
                    "{}",
                    events
                )
            },
        },
        Test {
            name: "get_transactions without event",
            run: |env: &mut testing::Env| {
                let response = env.send("get_transactions", json!([0, 1000, false]));
                let txns = response.result.unwrap();
                assert!(!txns.as_array().unwrap().is_empty());

                for (index, txn) in txns.as_array().unwrap().iter().enumerate() {
                    assert_eq!(txn["version"], index);
                    assert_eq!(txn["events"], json!([]));
                }
            },
        },
        Test {
            name: "get_account_transactions without event",
            run: |env: &mut testing::Env| {
                let sender = &env.vasps[0].children[0];
                let response = env.send(
                    "get_account_transactions",
                    json!([sender.address.to_string(), 0, 1000, false]),
                );
                let txns = response.result.unwrap();
                assert!(!txns.as_array().unwrap().is_empty());

                for txn in txns.as_array().unwrap() {
                    assert_eq!(txn["events"], json!([]));
                }
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
            name: "get_transactions_with_proofs",
            run: |env: &mut testing::Env| {
                let resp = env.send("get_metadata", json!([]));

                let limit = 10;
                let include_events = false;
                assert!(resp.diem_ledger_version > limit);
                // We test 2 cases:
                //      1. base_version + limit > resp.diem_ledger_version
                //      2. base_version + limit < resp.diem_ledger_version
                for base_version in &[resp.diem_ledger_version, 0] {
                   // We use a batched call to ensure we get an answer using the same latest server ledger_info for both
                    let responses = env.send_request(json!([
                        {"jsonrpc": "2.0", "method": "get_state_proof", "params": json!([0]), "id": 1},
                        {"jsonrpc": "2.0", "method": "get_transactions_with_proofs", "params": json!([*base_version, limit, include_events]), "id": 2}
                    ]));

                    let f:Vec<serde_json::Value> = serde_json::from_value(responses).expect("should be valid serde_json::Value");
                    let data = &f.iter().find(|g| g["id"] == 2).unwrap()["result"];
                    let proofs = data["proofs"].as_object().unwrap();

                    // We want to verify the signatures of the LedgerInfo that will be returned by the
                    // get_transactions_with_proofs call to be sure it's valid, but
                    // since we don't have a local state with the set of validators unlike an actual client,
                    // we need to get the validator set from the batched get_state_proof call.
                    let ledger_info_view = &f.iter().find(|g| g["id"] == 1).unwrap()["result"];
                    let ep_cp = ledger_info_view["epoch_change_proof"].as_str().unwrap();
                    let epoch_proofs:EpochChangeProof = bcs::from_bytes(&hex::decode(&ep_cp).unwrap()).unwrap();
                    let some_li:Vec<_> = epoch_proofs.ledger_info_with_sigs;
                    assert!(!some_li.is_empty());
                    // We use the last one (but the validator set does not change in the tests and
                    // in practice the epoch change proofs should be verified).
                    let validator_set = &some_li.last().unwrap().ledger_info().next_epoch_state().unwrap().verifier;

                    // The actual proofs
                    let raw_hex_li = proofs["ledger_info_to_transaction_infos_proof"].as_str().unwrap();
                    let li_to_tip:TransactionAccumulatorRangeProof = bcs::from_bytes(&hex::decode(&raw_hex_li).unwrap()).unwrap();
                    // The txs for which we got the proofs
                    let raw_hex_txs = proofs["transaction_infos"].as_str().unwrap();
                    let txs_infos:Vec<TransactionInfo> = bcs::from_bytes(&hex::decode(&raw_hex_txs).unwrap()).unwrap();
                    let hashes: Vec<_> = txs_infos
                    .iter()
                    .map(CryptoHash::hash)
                    .collect();

                    // We make sure we have between 1 and 10 txs
                    if hashes.len() > 10 || hashes.is_empty() {
                        panic!("Unexpected hash len returned at {} by get_transactions_with_proofs: {}", base_version, hashes.len());
                    }

                    // We must check the transactions we got correspond to the hashes in the proofs
                    let raw_blobs = data["serialized_transactions"].as_array().unwrap();
                    assert!(!raw_blobs.is_empty());
                    let actual_txs:Vec<Transaction>= raw_blobs.iter().map(|tx| {
                        bcs::from_bytes(&hex::decode(&tx.as_str().unwrap()).unwrap()).unwrap()
                    }).collect();
                    assert!(!actual_txs.is_empty());
                    assert_eq!(txs_infos.len(), actual_txs.len());
                    for (index, tx) in actual_txs.iter().enumerate() {
                        // Notice we need to actually hash the transaction to be sure its hash is correct
                        assert_eq!(tx.hash(), txs_infos[index].transaction_hash());
                    }

                    // We compare our results with the non-veryfing API for the test
                    let resp_tx = env.send("get_transactions", json!([*base_version, txs_infos.len(), false]));
                    let no_proof_txns = resp_tx.result.unwrap();
                    assert!(!no_proof_txns.as_array().unwrap().is_empty());
                    assert_eq!(no_proof_txns.as_array().unwrap().len(), actual_txs.len());
                    for (index, tx) in no_proof_txns.as_array().unwrap().iter().enumerate() {
                        assert_eq!(tx["hash"].as_str().unwrap(), actual_txs[index].hash().to_hex());
                    }

                    // We need to get the details required to verify the proof from the batched get_state_proof call
                    let li_raw = ledger_info_view["ledger_info_with_signatures"].as_str().unwrap();
                    let li:LedgerInfoWithSignatures = bcs::from_bytes(&hex::decode(&li_raw).unwrap()).unwrap();
                    let expected_hash = li.ledger_info().transaction_accumulator_hash();

                    // and we verify the signature of the provided ledger info that provided the accumulator hash
                    assert!(li.verify_signatures(&validator_set).is_ok());

                    // and we eventually verify the proofs for the transactions
                    assert!(li_to_tip.verify(expected_hash, Some(*base_version), &hashes).is_ok());
                }
            },
        },
        Test {
            name: "add and remove diem id domain to parent vasp account",
            run: |env: &mut testing::Env| {
                // add domain
                let domain = DiemIdVaspDomainIdentifier::new(&"diem").unwrap().as_str().as_bytes().to_vec();
                let txn = env.create_txn_by_payload(
                    &env.tc,
                    stdlib::encode_add_diem_id_domain_script_function(
                        env.vasps[0].address,
                        domain,
                    ),
                );
                let result = env.submit_and_wait(txn);
                let version1 = result["version"].as_u64().unwrap();

                // get account
                let account = &env.vasps[0];
                let address = format!("{:x}", &account.address);
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result["role"]["diem_id_domains"],
                    json!(
                        ["diem"]
                    ),
                );

                // remove domain
                let domain = DiemIdVaspDomainIdentifier::new(&"diem").unwrap().as_str().as_bytes().to_vec();
                let txn = env.create_txn_by_payload(
                    &env.tc,
                    stdlib::encode_remove_diem_id_domain_script_function(
                        env.vasps[0].address,
                        domain,
                    ),
                );
                let result = env.submit_and_wait(txn);
                let version2 = result["version"].as_u64().unwrap();

                // get account
                let account = &env.vasps[0];
                let address = format!("{:x}", &account.address);
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result["role"]["diem_id_domains"],
                    json!(
                        []
                    ),
                );

                // get event
                let tc_address = format!("{:x}", &env.tc.address);
                let resp = env.send("get_account", json!([tc_address]));
                let result = resp.result.unwrap();
                let diem_id_domain_events_key = result["role"]["diem_id_domain_events_key"].clone();
                let response = env.send(
                    "get_events",
                    json!([diem_id_domain_events_key, 0, 3]),
                );
                let events = response.result.unwrap();
                assert_eq!(
                    events,
                    json!([
                        {
                            "data":{
                                "domain": "diem",
                                "removed": false,
                                "address": address,
                                "type":"diemiddomain"
                            },
                            "key": format!("0000000000000000{}", tc_address),
                            "sequence_number": 0,
                            "transaction_version": version1,
                        },
                        {
                            "data":{
                                "domain": "diem",
                                "removed": true,
                                "address": address,
                                "type":"diemiddomain"
                            },
                            "key": format!("0000000000000000{}", tc_address),
                            "sequence_number": 1,
                            "transaction_version": version2,
                        },
                    ]),
                );
            },
        },
        Test {
            name: "get tc account",
            run: |env: &mut testing::Env| {
                let address = format!("{:x}", &env.tc.address);
                let resp = env.send("get_account", json!([address]));
                let result = resp.result.unwrap();
                assert_eq!(
                    result,
                    json!({
                        "address": address,
                        "authentication_key": &env.tc.auth_key().to_string(),
                        "balances": [],
                        "delegated_key_rotation_capability": false,
                        "delegated_withdrawal_capability": false,
                        "is_frozen": false,
                        "received_events_key": format!("0100000000000000{}", address),
                        "role": {
                            "diem_id_domain_events_key": format!("0000000000000000{}", address),
                            "type": "treasury_compliance",
                        },
                        "sent_events_key": format!("0200000000000000{}", address),
                        "sequence_number": 8,
                        "version": resp.diem_ledger_version,
                    }),
                );
            }
        },
        Test {
            name: "get_events_with_proofs",
            run: |env: &mut testing::Env| {
                let responses = env.send_request(json!([
                    {"jsonrpc": "2.0", "method": "get_state_proof", "params": json!([0]), "id": 1},
                    {"jsonrpc": "2.0", "method": "get_events_with_proofs", "params": json!(["00000000000000000000000000000000000000000a550c18", 0, 3]), "id": 2}
                ]));

                let resps:Vec<serde_json::Value> = serde_json::from_value(responses).expect("should be valid serde_json::Value");

                // we need te get the current ledger_info in order to verify the events
                let ledger_info_view = &resps.iter().find(|g| g["id"] == 1).unwrap()["result"];
                let li_raw = ledger_info_view["ledger_info_with_signatures"].as_str().unwrap();
                let li:LedgerInfoWithSignatures = bcs::from_bytes(&hex::decode(&li_raw).unwrap()).unwrap();
                // We want to verify the signatures of the LedgerInfo to be sure it's valid, but
                // since we don't have a local state with the set of validators unlike an actual client,
                // we need to get the validator set from the batched get_state_proof call.
                let ep_cp = ledger_info_view["epoch_change_proof"].as_str().unwrap();
                let epoch_proofs:EpochChangeProof = bcs::from_bytes(&hex::decode(&ep_cp).unwrap()).unwrap();
                let some_li:Vec<_> = epoch_proofs.ledger_info_with_sigs;
                assert!(!some_li.is_empty());
                // We use the last one (but the validator set does not change in the tests and
                // in practice the epoch change proofs should be verified).
                let validator_set = &some_li.last().unwrap().ledger_info().next_epoch_state().unwrap().verifier;
                // And we verify the signature
                assert!(li.verify_signatures(&validator_set).is_ok());

                // We now need to verify the events using this verified ledger_info:
                let ledger_info = li.ledger_info();
                let data = &resps.iter().find(|g| g["id"] == 2).unwrap()["result"].as_array().unwrap();
                let mut events:Vec<EventView> = vec![];
                for d in  data.iter() {
                    let bcs_data = d["event_with_proof"].as_str().unwrap();
                    let event:EventWithProof = bcs::from_bytes(&hex::decode(&bcs_data).unwrap()).unwrap();
                    let hash = event.event.hash();

                    // We verify the proof of the event
                    assert!(event.proof.verify(ledger_info, hash, event.transaction_version, event.event_index).is_ok());

                    // We can now use our verified events
                    events.push((event.transaction_version, event.event).try_into().unwrap());
                }

                assert_eq!(events.len(),3);
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
