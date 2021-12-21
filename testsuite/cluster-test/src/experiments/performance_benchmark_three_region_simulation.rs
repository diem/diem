// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    effects::{self, network_delay},
    experiments::{Context, Experiment, ExperimentParam},
};
use anyhow::Result;
use async_trait::async_trait;
use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{
    fmt::{Display, Error, Formatter},
    time::Duration,
};
use structopt::StructOpt;

pub struct PerformanceBenchmarkThreeRegionSimulation {
    cluster: Cluster,
}

#[derive(StructOpt, Debug)]
pub struct PerformanceBenchmarkThreeRegionSimulationParams {}

impl ExperimentParam for PerformanceBenchmarkThreeRegionSimulationParams {
    type E = PerformanceBenchmarkThreeRegionSimulation;
    fn build(self, cluster: &Cluster) -> Self::E {
        Self::E {
            cluster: cluster.clone(),
        }
    }
}

#[async_trait]
impl Experiment for PerformanceBenchmarkThreeRegionSimulation {
    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );

        let num_nodes = self.cluster.validator_instances().len();
        let split_country_num = ((num_nodes as f64) * 0.8) as usize;
        let split_region_num = split_country_num / 2;
        let (us, euro) = self.cluster.split_n_validators_random(split_country_num);
        let (us_west, us_east) = us.split_n_validators_random(split_region_num);
        let mut effects = network_delay::three_region_simulation_effects(
            (
                us_west.validator_instances().to_vec(),
                us_east.validator_instances().to_vec(),
                euro.validator_instances().to_vec(),
            ),
            (
                Duration::from_millis(60), // us_east<->eu one way delay
                Duration::from_millis(95), // us_west<->eu one way delay
                Duration::from_millis(40), // us_west<->us_east one way delay
            ),
        );

        effects::activate_all(&mut effects).await?;

        let window = Duration::from_secs(240);
        let emit_job_request = if context.emit_to_validator {
            crate::util::emit_job_request_for_instances(
                context.cluster.validator_instances().to_vec(),
                context.global_emit_job_request,
                0,
                0,
            )
        } else {
            crate::util::emit_job_request_for_instances(
                context.cluster.fullnode_instances().to_vec(),
                context.global_emit_job_request,
                0,
                0,
            )
        };
        let stats = txn_emitter.emit_txn_for(window, emit_job_request).await?;
        effects::deactivate_all(&mut effects).await?;
        context
            .report
            .report_txn_stats(self.to_string(), stats, window, "");
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(600)
    }
}

impl Display for PerformanceBenchmarkThreeRegionSimulation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "3 Region Simulation")
    }
}
