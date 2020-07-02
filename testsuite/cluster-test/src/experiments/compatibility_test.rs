// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    experiments::{update_batch_instance, Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
    tx_emitter::EmitJobRequest,
};
use async_trait::async_trait;
use libra_logger::prelude::*;
use std::{collections::HashSet, fmt, iter::once, time::Duration};
use structopt::StructOpt;

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
            .emit_txn_for(job_duration.clone(), fullnode_txn_job.clone())
            .await?;

        info!("1. Changing the images for the first instance to validate storage");
        let first_node = vec![self.first_node.clone()];
        update_batch_instance(context, &first_node, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(
                job_duration.clone(),
                EmitJobRequest::for_instances(first_node, context.global_emit_job_request),
            )
            .await?;

        info!("2. Changing images for the first batch to test consensus");
        update_batch_instance(context, &self.first_batch, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration.clone(), validator_txn_job.clone())
            .await?;

        info!("3. Changing images for the rest of the nodes");
        update_batch_instance(context, &self.second_batch, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration.clone(), validator_txn_job)
            .await?;

        info!("4. Changing images for full nodes");
        update_batch_instance(context, &self.full_nodes, self.updated_image_tag.clone()).await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration.clone(), fullnode_txn_job)
            .await?;
        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(9 * 60)
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
