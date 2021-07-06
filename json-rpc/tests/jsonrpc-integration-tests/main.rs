// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::hash::CryptoHash;
use diem_sdk::transaction_builder::Currency;
use diem_transaction_builder::stdlib;
use diem_types::{
    account_config::xus_tag,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    transaction::{Transaction, TransactionPayload},
};
use forge::{
    forge_main, ForgeConfig, LocalFactory, Options, PublicUsageContext, PublicUsageTest, Result,
    Test,
};
use serde_json::json;
use std::ops::Deref;

#[allow(dead_code)]
mod helper;
use helper::JsonRpcTestHelper;

fn main() -> Result<()> {
    let tests = ForgeConfig {
        public_usage_tests: &[
            &CurrencyInfo,
            &BlockMetadata,
            &OldMetadata,
            &AccoutNotFound,
            &UnknownAccountRoleType,
            &DesignatedDealerPreburns,
            &ParentVaspAccountRole,
            &GetAccountByVersion,
            &ChildVaspAccountRole,
            &PeerToPeerWithEvents,
            &PeerToPeerErrorExplination,
            &ReSubmittingTransactionWontFail,
            &MempoolValidationError,
            &ExpiredTransaction,
        ],
        admin_tests: &[],
        network_tests: &[],
    };

    let options = Options::from_args();
    forge_main(tests, LocalFactory::from_workspace()?, &options)
}

struct CurrencyInfo;

impl Test for CurrencyInfo {
    fn name(&self) -> &'static str {
        "jsonrpc::currency-info"
    }
}

impl PublicUsageTest for CurrencyInfo {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
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
        );

        Ok(())
    }
}

struct BlockMetadata;

impl Test for BlockMetadata {
    fn name(&self) -> &'static str {
        "jsonrpc::block-metadata"
    }
}

impl PublicUsageTest for BlockMetadata {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());

        // Fund and account to ensure there are some txns on the chain
        let account = ctx.random_account();
        ctx.create_parent_vasp_account(account.authentication_key())?;
        ctx.fund(account.address(), 10)?;

        // batch request
        let resp = env.send_request(json!([
            {"jsonrpc": "2.0", "method": "get_metadata", "params": [], "id": 1},
            {"jsonrpc": "2.0", "method": "get_state_proof", "params": [0], "id": 2}
        ]));

        // extract both responses
        let resps: Vec<serde_json::Value> =
            serde_json::from_value(resp).expect("should be valid serde_json::Value");
        let metadata = &resps.iter().find(|g| g["id"] == 1).unwrap()["result"];
        let state_proof = &resps.iter().find(|g| g["id"] == 2).unwrap()["result"];

        // extract header and ensure they match in both responses
        let diem_chain_id = &resps[0]["diem_chain_id"];
        let diem_ledger_timestampusec = &resps[0]["diem_ledger_timestampusec"];
        let diem_ledger_version = &resps[0]["diem_ledger_version"];

        assert_eq!(diem_chain_id, &resps[1]["diem_chain_id"]);
        assert_eq!(
            diem_ledger_timestampusec,
            &resps[1]["diem_ledger_timestampusec"]
        );
        assert_eq!(diem_ledger_version, &resps[1]["diem_ledger_version"]);

        // parse metadata
        assert_eq!(&metadata["chain_id"], diem_chain_id);
        assert_eq!(&metadata["timestamp"], diem_ledger_timestampusec);
        assert_eq!(&metadata["version"], diem_ledger_version);
        assert_eq!(metadata["chain_id"], ctx.chain_id().id());
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
        let info: LedgerInfoWithSignatures =
            bcs::from_bytes(&hex::decode(&info_hex).unwrap()).unwrap();
        let expected_hash = info
            .deref()
            .ledger_info()
            .transaction_accumulator_hash()
            .to_hex();
        assert_eq!(
            expected_hash,
            metadata["accumulator_root_hash"].as_str().unwrap()
        );

        Ok(())
    }
}

/// Get Metadata with older version parameter should not return version information
struct OldMetadata;

impl Test for OldMetadata {
    fn name(&self) -> &'static str {
        "jsonrpc::old-metadata"
    }
}

