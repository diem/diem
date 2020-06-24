// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    aws,
    cluster::Cluster,
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
    instance::{
        ApplicationConfig::{Fullnode, Validator, Vault, LSR},
        FullnodeConfig, Instance, InstanceConfig, LSRConfig, ValidatorConfig, VaultConfig,
    },
};
use anyhow::{format_err, Result};
use futures::{
    future::{join, try_join_all},
    try_join,
};
use libra_logger::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ClusterBuilderParams {
    #[structopt(long, default_value = "1")]
    pub fullnodes_per_validator: u32,
    #[structopt(long, use_delimiter = true, default_value = "")]
    pub cfg: Vec<String>,
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
        let (validators, fullnodes) = self
            .spawn_validator_and_fullnode_set(
                params.num_validators,
                params.fullnodes_per_validator,
                params.enable_lsr,
                &params.lsr_backend,
                current_tag,
                params.cfg.as_slice(),
                true,
            )
            .await
            .map_err(|e| format_err!("Failed to spawn_validator_and_fullnode_set: {}", e))?;
        let cluster = Cluster::new(validators, fullnodes);

        info!(
            "Deployed {} validators and {} fns",
            cluster.validator_instances().len(),
            cluster.fullnode_instances().len(),
        );
        Ok(cluster)
    }

    /// Creates a set of validators with the given `image_tag`
    pub async fn spawn_validator_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        enable_lsr: bool,
        lsr_backend: &str,
        image_tag: &str,
        config_overrides: &[String],
        delete_data: bool,
    ) -> Result<Vec<Instance>> {
        let mut lsrs = vec![];
        if enable_lsr {
            if lsr_backend == "vault" {
                let mut vault_instances: Vec<_> = (0..num_validators)
                    .map(|i| {
                        let vault_config = VaultConfig {};
                        self.cluster_swarm.spawn_new_instance(
                            InstanceConfig {
                                validator_group: i,
                                application_config: Vault(vault_config),
                            },
                            delete_data,
                        )
                    })
                    .collect();
                lsrs.append(&mut vault_instances);
            }
            let mut lsr_instances: Vec<_> = (0..num_validators)
                .map(|i| {
                    let lsr_config = LSRConfig {
                        num_validators,
                        image_tag: image_tag.to_string(),
                        lsr_backend: lsr_backend.to_string(),
                    };
                    self.cluster_swarm.spawn_new_instance(
                        InstanceConfig {
                            validator_group: i,
                            application_config: LSR(lsr_config),
                        },
                        delete_data,
                    )
                })
                .collect();
            lsrs.append(&mut lsr_instances);
        }
        let validators = (0..num_validators).map(|i| {
            let validator_config = ValidatorConfig {
                num_validators,
                num_fullnodes: num_fullnodes_per_validator,
                enable_lsr,
                image_tag: image_tag.to_string(),
                config_overrides: config_overrides.to_vec(),
            };
            self.cluster_swarm.spawn_new_instance(
                InstanceConfig {
                    validator_group: i,
                    application_config: Validator(validator_config),
                },
                delete_data,
            )
        });
        let (lsrs, validators) = join(try_join_all(lsrs), try_join_all(validators)).await;
        lsrs?;
        validators
    }

    /// Creates a set of fullnodes with the given `image_tag`
    pub async fn spawn_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        config_overrides: &[String],
        delete_data: bool,
    ) -> Result<Vec<Instance>> {
        let fullnodes = (0..num_validators).flat_map(move |validator_index| {
            (0..num_fullnodes_per_validator).map(move |fullnode_index| {
                let fullnode_config = FullnodeConfig {
                    fullnode_index,
                    num_fullnodes_per_validator,
                    num_validators,
                    image_tag: image_tag.to_string(),
                    config_overrides: config_overrides.to_vec(),
                };
                self.cluster_swarm.spawn_new_instance(
                    InstanceConfig {
                        validator_group: validator_index,
                        application_config: Fullnode(fullnode_config),
                    },
                    delete_data,
                )
            })
        });
        let fullnodes = try_join_all(fullnodes).await?;
        Ok(fullnodes)
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
        delete_data: bool,
    ) -> Result<(Vec<Instance>, Vec<Instance>)> {
        try_join!(
            self.spawn_validator_set(
                num_validators,
                num_fullnodes_per_validator,
                enable_lsr,
                lsr_backend,
                image_tag,
                config_overrides,
                delete_data,
            ),
            self.spawn_fullnode_set(
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                config_overrides,
                delete_data,
            ),
        )
    }
}
