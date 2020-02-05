// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use structopt::StructOpt;
use tokio::time;

use crate::cluster::Cluster;
use crate::effects::{DeleteLibraData, Effect, StopContainer};
use crate::experiments::{Context, ExperimentParam};
use crate::instance::Instance;
use crate::tx_emitter::EmitJobRequest;
use crate::{effects::Action, experiments::Experiment};
use async_trait::async_trait;
use slog_scope::info;
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
        let stop_effect = StopContainer::new(self.instance.clone());
        let delete_action = DeleteLibraData::new(self.instance.clone());
        context
            .tx_emitter
            .mint_accounts(
                &EmitJobRequest::for_instances(context.cluster.validator_instances().to_vec()),
                self.params.num_accounts_to_mint as usize,
            )
            .await?;
        info!("Stopping {}", self.instance);
        stop_effect.activate().await?;
        info!("Deleted db for {}", self.instance);
        delete_action.apply().await?;
        info!("Starting instance {}", self.instance);
        stop_effect.deactivate().await?;
        info!("Waiting for instance to be up: {}", self.instance);
        while !self.instance.is_up() {
            time::delay_for(Duration::from_secs(1)).await;
        }
        let start_instant = Instant::now();
        info!(
            "Instance {} is up. Waiting for it to start committing.",
            self.instance
        );
        while self
            .instance
            .counter("libra_consensus_last_committed_round")
            .is_err()
        {
            time::delay_for(Duration::from_secs(1)).await;
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