impl PublicUsageTest for OldMetadata {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        // create a random accound in order to force the version hieght to be greater than 1
        let account = ctx.random_account();
        ctx.create_parent_vasp_account(account.authentication_key())?;

        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let resp = env.send("get_metadata", json!([1]));
        let metadata = resp.result.unwrap();
        // no data provided for the following fields when requesting older version
        assert_eq!(metadata["script_hash_allow_list"], json!(null));
        assert_eq!(metadata["module_publishing_allowed"], json!(null));
        assert_eq!(metadata["diem_version"], json!(null));
        Ok(())
    }
}

struct AccoutNotFound;

impl Test for AccoutNotFound {
    fn name(&self) -> &'static str {
        "jsonrpc::account-not-found"
    }
}

impl PublicUsageTest for AccoutNotFound {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let resp = env.send("get_account", json!(["d738a0b9851305dfe1d17707f0841dbc"]));
        assert!(resp.result.is_none());
        Ok(())
    }
}

struct UnknownAccountRoleType;

impl Test for UnknownAccountRoleType {
    fn name(&self) -> &'static str {
        "jsonrpc::unknown-account-role-type"
    }
}

impl PublicUsageTest for UnknownAccountRoleType {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let address = format!("{:x}", diem_types::account_config::diem_root_address());
        let resp = env.send("get_account", json!([address]));
        let mut result = resp.result.unwrap();
        // as we generate account auth key, ignore it in assertion
        assert_ne!(result["authentication_key"].as_str().unwrap(), "");
        result["authentication_key"] = json!(null);
        let sequence_number = result["sequence_number"].as_u64().unwrap();
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
                "sequence_number": sequence_number,
                "version": resp.diem_ledger_version,
            }),
        );
        Ok(())
    }
}

struct DesignatedDealerPreburns;

impl Test for DesignatedDealerPreburns {
    fn name(&self) -> &'static str {
        "jsonrpc::designated-dealer-preburns"
    }
}

impl PublicUsageTest for DesignatedDealerPreburns {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let factory = ctx.transaction_factory();
        let mut dd = ctx.random_account();
        ctx.create_designated_dealer_account(dd.authentication_key())?;

        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let address = format!("{:x}", dd.address());
        let resp = env.send("get_account", json!([address]));
        let result = resp.result.unwrap();
        let human_name = result["role"]["human_name"].as_str().unwrap();

        // Without Preburns
        assert_eq!(
            result,
            json!({
                "address": address,
                "authentication_key": dd.authentication_key(),
                "balances": [
                    {
                        "amount": 0_u64,
                        "currency": "XUS"
                    },
                ],
                "delegated_key_rotation_capability": false,
                "delegated_withdrawal_capability": false,
                "is_frozen": false,
                "received_events_key": EventKey::new_from_address(&dd.address(), 3),
                "role": {
                    "type": "designated_dealer",
                    "base_url": "",
                    "compliance_key": "",
                    "expiration_time": 18446744073709551615_u64,
                    "human_name": human_name,
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
                    "received_mint_events_key": EventKey::new_from_address(&dd.address(), 0),
                    "compliance_key_rotation_events_key": EventKey::new_from_address(&dd.address(), 1),
                    "base_url_rotation_events_key": EventKey::new_from_address(&dd.address(), 2),
                },
                "sent_events_key": EventKey::new_from_address(&dd.address(), 4),
                "sequence_number": dd.sequence_number(),
                "version": resp.diem_ledger_version,
            }),
        );

        // Fund the DD account and create some Pre-burns
        ctx.fund(dd.address(), 400)?;

        env.submit_and_wait(&dd.sign_with_transaction_builder(
            factory.script(stdlib::encode_preburn_script(xus_tag(), 100)),
        ));
        env.submit_and_wait(&dd.sign_with_transaction_builder(
            factory.script(stdlib::encode_preburn_script(xus_tag(), 40)),
        ));
        env.submit_and_wait(&dd.sign_with_transaction_builder(
            factory.script(stdlib::encode_preburn_script(xus_tag(), 60)),
        ));

