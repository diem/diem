// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf, process::Command};

use diem_sdk::transaction_builder::Currency;
use forge::{forge_main, ForgeConfig, Result, *};

fn main() -> Result<()> {
    let tests = ForgeConfig {
        public_usage_tests: &[&FundAccount],
        admin_tests: &[&GetMetadata],
    };

    forge_main(tests, LocalFactory::new(get_diem_node().to_str().unwrap()))
}

//TODO Make public test later
#[derive(Debug)]
struct GetMetadata;

impl Test for GetMetadata {
    fn name(&self) -> &'static str {
        "get_metadata"
    }
}

impl AdminTest for GetMetadata {
    fn run<'t>(&self, ctx: &mut AdminContext<'t>) -> Result<()> {
        let client = ctx.client();

        let metadata = client.get_metadata()?.into_inner();

        // get_metadata documentation states that the following fields will be present when no version
        // argument is provided
        metadata.script_hash_allow_list.unwrap();
        metadata.diem_version.unwrap();
        metadata.module_publishing_allowed.unwrap();
        metadata.dual_attestation_limit.unwrap();

        Ok(())
    }
}

#[derive(Debug)]
struct FundAccount;

impl Test for FundAccount {
    fn name(&self) -> &'static str {
        "fund_account"
    }
}

impl PublicUsageTest for FundAccount {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let client = ctx.client();

        let account = ctx.random_account();
        let amount = 1000;
        let currency = Currency::XUS;
        ctx.fund(currency, account.authentication_key(), amount)?;

        let account_view = client.get_account(account.address())?.into_inner().unwrap();
        let balance = account_view
            .balances
            .iter()
            .find(|b| b.currency == currency)
            .unwrap();
        assert_eq!(balance.amount, amount);

        Ok(())
    }
}

// TODO Remove everything below here
// The following is copied from the workspace-builder in the smoke-test crate. Its only intended to
// be here temporarily

fn get_diem_node() -> PathBuf {
    let output = Command::new("cargo")
        .current_dir(workspace_root())
        .args(&["build", "--bin=diem-node"])
        .output()
        .expect("Failed to build diem-node");

    if output.status.success() {
        let bin_path = build_dir().join(format!("{}{}", "diem-node", env::consts::EXE_SUFFIX));
        if !bin_path.exists() {
            panic!(
                "Can't find binary diem-node in expected path {:?}",
                bin_path
            );
        }

        bin_path
    } else {
        panic!("Faild to build diem-node");
    }
}

// Path to top level workspace
pub fn workspace_root() -> PathBuf {
    let mut path = build_dir();
    while !path.ends_with("target") {
        path.pop();
    }
    path.pop();
    path
}

// Path to the directory where build artifacts live.
fn build_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .map(|mut path| {
            path.pop();
            if path.ends_with("deps") {
                path.pop();
            }
            path
        })
        .expect("Can't find the build directory. Cannot continue running tests")
}
