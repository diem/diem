// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use forge::{
    forge_main, ForgeConfig, LocalFactory, Options, PublicUsageContext, PublicUsageTest, Result,
    Test,
};
use serde_json::json;

#[allow(dead_code)]
mod helper;
use helper::JsonRpcTestHelper;

fn main() -> Result<()> {
    let tests = ForgeConfig {
        public_usage_tests: &[&CurrencyInfo],
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