        let resp = env.send("get_account", json!([address]));
        let result = resp.result.unwrap();

        // With Preburns
        assert_eq!(
            result,
            json!({
                "address": address,
                "authentication_key": dd.authentication_key(),
                "balances": [
                    {
                        "amount": 200_u64,
                        "currency": "XUS"
                    },
                ],
                "delegated_key_rotation_capability": false,
                "delegated_withdrawal_capability": false,
                "is_frozen": false,
                "received_events_key": EventKey::new_from_address(&dd.address(), 3),
                "role": {
                    "type": "designated_dealer",
                    "base_url": "",
                    "compliance_key": "",
                    "expiration_time": 18446744073709551615_u64,
                    "human_name": human_name,
                    "preburn_balances": [
                        {
                            "amount": 200_u64,
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
                                        "amount": 40_u64,
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
                    "received_mint_events_key": EventKey::new_from_address(&dd.address(), 0),
                    "compliance_key_rotation_events_key": EventKey::new_from_address(&dd.address(), 1),
                    "base_url_rotation_events_key": EventKey::new_from_address(&dd.address(), 2),
                },
                "sent_events_key": EventKey::new_from_address(&dd.address(), 4),
                "sequence_number": dd.sequence_number(),
                "version": resp.diem_ledger_version,
            }),
        );

        Ok(())
    }
}

struct ParentVaspAccountRole;

impl Test for ParentVaspAccountRole {
    fn name(&self) -> &'static str {
        "jsonrpc::parent-vasp-account-role"
    }
}

impl PublicUsageTest for ParentVaspAccountRole {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let vasp = ctx.random_account();
        ctx.create_parent_vasp_account(vasp.authentication_key())?;

        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let address = format!("{:x}", vasp.address());
        let resp = env.send("get_account", json!([address]));
        let result = resp.result.unwrap();
        let human_name = result["role"]["human_name"].as_str().unwrap();

        assert_eq!(
            result,
            json!({
                "address": address,
                "authentication_key": vasp.authentication_key().to_string(),
                "balances": [{"amount": 0_u64, "currency": "XUS"}],
                "delegated_key_rotation_capability": false,
                "delegated_withdrawal_capability": false,
                "is_frozen": false,
                "received_events_key": EventKey::new_from_address(&vasp.address(), 2),
                "role": {
                    "base_url": "",
                    "base_url_rotation_events_key": EventKey::new_from_address(&vasp.address(), 1),
                    "compliance_key": "",
                    "compliance_key_rotation_events_key": EventKey::new_from_address(&vasp.address(), 0),
                    "vasp_domains": [],
                    "expiration_time": 18446744073709551615_u64,
                    "human_name": human_name,
                    "num_children": 0,
                    "type": "parent_vasp",
                },
                "sent_events_key": EventKey::new_from_address(&vasp.address(), 3),
                "sequence_number": 0,
                "version": resp.diem_ledger_version,
            }),
        );

        Ok(())
    }
}

struct GetAccountByVersion;

impl Test for GetAccountByVersion {
    fn name(&self) -> &'static str {
        "jsonrpc::get-account-by-version"
    }
}

impl PublicUsageTest for GetAccountByVersion {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();

        let mut vasp = ctx.random_account();
        let child = ctx.random_account();
        ctx.create_parent_vasp_account(vasp.authentication_key())?;
        env.submit_and_wait(&vasp.sign_with_transaction_builder(
            factory.create_child_vasp_account(Currency::XUS, child.authentication_key(), false, 0),
        ));

        let address = format!("{:x}", vasp.address());
        let resp = env.send("get_account_transaction", json!([address, 0, false]));
        let result = resp.result.unwrap();
        let prev_version: u64 = result["version"].as_u64().unwrap() - 1;
        let resp = env.send("get_account", json!([address, prev_version]));
        let result = resp.result.unwrap();
        let human_name = result["role"]["human_name"].as_str().unwrap();

