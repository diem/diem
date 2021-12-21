// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use async_trait::async_trait;
use diem_infallible::duration_since_epoch;
use diem_logger::info;
use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use futures::future::try_join_all;
use rand::{prelude::StdRng, rngs::OsRng, Rng, SeedableRng};
use std::{
    collections::HashSet,
    fmt,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::time;

#[derive(StructOpt, Debug)]
pub struct TwinValidatorsParams {
    #[structopt(long, default_value = "1", help = "Set twin node pair number")]
    pub pair: usize,
}

pub struct TwinValidators {
    instances: Vec<Instance>,
    twin_validators: Vec<Instance>,
}

impl ExperimentParam for TwinValidatorsParams {
    type E = TwinValidators;
    fn build(self, cluster: &Cluster) -> Self::E {
        if self.pair >= cluster.validator_instances().len() {
            panic!(
                "pair number {} can not equal or more than validator number {}",
                self.pair,
                cluster.validator_instances().len()
            );
        }
        let mut instances = cluster.validator_instances().to_vec();
        let mut twin_validators = vec![];
        let mut rnd = rand::thread_rng();
        for _i in 0..self.pair {
            twin_validators.push(instances.remove(rnd.gen_range(1..instances.len())));
        }
        Self::E {
            instances,
            twin_validators,
        }
    }
}

#[async_trait]
impl Experiment for TwinValidators {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.twin_validators)
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context.cluster.random_validator_instance().rest_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
        );
        let buffer = Duration::from_secs(60);
        let window = Duration::from_secs(240);
        let mut new_instances = vec![];
        let mut origin_instances = vec![];
        for inst in self.twin_validators.iter() {
            info!("Stopping origin validator {}", inst);
            inst.stop().await?;
            let mut new_twin_config = inst.instance_config().clone();
            new_twin_config.make_twin(1);
            info!(
                "Deleting db and starting twin node {} for {}",
                new_twin_config.pod_name(),
                inst
            );
            context
                .cluster_swarm
                .clean_data(
                    &context
                        .cluster_swarm
                        .get_node_name(&new_twin_config.pod_name())
                        .await?,
                )
                .await?;
            let new_inst = context
                .cluster_swarm
                .spawn_new_instance(new_twin_config)
                .await?;
            info!("Waiting for twin node to be up: {}", new_inst);
            new_inst
                .wait_server_ready(Instant::now() + Duration::from_secs(120))
                .await?;
            info!("Twin node {} is up", new_inst);
            info!("Restarting origin validator {}", inst);
            inst.start().await?;
            origin_instances.push(inst.clone());
            new_instances.push(new_inst.clone());
        }
        let instances = self.instances.clone();
        let emit_job_request = crate::util::emit_job_request_for_instances(
            instances,
            context.global_emit_job_request,
            0,
            0,
        );
        info!("Starting txn generation");
        let stats = txn_emitter.emit_txn_for(window, emit_job_request).await?;
        let end = duration_since_epoch() - buffer;
        let start = end - window + 2 * buffer;
        info!(
            "Link to dashboard : {}",
            context.prometheus.link_to_dashboard(start, end)
        );
        info!("Stopping origin validators");
        let futures: Vec<_> = origin_instances.iter().map(|ic| ic.stop()).collect();
        try_join_all(futures).await?;
        time::sleep(Duration::from_secs(10)).await;
        info!("Stopping twin validators");
        let futures: Vec<_> = new_instances.iter().map(|ic| ic.stop()).collect();
        try_join_all(futures).await?;
        time::sleep(Duration::from_secs(10)).await;
        info!("Restarting origin validators");
        let futures: Vec<_> = origin_instances.iter().map(|ic| ic.start()).collect();
        try_join_all(futures).await?;
        time::sleep(Duration::from_secs(10)).await;

        for inst in origin_instances.iter() {
            info!("Waiting for origin node to be up: {}", inst);
            inst.wait_server_ready(Instant::now() + Duration::from_secs(120))
                .await?;
            info!("Origin node {} is up", inst);
        }

        context
            .report
            .report_txn_stats(self.to_string(), stats, window, "");
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(20 * 60)
    }
}

impl fmt::Display for TwinValidators {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Twin validator [")?;
        for instance in self.twin_validators.iter() {
            write!(f, "{}, ", instance.instance_config().pod_name())?;
        }
        write!(f, "]")
    }
}
