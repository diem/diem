// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    aws,
    cluster::Cluster,
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    instance::{
        fullnode_pod_name, lsr_pod_name, validator_pod_name, vault_pod_name,
        ApplicationConfig::{Fullnode, Validator, Vault, LSR},
        FullnodeConfig, Instance, InstanceConfig, LSRConfig, ValidatorConfig, ValidatorGroup,
        VaultConfig,
    },
};
use anyhow::{format_err, Result};
use futures::{future::try_join_all, try_join};
use libra_logger::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ClusterBuilderParams {
    #[structopt(long, default_value = "1")]
    pub fullnodes_per_validator: u32,
    #[structopt(long, use_delimiter = true, default_value = "")]
    cfg: Vec<String>,
    #[structopt(long, parse(try_from_str), default_value = "30")]
    pub num_validators: u32,
    #[structopt(long)]
    pub enable_lsr: bool,
    #[structopt(
        long,
        help = "Backend used by lsr. Possible Values are in-memory, on-disk, vault",
        default_value = "vault"
    )]
    pub lsr_backend: String,
}

impl ClusterBuilderParams {
    pub fn cfg_overrides(&self) -> Vec<String> {
        // Default overrides
        let mut overrides = vec!["prune_window=50000".to_string()];

        // overrides from the command line
        overrides.extend(self.cfg.iter().cloned());

        overrides
    }
}

pub struct ClusterBuilder {
    current_tag: String,
    cluster_swarm: ClusterSwarmKube,
}

impl ClusterBuilder {
    pub fn new(current_tag: String, cluster_swarm: ClusterSwarmKube) -> Self {
        Self {
            current_tag,
            cluster_swarm,
        }
    }

    pub async fn setup_cluster(&self, params: &ClusterBuilderParams) -> Result<Cluster> {
        self.cluster_swarm
            .cleanup()
            .await
            .map_err(|e| format_err!("cleanup on startup failed: {}", e))?;
        let current_tag = &self.current_tag;
        info!(
            "Deploying with {} tag for validators and fullnodes",
            current_tag
        );
        let asg_name = format!(
            "{}-k8s-testnet-validators",
            self.cluster_swarm
                .get_workspace()
                .await
                .expect("Failed to get workspace")
        );
        let mut instance_count =
            params.num_validators + (params.fullnodes_per_validator * params.num_validators);
        if params.enable_lsr {
            if params.lsr_backend == "vault" {
                instance_count += params.num_validators * 2;
            } else {
                instance_count += params.num_validators;
            }
        }
        // First scale down to zero instances and wait for it to complete so that we don't schedule pods on
        // instances which are going into termination state
        aws::set_asg_size(0, 0.0, &asg_name, true, true)
            .await
            .map_err(|err| format_err!("{} scale down failed: {}", asg_name, err))?;
        // Then scale up and bring up new instances
        aws::set_asg_size(instance_count as i64, 5.0, &asg_name, true, false)
            .await
            .map_err(|err| format_err!("{} scale up failed: {}", asg_name, err))?;
        let (validators, lsrs, vaults, fullnodes) = self
            .spawn_validator_and_fullnode_set(
                params.num_validators,
                params.fullnodes_per_validator,
                params.enable_lsr,
                &params.lsr_backend,
                current_tag,
                &params.cfg_overrides(),
            )
            .await
            .map_err(|e| format_err!("Failed to spawn_validator_and_fullnode_set: {}", e))?;
        let cluster = Cluster::new(validators, fullnodes, lsrs, vaults);

        info!(
            "Deployed {} validators and {} fns",
            cluster.validator_instances().len(),
            cluster.fullnode_instances().len(),
        );
        Ok(cluster)
    }