        assert_eq!(
            result,
            json!({
                "address": address,
                "authentication_key": vasp.authentication_key().to_string(),
                "balances": [{"amount": 0_u64, "currency": "XUS"}],
                "delegated_key_rotation_capability": false,
                "delegated_withdrawal_capability": false,
                "is_frozen": false,
                "received_events_key": EventKey::new_from_address(&vasp.address(), 2),
                "role": {
                    "base_url": "",
                    "base_url_rotation_events_key": EventKey::new_from_address(&vasp.address(), 1),
                    "compliance_key": "",
                    "compliance_key_rotation_events_key": EventKey::new_from_address(&vasp.address(), 0),
                    "vasp_domains": [],
                    "expiration_time": 18446744073709551615_u64,
                    "human_name": human_name,
                    "num_children": 0,
                    "type": "parent_vasp",
                },
                "sent_events_key": EventKey::new_from_address(&vasp.address(), 3),
                "sequence_number": 0,
                "version": prev_version,
            }),
        );

        Ok(())
    }
}

struct ChildVaspAccountRole;

impl Test for ChildVaspAccountRole {
    fn name(&self) -> &'static str {
        "jsonrpc::child-vasp-account-role"
    }
}

impl PublicUsageTest for ChildVaspAccountRole {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();

        let mut parent = ctx.random_account();
        let child = ctx.random_account();
        ctx.create_parent_vasp_account(parent.authentication_key())?;
        env.submit_and_wait(&parent.sign_with_transaction_builder(
            factory.create_child_vasp_account(Currency::XUS, child.authentication_key(), false, 0),
        ));

        let address = format!("{:x}", child.address());
        let resp = env.send("get_account", json!([address]));
        let result = resp.result.unwrap();

        assert_eq!(
            result,
            json!({
                "address": address,
                "authentication_key": child.authentication_key(),
                "balances": [{"amount": 0_u64, "currency": "XUS"}],
                "delegated_key_rotation_capability": false,
                "delegated_withdrawal_capability": false,
                "is_frozen": false,
                "received_events_key": EventKey::new_from_address(&child.address(), 0),
                "role": {
                    "type": "child_vasp",
                    "parent_vasp_address": parent.address(),
                },
                "sent_events_key": EventKey::new_from_address(&child.address(), 1),
                "sequence_number": 0,
                "version": resp.diem_ledger_version,
            }),
        );

        Ok(())
    }
}

struct PeerToPeerWithEvents;

impl Test for PeerToPeerWithEvents {
    fn name(&self) -> &'static str {
        "jsonrpc::peer-to-peer-with-events"
    }
}

impl PublicUsageTest for PeerToPeerWithEvents {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();

        let prev_ledger_version = env.send("get_metadata", json!([])).diem_ledger_version;

        let (_parent1, mut child1_1, _child1_2) =
            env.create_parent_and_child_accounts(ctx, 3_000_000_000)?;
        let (_parent2, child2_1, _child2_2) =
            env.create_parent_and_child_accounts(ctx, 3_000_000_000)?;

        let txn = child1_1.sign_with_transaction_builder(factory.peer_to_peer(
            Currency::XUS,
            child2_1.address(),
            200_000,
        ));

        env.submit_and_wait(&txn);
        let txn_hex = hex::encode(bcs::to_bytes(&txn).expect("bcs txn failed"));

        let sender = &child1_1;
        let receiver = &child2_1;

        let resp = env.send(
            "get_account_transaction",
            json!([sender.address(), 0, true]),
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
                            "receiver": format!("{:x}", receiver.address()),
                            "sender": format!("{:x}", sender.address()),
                            "type": "sentpayment"
                        },
                        "key": format!("0100000000000000{:x}", sender.address()),
                        "sequence_number": 0,
                        "transaction_version": version,
                    },
                    {
                        "data": {
                            "amount": {"amount": 200000_u64, "currency": "XUS"},
                            "metadata": "",
                            "receiver": format!("{:x}", receiver.address()),
                            "sender": format!("{:x}", sender.address()),
                            "type": "receivedpayment"
                        },
                        "key": format!("0000000000000000{:x}", receiver.address()),
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
                    "public_key": sender.public_key().to_string(),
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
                            format!("{{ADDRESS: {:?}}}", receiver.address()),
                            "{U64: 200000}",
                            "{U8Vector: 0x}",
                            "{U8Vector: 0x}"
                        ],
                        "code": hex::encode(script.code()),
                        "amount": 200000,
                        "currency": "XUS",
                        "metadata": "",
                        "metadata_signature": "",
                        "receiver": format!("{:x}", receiver.address()),
                    },
                    "script_bytes": script_bytes,
                    "script_hash": script_hash,
                    "sender": format!("{:x}", sender.address()),
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

        Ok(())
    }
}

