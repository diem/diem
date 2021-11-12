// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::generate_traffic;
use anyhow::anyhow;
use diem_sdk::{client::Client, transaction_builder::TransactionFactory};
use diem_types::{chain_id::{ChainId, NamedChain}, transaction::TransactionPayload};
use forge::{execute_and_wait_transactions, NetworkContext, NetworkTest, Result, Test};
use tokio::{runtime::Runtime, time::Duration};

pub struct PerformanceBenchmark;

impl Test for PerformanceBenchmark {
    fn name(&self) -> &'static str {
        "performance::performance-benchmark"
    }
}

impl NetworkTest for PerformanceBenchmark {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let duration = Duration::from_secs(240);
        let all_validators = ctx
            .swarm()
            .validators()
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();

        // Generate some traffic
        let txn_stat = generate_traffic(ctx, &all_validators, duration, 0, None)?;
        ctx.report
            .report_txn_stats(self.name().to_string(), txn_stat, duration);

        Ok(())
    }
}

pub struct ParallelPerformanceBenchamrk;

impl Test for ParallelPerformanceBenchamrk {
    fn name(&self) -> &'static str {
        "performance::parallel-performance-benchmark"
    }
}

impl NetworkTest for ParallelPerformanceBenchamrk {
    fn run<'t>(&self, ctx: &mut NetworkContext<'t>) -> Result<()> {
        let duration = Duration::from_secs(240);
        let all_validators = ctx
            .swarm()
            .validators()
            .map(|v| v.peer_id())
            .collect::<Vec<_>>();

        let client = Client::new(
            ctx.swarm()
                .validators()
                .last()
                .ok_or_else(|| anyhow!("Node is not available"))?
                .json_rpc_endpoint(),
        );

        let mut diem_root_account = &mut ctx.swarm().chain_info().root_account;
        let txn = diem_root_account.sign_with_transaction_builder(
            TransactionFactory::new(ChainId::new(NamedChain::DEVNET.id())).payload(TransactionPayload::WriteSet(
                diem_writeset_generator::encode_enable_parallel_execution_with_config(),
            )),
        );

        let rt = Runtime::new()?;
        rt.block_on(execute_and_wait_transactions(
            &client,
            &mut diem_root_account,
            vec![txn],
        ))?;

        // Generate some traffic
        let txn_stat = generate_traffic(ctx, &all_validators, duration, 0, None)?;
        ctx.report
            .report_txn_stats(self.name().to_string(), txn_stat, duration);

        Ok(())
    }
}