    /// Creates a set of validators and fullnodes with the given parameters
    pub async fn spawn_validator_and_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        enable_lsr: bool,
        lsr_backend: &str,
        image_tag: &str,
        config_overrides: &[String],
    ) -> Result<(Vec<Instance>, Vec<Instance>, Vec<Instance>, Vec<Instance>)> {
        let vault_nodes;
        let mut lsrs_nodes = vec![];
        let mut vaults = vec![];
        let mut lsrs = vec![];

        if enable_lsr {
            if lsr_backend == "vault" {
                vault_nodes = try_join_all((0..num_validators).map(|i| async move {
                    let pod_name = vault_pod_name(i);
                    self.cluster_swarm.allocate_node(&pod_name).await
                }))
                .await?;
                let mut vault_instances: Vec<_> = vault_nodes
                    .iter()
                    .enumerate()
                    .map(|(i, node)| async move {
                        let vault_config = VaultConfig {};
                        self.cluster_swarm.clean_data(&node.name).await?;
                        self.cluster_swarm
                            .spawn_new_instance(InstanceConfig {
                                validator_group: ValidatorGroup::new_for_index(i as u32),
                                application_config: Vault(vault_config),
                            })
                            .await
                    })
                    .collect();
                vaults.append(&mut vault_instances);
            }

            lsrs_nodes = try_join_all((0..num_validators).map(|i| async move {
                let pod_name = lsr_pod_name(i);
                self.cluster_swarm.allocate_node(&pod_name).await
            }))
            .await?;
            let mut lsr_instances: Vec<_> = lsrs_nodes
                .iter()
                .enumerate()
                .map(|(i, node)| async move {
                    let lsr_config = LSRConfig {
                        num_validators,
                        image_tag: image_tag.to_string(),
                        lsr_backend: lsr_backend.to_string(),
                    };
                    self.cluster_swarm.clean_data(&node.name).await?;
                    self.cluster_swarm
                        .spawn_new_instance(InstanceConfig {
                            validator_group: ValidatorGroup::new_for_index(i as u32),
                            application_config: LSR(lsr_config),
                        })
                        .await
                })
                .collect();
            lsrs.append(&mut lsr_instances);
        }

        let validator_nodes = try_join_all((0..num_validators).map(|i| async move {
            let pod_name = validator_pod_name(i);
            self.cluster_swarm.allocate_node(&pod_name).await
        }))
        .await?;
        let validators = (0..num_validators).map(|i| {
            let validator_nodes = &validator_nodes;
            let lsrs_nodes = &lsrs_nodes;
            async move {
                let seed_peer_ip = validator_nodes[0].internal_ip.clone();
                let safety_rules_addr = if enable_lsr {
                    Some(lsrs_nodes[i as usize].internal_ip.clone())
                } else {
                    None
                };
                let validator_config = ValidatorConfig {
                    num_validators,
                    num_fullnodes: num_fullnodes_per_validator,
                    enable_lsr,
                    image_tag: image_tag.to_string(),
                    config_overrides: config_overrides.to_vec(),
                    seed_peer_ip,
                    safety_rules_addr,
                };
                self.cluster_swarm
                    .clean_data(&validator_nodes[i as usize].name)
                    .await?;
                self.cluster_swarm
                    .spawn_new_instance(InstanceConfig {
                        validator_group: ValidatorGroup::new_for_index(i),
                        application_config: Validator(validator_config),
                    })
                    .await
            }
        });

        let fullnode_nodes = try_join_all((0..num_validators).flat_map(move |validator_index| {
            (0..num_fullnodes_per_validator).map(move |fullnode_index| async move {
                let pod_name = fullnode_pod_name(validator_index, fullnode_index);
                self.cluster_swarm.allocate_node(&pod_name).await
            })
        }))
        .await?;
        let fullnodes = (0..num_validators).flat_map(|validator_index| {
            let fullnode_nodes = &fullnode_nodes;
            let validator_nodes = &validator_nodes;
            (0..num_fullnodes_per_validator).map(move |fullnode_index| async move {
                let seed_peer_ip = validator_nodes[validator_index as usize]
                    .internal_ip
                    .clone();
                let fullnode_config = FullnodeConfig {
                    fullnode_index,
                    num_fullnodes_per_validator,
                    num_validators,
                    image_tag: image_tag.to_string(),
                    config_overrides: config_overrides.to_vec(),
                    seed_peer_ip,
                };
                self.cluster_swarm
                    .clean_data(
                        &fullnode_nodes[(validator_index * num_fullnodes_per_validator
                            + fullnode_index) as usize]
                            .name,
                    )
                    .await?;
                self.cluster_swarm
                    .spawn_new_instance(InstanceConfig {
                        validator_group: ValidatorGroup::new_for_index(validator_index),
                        application_config: Fullnode(fullnode_config),
                    })
                    .await
            })
        });

        try_join!(
            try_join_all(validators),
            try_join_all(lsrs),
            try_join_all(vaults),
            try_join_all(fullnodes),
        )
    }
}