struct PeerToPeerErrorExplination;

impl Test for PeerToPeerErrorExplination {
    fn name(&self) -> &'static str {
        "jsonrpc::peer-to-peer-error-explination"
    }
}

impl PublicUsageTest for PeerToPeerErrorExplination {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let mut env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();
        let (_parent, mut child1, child2) =
            env.create_parent_and_child_accounts(ctx, 1_000_000_000)?;
        let txn = child1.sign_with_transaction_builder(factory.peer_to_peer(
            Currency::XUS,
            child2.address(),
            200000000000000,
        ));

        env.allow_execution_failures(|env| {
            env.submit_and_wait(&txn);
        });

        let sender = &child1;

        let resp = env.send(
            "get_account_transaction",
            json!([sender.address(), 0, true]),
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

        Ok(())
    }
}

struct ReSubmittingTransactionWontFail;

impl Test for ReSubmittingTransactionWontFail {
    fn name(&self) -> &'static str {
        "jsonrpc::re-submitting-transaction-wont-fail"
    }
}

impl PublicUsageTest for ReSubmittingTransactionWontFail {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();
        let (_parent, mut child1, child2) =
            env.create_parent_and_child_accounts(ctx, 1_000_000_000)?;
        let txn = child1.sign_with_transaction_builder(factory.peer_to_peer(
            Currency::XUS,
            child2.address(),
            200,
        ));

        env.submit(&txn);
        env.submit(&txn);
        env.wait_for_txn(&txn);

        Ok(())
    }
}

struct MempoolValidationError;

impl Test for MempoolValidationError {
    fn name(&self) -> &'static str {
        "jsonrpc::mempool-validator-error"
    }
}

impl PublicUsageTest for MempoolValidationError {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let mut env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();
        let (_parent, child1, child2) = env.create_parent_and_child_accounts(ctx, 1_000_000_000)?;
        let txn1 = child1.sign_transaction(
            factory
                .peer_to_peer(Currency::XUS, child2.address(), 200)
                .sender(child1.address())
                .sequence_number(child1.sequence_number())
                .build(),
        );
        let txn2 = child1.sign_transaction(
            factory
                .peer_to_peer(Currency::XUS, child2.address(), 300)
                .sender(child1.address())
                .sequence_number(child1.sequence_number())
                .build(),
        );

        env.submit(&txn1);
        env.allow_execution_failures(|env| {
            let resp = env.submit(&txn2);
            assert_eq!(
                resp.error.expect("error").message,
                "Server error: Mempool submission error: \"Failed to update gas price to 0\""
                    .to_string(),
            );
        });
        env.wait_for_txn(&txn1);

        Ok(())
    }
}

struct ExpiredTransaction;

impl Test for ExpiredTransaction {
    fn name(&self) -> &'static str {
        "jsonrpc::expired-transaction"
    }
}

impl PublicUsageTest for ExpiredTransaction {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let mut env = JsonRpcTestHelper::new(ctx.url().to_owned());
        let factory = ctx.transaction_factory();
        let (_parent, child1, child2) = env.create_parent_and_child_accounts(ctx, 1_000_000_000)?;

        env.allow_execution_failures(|env| {
            let txn = child1.sign_transaction(
                factory
                    .peer_to_peer(Currency::XUS, child2.address(), 200)
                    .sender(child1.address())
                    .sequence_number(child1.sequence_number() + 100)
                    .expiration_timestamp_secs(0)
                    .build(),
            );
            let resp = env.submit(&txn);
            assert_eq!(
                resp.error.expect("error").message,
                "Server error: VM Validation error: TRANSACTION_EXPIRED".to_string(),
            );
        });
        Ok(())
    }
}
