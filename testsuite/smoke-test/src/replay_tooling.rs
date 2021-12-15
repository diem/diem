// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use diem_rest_client::Transaction;
use diem_sdk::{transaction_builder::Currency, types::account_address::AccountAddress};
use diem_transaction_replay::DiemDebugger;
use forge::{PublicUsageContext, PublicUsageTest, Result, Test};
use tokio::runtime::Runtime;

pub struct ReplayTooling;

impl Test for ReplayTooling {
    fn name(&self) -> &'static str {
        "smoke-test::replay-tooling"
    }

    // FIXME(aldenhu): maybe support state read at non-checkpoint versions
    fn ignored(&self) -> bool {
        true
    }
}

impl PublicUsageTest for ReplayTooling {
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()> {
        let client = ctx.rest_client();
        // we need turn this into async for making this test fully async
        let json_debugger = DiemDebugger::json_rpc(ctx.url())?;

        let treasury_account_address =
            AccountAddress::from_hex("0000000000000000000000000b1e55ed")?;

        let runtime = Runtime::new().unwrap();
        let treasury_account = runtime
            .block_on(client.get_account(treasury_account_address))?
            .into_inner();
        let mut account1 = ctx.random_account();
        let account2 = ctx.random_account();
        runtime.block_on(ctx.create_parent_vasp_account(account1.authentication_key()))?;
        runtime.block_on(ctx.create_parent_vasp_account(account2.authentication_key()))?;
        runtime.block_on(ctx.fund(account1.address(), 100))?;
        runtime.block_on(ctx.fund(account2.address(), 100))?;
        let txn = account1.sign_with_transaction_builder(ctx.transaction_factory().peer_to_peer(
            Currency::XUS,
            account2.address(),
            3,
        ));
        let txn = runtime.block_on(client.submit_and_wait(&txn))?.into_inner();
        let (txn_version, txn_gas_used) = match txn {
            Transaction::UserTransaction(user_txn) => {
                (user_txn.info.version.0, user_txn.info.gas_used.0)
            }
            _ => bail!("unexpected transaction type: {:?}", txn),
        };
        let replay_result = json_debugger
            // FIXME(aldenhu): this hack is to skip backwards past the epilogue txn, so that a state checkpoint is reached
            .execute_past_transactions(txn_version - 1, 1, false)?
            .pop()
            .unwrap();
        assert_eq!(replay_result.gas_used(), txn_gas_used);

        let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../diem-move/transaction-replay/examples/account_exists.move")
            .canonicalize()?;

        let bisect_result = json_debugger
            .bisect_transactions_by_script(
                script_path.to_str().unwrap(),
                account1.address(),
                0,
                txn_version,
                None,
            )?
            .unwrap();

        let account_creation_txn = runtime
            .block_on(client.get_account_transactions(
                treasury_account_address,
                Some(treasury_account.sequence_number),
                Some(1),
            ))?
            .into_inner()
            .into_iter()
            .next()
            .unwrap();

        match account_creation_txn {
            Transaction::UserTransaction(user_txn) => {
                assert_eq!(user_txn.info.version.0 + 1, bisect_result);
            }
            _ => bail!("unexpected transaction type: {:?}", account_creation_txn),
        }

        Ok(())
    }
}
