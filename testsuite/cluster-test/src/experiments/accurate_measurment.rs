// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance::Instance,
};
use anyhow::Result;
use async_trait::async_trait;
use core::fmt;
use diem_sdk::transaction_builder::TransactionFactory;
use forge::{EmitJobRequest, TxnEmitter};
use futures::FutureExt;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{collections::HashSet, convert::TryInto, time::Duration};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct AccurateMeasurementParams {
    #[structopt(
    long,
    default_value = Box::leak(format!("{}", DEFAULT_BENCH_DURATION).into_boxed_str()),
    help = "Duration of an experiment in seconds"
    )]
    pub duration: u64,
    #[structopt(long, help = "Set fixed tps as the base tps number of experiment")]
    pub base_tps: u64,
    #[structopt(long, help = "Step numbers to change tps")]
    pub step_num: u64,
    #[structopt(long, help = "How may tps change for each step")]
    pub step_length: u64,
}

pub struct AccurateMeasurement {
    validators: Vec<Instance>,
    fullnodes: Vec<Instance>,
    duration: Duration,
    base_tps: u64,
    step_num: u64,
    step_length: u64,
}

pub const DEFAULT_BENCH_DURATION: u64 = 600;

impl ExperimentParam for AccurateMeasurementParams {
    type E = AccurateMeasurement;
    fn build(self, cluster: &Cluster) -> Self::E {
        let validators = cluster.validator_instances().to_vec();
        let fullnodes = cluster.fullnode_instances().to_vec();
        Self::E {
            validators,
            fullnodes,
            duration: Duration::from_secs(self.duration),
            base_tps: self.base_tps,
            step_num: self.step_num,
            step_length: self.step_length,
        }
    }
}

#[async_trait]
impl Experiment for AccurateMeasurement {
    fn affected_validators(&self) -> HashSet<String> {
        HashSet::new()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );
        let instances = if context.emit_to_validator {
            self.validators.clone()
        } else {
            self.fullnodes.clone()
        };
        for i in 0..self.step_num {
            let tps = self.base_tps + self.step_length * i;
            let window = self.duration / self.step_num as u32;
            let emit_job_request =
                EmitJobRequest::new(instances.iter().map(Instance::rest_client).collect())
                    .fixed_tps(tps.try_into().unwrap());
            let emit_txn = txn_emitter.emit_txn_for(window, emit_job_request).boxed();
            let stats = emit_txn.await?;
            // Report
            let test_step = format!("Step {}", i);
            context
                .report
                .report_txn_stats(test_step, stats, window, "");
        }

        Ok(())
    }

    fn deadline(&self) -> Duration {
        let buffer = Duration::from_secs(60);
        self.duration + buffer
    }
}

impl fmt::Display for AccurateMeasurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Perf Measurement: start tps {}, steps number {}, step length = {}",
            self.base_tps, self.step_num, self.step_length
        )?;
        Ok(())
    }
}
