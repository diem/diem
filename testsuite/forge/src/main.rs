// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{env, path::PathBuf, process::Command};

use diem_sdk::{
    client::{BlockingClient, MethodRequest},
    move_types::account_address::AccountAddress,
    transaction_builder::Currency,
};
use forge::{forge_main, ForgeConfig, Result, *};
use itertools::Itertools;
use rand::SeedableRng;
use std::time::Duration;
use tokio::runtime::Runtime;

fn main() -> Result<()> {
    let tests = ForgeConfig {
        public_usage_tests: &[&FundAccount, &TransferCoins],
        admin_tests: &[&GetMetadata],
        network_tests: &[&EmitTransaction],
    };

    //forge_main(tests, LocalFactory::new(get_diem_node().to_str().unwrap()));
    // let k8s_fac = K8sFactory::new();
    // k8s_fac.launch_swarm(4);
    forge_main(tests, K8sFactory::new());

    Ok(())
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

pub fn check_account_balance(
    client: &BlockingClient,
    currency: Currency,
    account_address: AccountAddress,
    expected: u64,
) -> Result<()> {
    let account_view = client.get_account(account_address)?.into_inner().unwrap();
    let balance = account_view
        .balances
        .iter()
        .find(|b| b.currency == currency)
        .unwrap();
    assert_eq!(balance.amount, expected);

    Ok(())
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
        check_account_balance(&client, currency, account.address(), amount)?;

        Ok(())
    }
}

#[derive(Debug)]
struct TransferCoins;

impl Test for TransferCoins {
    fn name(&self) -> &'static str {
        "transfer_coins"
    }
}

impl PublicUsageTest for TransferCoins {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let mut account = ctx.random_account();
        let amount = 1000;
        let currency = Currency::XUS;
        let client = ctx.client();
        ctx.fund(currency, account.authentication_key(), amount)?;

        let mut payer = ctx.random_account();
        let payee = ctx.random_account();
        let create_payer =
            account.sign_with_transaction_builder(ctx.tx_factory().create_child_vasp_account(
                currency,
                payer.authentication_key(),
                false,
                100,
            ));
        let create_payee =
            account.sign_with_transaction_builder(ctx.tx_factory().create_child_vasp_account(
                currency,
                payee.authentication_key(),
                false,
                0,
            ));
        let batch = vec![
            MethodRequest::submit(&create_payer)?,
            MethodRequest::submit(&create_payee)?,
        ];
        client.batch(batch)?;
        client.wait_for_signed_transaction(&create_payer, None, None)?;
        client.wait_for_signed_transaction(&create_payee, None, None)?;
        check_account_balance(&client, currency, payer.address(), 100)?;

        ctx.transfer_coins(currency, &mut payer, payee.address(), 10)?;
        check_account_balance(&client, currency, payer.address(), 90)?;
        check_account_balance(&client, currency, payee.address(), 10)?;
        let account_view = client.get_account(payee.address())?.into_inner().unwrap();
        let balance = account_view
            .balances
            .iter()
            .find(|b| b.currency == currency)
            .unwrap();
        assert_eq!(balance.amount, 10);

        Ok(())
    }
}

#[derive(Debug)]
struct RestartValidator;

impl Test for RestartValidator {
    fn name(&self) -> &'static str {
        "restart_validator"
    }
}

impl NetworkTest for RestartValidator {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let node_id = NodeId::new(0);
        let node = ctx.swarm().validator_mut(node_id);
        node.health_check().expect("node health check failed");
        node.stop()?;
        println!("Restarting node {}", node.node_id().as_inner());
        node.start()?;
        // wait node to recovery
        std::thread::sleep(Duration::from_millis(1000));
        node.health_check().expect("node health check failed");

        Ok(())
    }
}

#[derive(Debug)]
struct EmitTransaction;

impl Test for EmitTransaction {
    fn name(&self) -> &'static str {
        "emit_transaction"
    }
}

impl NetworkTest for EmitTransaction {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let duration = Duration::from_secs(10);
        let rng = ::rand::rngs::StdRng::from_seed([0; 32]);
        let validator_clients = ctx
            .swarm()
            .validators()
            .into_iter()
            .map(|n| n.json_rpc_client())
            .collect_vec();
        let mut emitter = TxEmitter::new(ctx.swarm().chain_info(), rng);
        let rt = Runtime::new().unwrap();
        let stats = rt
            .block_on(emitter.emit_txn_for(duration, EmitJobRequest::default(validator_clients)))
            .unwrap();
        println!("{:?}", stats);

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
