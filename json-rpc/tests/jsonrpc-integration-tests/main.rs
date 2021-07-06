// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::ledger_info::LedgerInfoWithSignatures;
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
        public_usage_tests: &[&CurrencyInfo, &BlockMetadata],
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
