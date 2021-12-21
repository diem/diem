// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use structopt::StructOpt;
use tokio::time;

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};
use async_trait::async_trait;
use diem_logger::info;
use std::time::Instant;

#[derive(StructOpt, Debug)]
pub struct RecoveryTimeParams {
    #[structopt(
        long,
        default_value = "100",
        help = "Number of accounts to mint before starting the experiment"
    )]
    pub num_accounts_to_mint: u64,
}

pub struct RecoveryTime {
    params: RecoveryTimeParams,
    instance: Instance,
}

impl ExperimentParam for RecoveryTimeParams {
    type E = RecoveryTime;
    fn build(self, cluster: &Cluster) -> Self::E {
        let instance = cluster.random_validator_instance();
        Self::E {
            params: self,
            instance,
        }
    }
}

#[async_trait]
impl Experiment for RecoveryTime {
    fn affected_validators(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        result.insert(self.instance.peer_name().clone());
        result
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );

        txn_emitter
            .mint_accounts(
                &crate::util::emit_job_request_for_instances(
                    context.cluster.validator_instances().to_vec(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
                self.params.num_accounts_to_mint as usize,
            )
            .await?;
        info!("Stopping {}", self.instance);
        self.instance.stop().await?;
        info!("Deleting db and restarting node for {}", self.instance);
        self.instance.clean_data().await?;
        self.instance.start().await?;
        info!("Waiting for instance to be up: {}", self.instance);
        self.instance
            .wait_server_ready(Instant::now() + Duration::from_secs(120))
            .await?;
        let start_instant = Instant::now();
        info!(
            "Instance {} is up. Waiting for it to start committing.",
            self.instance
        );
        while self
            .instance
            .counter("diem_consensus_last_committed_round")
            .is_err()
        {
            time::sleep(Duration::from_secs(1)).await;
        }
        let time_to_recover = start_instant.elapsed();
        let recovery_rate =
            self.params.num_accounts_to_mint as f64 / time_to_recover.as_secs() as f64;
        let result = format!("Recovery rate : {:.1} txn/sec", recovery_rate,);
        info!("{}", result);
        context.report.report_text(result);
        context
            .report
            .report_metric(self, "recovery_rate", recovery_rate);
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for RecoveryTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RecoveryTime")
    }
}
