// Copyright (c) The Diem Core Contributors
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
use diem_logger::prelude::*;
use futures::future::try_join_all;
use std::{
    collections::HashSet,
    env, fmt,
    iter::once,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::time;

/// Reboot `updated_instance` with newer image tag
pub async fn update_batch_instance(
    context: &mut Context<'_>,
    updated_instance: &[Instance],
    updated_lsr: &[Instance],
    updated_tag: String,
) -> anyhow::Result<()> {
    info!("Stop Existing instances.");
    let futures: Vec<_> = updated_instance.iter().map(Instance::stop).collect();
    try_join_all(futures).await?;

    if !updated_lsr.is_empty() {
        info!("Stop associated lsr instances.");
        let futures: Vec<_> = updated_lsr.iter().map(Instance::stop).collect();
        try_join_all(futures).await?;
        info!("Reinstantiate a set of new lsr.");
        let futures: Vec<_> = updated_lsr
            .iter()
            .map(|instance| {
                let mut newer_config = instance.instance_config().clone();
                newer_config.replace_tag(updated_tag.clone()).unwrap();
                context.cluster_swarm.spawn_new_instance(newer_config)
            })
            .collect();
        try_join_all(futures).await?;
        info!("Wait for the instance to sync up with peers");
        time::delay_for(Duration::from_secs(20)).await;
    }

    info!("Reinstantiate a set of new nodes.");
    let futures: Vec<_> = updated_instance
        .iter()
        .map(|instance| {
            let mut newer_config = instance.instance_config().clone();
            newer_config.replace_tag(updated_tag.clone()).unwrap();
            context.cluster_swarm.spawn_new_instance(newer_config)
        })
        .collect();
    let instances = try_join_all(futures).await?;

    info!("Wait for the instances to recover.");
    let deadline = Instant::now() + Duration::from_secs(5 * 60);
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

pub fn get_instance_list_str(batch: &[Instance]) -> String {
    let mut nodes_list = String::from("");
    for instance in batch.iter() {
        nodes_list.push_str(&instance.to_string());
        nodes_list.push_str(", ")
    }
    nodes_list
}

#[derive(StructOpt, Debug)]
pub struct CompatiblityTestParams {
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
    first_lsr: Vec<Instance>,
    first_batch: Vec<Instance>,
    first_batch_lsr: Vec<Instance>,
    second_batch: Vec<Instance>,
    second_batch_lsr: Vec<Instance>,
    full_nodes: Vec<Instance>,
    updated_image_tag: String,
}

impl ExperimentParam for CompatiblityTestParams {
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
        let second_batch = second_batch.into_validator_instances();
        let first_node = first_batch
            .pop()
            .expect("Requires at least one validator in the first batch");
        let mut first_lsr = vec![];
        let mut first_batch_lsr = vec![];
        let mut second_batch_lsr = vec![];
        if !cluster.lsr_instances().is_empty() {
            first_batch_lsr = cluster.lsr_instances_for_validators(&first_batch);
            second_batch_lsr = cluster.lsr_instances_for_validators(&second_batch);
            first_lsr = cluster.lsr_instances_for_validators(&[first_node.clone()]);
        }

        Self::E {
            first_node,
            first_lsr,
            first_batch,
            first_batch_lsr,
            second_batch,
            second_batch_lsr,
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
            0,
        );
        let fullnode_txn_job = EmitJobRequest::for_instances(
            context.cluster.fullnode_instances().to_vec(),
            context.global_emit_job_request,
            0,
        );
        let job_duration = Duration::from_secs(3);
        context.report.report_text(format!(
            "Compatibility test results for {} ==> {} (PR)",
            context.current_tag, self.updated_image_tag
        ));

        // Generate some traffic
        let msg = format!(
            "1. All instances running {}, generating some traffic on network",
            context.current_tag
        );
        info!("{}", msg);
        context.report.report_text(msg);
        context
            .tx_emitter
            .emit_txn_for(job_duration, fullnode_txn_job.clone())
            .await
            .map_err(|e| anyhow::format_err!("Failed to generate traffic: {}", e))?;

        let msg = format!(
            "2. First validator {} ==> {}, to validate storage",
            context.current_tag, self.updated_image_tag
        );
        info!("{}", msg);
        info!("Upgrading validator: {}", self.first_node);
        context.report.report_text(msg);
        let first_node = vec![self.first_node.clone()];
        update_batch_instance(
            context,
            &first_node,
            &self.first_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;
        context
            .tx_emitter
            .emit_txn_for(
                job_duration,
                EmitJobRequest::for_instances(first_node, context.global_emit_job_request, 0),
            )
            .await
            .map_err(|e| anyhow::format_err!("Storage backwards compat broken: {}", e))?;

        let msg = format!(
            "3. First batch validators ({}) {} ==> {}, to test consensus",
            self.first_batch.len(),
            context.current_tag,
            self.updated_image_tag
        );
        info!("{}", msg);
        info!(
            "Upgrading validators: {}",
            get_instance_list_str(&self.first_batch)
        );
        context.report.report_text(msg);
        update_batch_instance(
            context,
            &self.first_batch,
            &self.first_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration, validator_txn_job.clone())
            .await
            .map_err(|e| anyhow::format_err!("Consensus backwards compat broken: {}", e))?;

        let msg = format!(
            "4. Second batch validators ({}) {} ==> {}, to upgrade rest of the validators",
            self.second_batch.len(),
            context.current_tag,
            self.updated_image_tag
        );
        info!("{}", msg);
        info!(
            "Upgrading validators: {}",
            get_instance_list_str(&self.second_batch)
        );
        context.report.report_text(msg);
        update_batch_instance(
            context,
            &self.second_batch,
            &self.second_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;
        context
            .tx_emitter
            .emit_txn_for(job_duration, validator_txn_job)
            .await
            .map_err(|e| {
                anyhow::format_err!("Failed to upgrade rest of validator images: {}", e)
            })?;

        // TODO(rustielin): fullnode reboot in cluster-test breaks with current version of config-builder
        //                  skipping for now, so cluster is not restored to same version across all instances
        let disable_fn_upgrade = env::var("DISABLE_FN_UPGRADE").is_ok();
        if disable_fn_upgrade {
            let msg = format!(
                "5. Reset all nodes ==> {}, to finish the network upgrade",
                self.updated_image_tag
            );
            info!("{}", msg);
            context.report.report_text(msg);
            context.cluster_builder.current_tag = self.updated_image_tag.clone();
            context
                .cluster_builder
                .setup_cluster(context.cluster_builder_params, false)
                .await?;
        } else {
            let msg = format!(
                "5. All full nodes ({}) {} ==> {}, to finish the network upgrade",
                self.full_nodes.len(),
                context.current_tag,
                self.updated_image_tag
            );
            info!("{}", msg);
            info!(
                "Upgrading full nodes: {}",
                get_instance_list_str(&self.full_nodes)
            );
            context.report.report_text(msg);
            update_batch_instance(
                context,
                &self.full_nodes,
                &[],
                self.updated_image_tag.clone(),
            )
            .await?;
            context
                .tx_emitter
                .emit_txn_for(job_duration, fullnode_txn_job)
                .await
                .map_err(|e| anyhow::format_err!("Failed to upgrade full node images: {}", e))?;
        }

        Ok(())
    }

    fn deadline(&self) -> Duration {
        Duration::from_secs(16 * 60)
    }
}

impl fmt::Display for CompatibilityTest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Compatibility test, phased upgrade to {} in batches of 1, {}, {}",
            self.updated_image_tag,
            self.first_batch.len(),
            self.second_batch.len()
        )
    }
}
