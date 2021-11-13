// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    cluster_swarm::ClusterSwarm,
    experiments::{Context, Experiment, ExperimentParam},
    instance,
    instance::Instance,
};
use async_trait::async_trait;
use diem_logger::prelude::*;
use diem_sdk::transaction_builder::TransactionFactory;
use forge::TxnEmitter;
use futures::future::try_join_all;
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use std::{
    collections::HashSet,
    env, fmt,
    iter::{once, Iterator},
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tokio::time;

/// Reboot `updated_instance` with newer image tag
pub async fn update_batch_instance(
    cluster_swarm: &dyn ClusterSwarm,
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
                cluster_swarm.spawn_new_instance(newer_config)
            })
            .collect();
        try_join_all(futures).await?;
        info!("Wait for the instance to sync up with peers");
        time::sleep(Duration::from_secs(20)).await;
    }

    info!("Reinstantiate a set of new nodes.");
    let futures: Vec<_> = updated_instance
        .iter()
        .map(|instance| {
            let mut newer_config = instance.instance_config().clone();
            info!("newer config: {:?}", newer_config);
            newer_config.replace_tag(updated_tag.clone()).unwrap();
            cluster_swarm.spawn_new_instance(newer_config)
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
    time::sleep(Duration::from_secs(20)).await;
    Ok(())
}

pub fn get_corresponding_full_nodes<'a>(
    validators: impl Iterator<Item = &'a Instance>,
    cluster: &Cluster,
) -> Vec<Instance> {
    let validator_groups = validators
        .map(|instance| instance.validator_group())
        .collect::<Vec<_>>();
    cluster
        .fullnode_instances()
        .iter()
        .filter(|full_node| validator_groups.contains(&full_node.validator_group()))
        .cloned()
        .collect()
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
    first_full_node: Vec<Instance>,
    first_batch: Vec<Instance>,
    first_batch_lsr: Vec<Instance>,
    first_full_nodes_batch: Vec<Instance>,
    second_batch: Vec<Instance>,
    second_batch_lsr: Vec<Instance>,
    second_full_nodes_batch: Vec<Instance>,
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
        let first_full_node = get_corresponding_full_nodes([first_node.clone()].iter(), cluster);

        let mut first_lsr = vec![];
        let mut first_batch_lsr = vec![];
        let mut second_batch_lsr = vec![];
        if !cluster.lsr_instances().is_empty() {
            first_batch_lsr = cluster.lsr_instances_for_validators(&first_batch);
            second_batch_lsr = cluster.lsr_instances_for_validators(&second_batch);
            first_lsr = cluster.lsr_instances_for_validators(&[first_node.clone()]);
        }

        let first_full_nodes_batch = get_corresponding_full_nodes(first_batch.iter(), cluster);
        let second_full_nodes_batch = get_corresponding_full_nodes(second_batch.iter(), cluster);

        Self::E {
            first_node,
            first_lsr,
            first_full_node,
            first_batch,
            first_batch_lsr,
            first_full_nodes_batch,
            second_batch,
            second_batch_lsr,
            second_full_nodes_batch,
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
        let mut txn_emitter = TxnEmitter::new(
            &mut context.treasury_compliance_account,
            &mut context.designated_dealer_account,
            context
                .cluster
                .random_validator_instance()
                .json_rpc_client(),
            TransactionFactory::new(context.cluster.chain_id),
            StdRng::from_seed(OsRng.gen()),
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
        let all_full_nodes_request = crate::util::emit_job_request_for_instances(
            context.cluster.fullnode_instances().to_vec(),
            context.global_emit_job_request,
            0,
            0,
        );
        txn_emitter
            .emit_txn_for(job_duration, all_full_nodes_request)
            .await
            .map_err(|e| anyhow::format_err!("Failed to generate traffic: {}", e))?;

        let msg = format!(
            "2. First full node {} ==> {}, to validate new full node to old validator node traffic",
            context.current_tag, self.updated_image_tag
        );
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Upgrade Full Node: {}",
            get_instance_list_str(&self.first_full_node)
        );
        update_batch_instance(
            context.cluster_swarm,
            &self.first_full_node,
            &[],
            self.updated_image_tag.clone(),
        )
        .await?;

        // Full node running at n+1, validator running n
        txn_emitter
            .emit_txn_for(
                job_duration,
                crate::util::emit_job_request_for_instances(
                    self.first_full_node.clone(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
            )
            .await
            .map_err(|e| anyhow::format_err!("Storage backwards compat broken: {}", e))?;

        let msg = format!(
            "3. First Validator node {} ==> {}, to validate storage compatibility",
            context.current_tag, self.updated_image_tag
        );
        info!("{}", msg);
        context.report.report_text(msg);
        info!("Upgrading validator: {}", self.first_node);
        let first_node = vec![self.first_node.clone()];
        update_batch_instance(
            context.cluster_swarm,
            &first_node,
            &self.first_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;
        txn_emitter
            .emit_txn_for(
                job_duration,
                crate::util::emit_job_request_for_instances(
                    self.first_full_node.clone(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
            )
            .await
            .map_err(|e| anyhow::format_err!("Storage backwards compat broken: {}", e))?;

        let msg = format!(
            "4. First batch validators ({}) {} ==> {}, to test consensus and traffic between old full nodes and new validator node",
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
            context.cluster_swarm,
            &self.first_batch,
            &self.first_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;

        // Full node running at n, validator running n+1
        txn_emitter
            .emit_txn_for(
                job_duration,
                crate::util::emit_job_request_for_instances(
                    self.first_full_nodes_batch.clone(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
            )
            .await
            .map_err(|e| anyhow::format_err!("Consensus backwards compat broken: {}", e))?;

        let msg = format!(
            "5. First batch full nodes ({}) {} ==> {}",
            self.first_full_nodes_batch.len(),
            context.current_tag,
            self.updated_image_tag
        );
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Upgrading full nodes: {}",
            get_instance_list_str(&self.first_full_nodes_batch)
        );
        update_batch_instance(
            context.cluster_swarm,
            &self.first_full_nodes_batch,
            &[],
            self.updated_image_tag.clone(),
        )
        .await?;

        let msg = format!(
            "6. Second batch validators ({}) {} ==> {}, to upgrade rest of the validators",
            self.second_batch.len(),
            context.current_tag,
            self.updated_image_tag
        );
        info!("{}", msg);
        context.report.report_text(msg);
        info!(
            "Upgrading validators: {}",
            get_instance_list_str(&self.second_batch)
        );
        update_batch_instance(
            context.cluster_swarm,
            &self.second_batch,
            &self.second_batch_lsr,
            self.updated_image_tag.clone(),
        )
        .await?;
        txn_emitter
            .emit_txn_for(
                job_duration,
                crate::util::emit_job_request_for_instances(
                    self.second_batch.clone(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
            )
            .await
            .map_err(|e| {
                anyhow::format_err!("Failed to upgrade rest of validator images: {}", e)
            })?;

        let msg = format!(
            "7. Second batch of full nodes ({}) {} ==> {}, to finish the network upgrade",
            self.second_full_nodes_batch.len(),
            context.current_tag,
            self.updated_image_tag
        );
        info!("{}", msg);
        info!(
            "Upgrading full nodes: {}",
            get_instance_list_str(&self.second_full_nodes_batch)
        );
        context.report.report_text(msg);
        update_batch_instance(
            context.cluster_swarm,
            &self.second_full_nodes_batch,
            &[],
            self.updated_image_tag.clone(),
        )
        .await?;
        txn_emitter
            .emit_txn_for(
                job_duration,
                crate::util::emit_job_request_for_instances(
                    self.second_full_nodes_batch.clone(),
                    context.global_emit_job_request,
                    0,
                    0,
                ),
            )
            .await
            .map_err(|e| anyhow::format_err!("Failed to upgrade full node images: {}", e))?;

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
