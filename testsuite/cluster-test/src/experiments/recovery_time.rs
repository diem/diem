// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashSet, fmt, time::Duration};

use anyhow::Result;
use futures::future::{BoxFuture, FutureExt};
use structopt::StructOpt;
use tokio::time;

use crate::cluster::Cluster;
use crate::effects::{DeleteLibraData, Effect, StopContainer};
use crate::experiments::{Context, ExperimentParam};
use crate::instance::Instance;
use crate::tx_emitter::{EmitJobRequest, EmitThreadParams};
use crate::{effects::Action, experiments::Experiment};
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

impl Experiment for RecoveryTime {
    fn affected_validators(&self) -> HashSet<String> {
        let mut result = HashSet::new();
        result.insert(self.instance.peer_name().clone());
        result
    }

    fn run<'a>(&'a mut self, context: &'a mut Context) -> BoxFuture<'a, Result<Option<String>>> {
        async move {
            let stop_effect = StopContainer::new(self.instance.clone());
            let delete_action = DeleteLibraData::new(self.instance.clone());
            context
                .tx_emitter
                .mint_accounts(
                    &EmitJobRequest {
                        instances: context.cluster.validator_instances().to_vec(),
                        accounts_per_client: 100,
                        thread_params: EmitThreadParams::default(),
                    },
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
            let result = format!(
                "Recovery rate : {:.1} txn/sec",
                self.params.num_accounts_to_mint as f64 / time_to_recover.as_secs() as f64
            );
            info!("{}", result);
            Ok(Some(result))
        }
            .boxed()
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
