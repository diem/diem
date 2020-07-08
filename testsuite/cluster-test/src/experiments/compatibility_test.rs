// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    tx_emitter::EmitJobRequest,
};
use async_trait::async_trait;
use futures::future::try_join_all;
use libra_logger::prelude::*;
use std::{
    collections::HashSet,
    fmt,
    iter::once,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::time;

/// Reboot `updated_instance` with newer image tag
pub async fn update_batch_instance(
    context: &mut Context<'_>,
    updated_instance: &[Instance],
    updated_tag: String,
) -> anyhow::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(2 * 60);

    info!("Stop Existing instances.");
    let futures: Vec<_> = updated_instance.iter().map(Instance::stop).collect();
    try_join_all(futures).await?;

    info!("Reinstantiate a set of new nodes.");
    let futures: Vec<_> = updated_instance
        .iter()
        .map(|instance| {
            let mut newer_config = instance.instance_config().clone();
            newer_config.replace_tag(updated_tag.clone()).unwrap();
            context
                .cluster_swarm
                .spawn_new_instance(newer_config, false)
        })
        .collect();
    let instances = try_join_all(futures).await?;

    info!("Wait for the instances to recover.");
    let futures: Vec<_> = instances
        .iter()
        .map(|instance| instance.wait_json_rpc(deadline))
        .collect();
    try_join_all(futures).await?;

    // Add a timeout to have wait for validators back to healthy mode.
    // TODO: Replace this with a blocking health check.
    info!("Wait for the instance to sync up with peers");
    time::delay_for(Duration::from_secs(20)).await;
    Ok(())
}

#[derive(StructOpt, Debug)]
pub struct ComaptiblityTestParams {
    #[structopt(
        long,
        default_value = "15",
        help = "Number of nodes to update in the first batch"
    )]
    pub count: usize,
    #[structopt(long, help = "Image tag of newer validator software")]
    pub updated_image_tag: String,
}

pub struct CompatibilityTest {
    first_node: Instance,
    first_batch: Vec<Instance>,
    second_batch: Vec<Instance>,
    full_nodes: Vec<Instance>,
    updated_image_tag: String,
}

impl ExperimentParam for ComaptiblityTestParams {
    type E = CompatibilityTest;
    fn build(self, cluster: &Cluster) -> Self::E {
        if self.count > cluster.validator_instances().len() || self.count == 0 {
            panic!(
                "Can not reboot {} validators in cluster with {} instances",
                self.count,
                cluster.validator_instances().len()
            );
        }
        let (first_batch, second_batch) = cluster.split_n_validators_random(self.count);
        let mut first_batch = first_batch.into_validator_instances();
        let first_node = first_batch
            .pop()
            .expect("Requires at least one validator in the first batch");

        Self::E {
            first_node,
            first_batch,
            second_batch: second_batch.into_validator_instances(),
            full_nodes: cluster.fullnode_instances().to_vec(),
            updated_image_tag: self.updated_image_tag,
        }
    }
}

#[async_trait]
impl Experiment for CompatibilityTest {
    fn affected_validators(&self) -> HashSet<String> {
        instance::instancelist_to_set(&self.first_batch)
            .union(&instance::instancelist_to_set(&self.second_batch))
            .cloned()
            .chain(once(self.first_node.peer_name().clone()))
            .collect()
    }

    async fn run(&mut self, context: &mut Context<'_>) -> anyhow::Result<()> {
        let validator_txn_job = EmitJobRequest::for_instances(
            context.cluster.validator_instances().to_vec(),
            context.global_emit_job_request,
        );
        let fullnode_txn_job = EmitJobRequest::for_instances(
            context.cluster.fullnode_instances().to_vec(),
            context.global_emit_job_request,
        );
        let job_duration = Duration::from_secs(3);

        // Generate some traffic
        context
            .tx_emitter
            .emit_txn_for(job_duration, fullnode_txn_job.clone())
            .await?;

        info!("1. Changing the images for the first instance to validate storage");
        let first_node = vec![self.first_node.clone()];
        update_batch_instance(context, &first_node, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(
                job_duration,
                EmitJobRequest::for_instances(first_node, context.global_emit_job_request),
            )
            .await?;

        info!("2. Changing images for the first batch to test consensus");
        update_batch_instance(context, &self.first_batch, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration, validator_txn_job.clone())
            .await?;

        info!("3. Changing images for the rest of the nodes");
        update_batch_instance(context, &self.second_batch, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration, validator_txn_job)
            .await?;

        info!("4. Changing images for full nodes");
        update_batch_instance(context, &self.full_nodes, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration, fullnode_txn_job)
            .await?;
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(16 * 60)
    }
}

impl fmt::Display for CompatibilityTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Updating [")?;
        for instance in self.first_batch.iter() {
            write!(f, "{}, ", instance)?;
        }
        for instance in self.second_batch.iter() {
            write!(f, "{}, ", instance)?;
        }
        write!(f, "]")?;
        writeln!(f, "Updated Config: {:?}", self.updated_image_tag)
    }
}
