// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use crate::instance::{
    FullnodeConfig, Instance, InstanceConfig,
    InstanceConfig::{Fullnode, Validator, Vault, LSR},
    LSRConfig, ValidatorConfig, VaultConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{
    future::{join, try_join_all},
    try_join,
};

#[async_trait]
pub trait ClusterSwarm {
    async fn remove_all_network_effects(&self) -> Result<()>;

    /// Spawns a new instance.
    async fn spawn_new_instance(&self, instance_config: InstanceConfig) -> Result<Instance>;

    /// Creates a set of validators with the given `image_tag`
    async fn spawn_validator_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        enable_lsr: bool,
        lsr_backend: &str,
        image_tag: &str,
        config_overrides: Vec<String>,
    ) -> Result<Vec<Instance>> {
        let mut lsrs = vec![];
        if enable_lsr {
            if lsr_backend == "vault" {
                let mut vault_instances: Vec<_> = (0..num_validators)
                    .map(|i| {
                        let vault_config = VaultConfig { index: i };
                        self.spawn_new_instance(Vault(vault_config))
                    })
                    .collect();
                lsrs.append(&mut vault_instances);
            }
            let mut lsr_instances: Vec<_> = (0..num_validators)
                .map(|i| {
                    let lsr_config = LSRConfig {
                        index: i,
                        num_validators,
                        image_tag: image_tag.to_string(),
                        lsr_backend: lsr_backend.to_string(),
                    };
                    self.spawn_new_instance(LSR(lsr_config))
                })
                .collect();
            lsrs.append(&mut lsr_instances);
        }
        let validators = (0..num_validators).map(|i| {
            let validator_config = ValidatorConfig {
                index: i,
                num_validators,
                num_fullnodes: num_fullnodes_per_validator,
                enable_lsr,
                image_tag: image_tag.to_string(),
                config_overrides: config_overrides.clone(),
            };
            self.spawn_new_instance(Validator(validator_config))
        });
        let (lsrs, validators) = join(try_join_all(lsrs), try_join_all(validators)).await;
        lsrs?;
        validators
    }

    /// Creates a set of fullnodes with the given `image_tag`
    async fn spawn_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        config_overrides: Vec<String>,
    ) -> Result<Vec<Instance>> {
        let fullnodes = (0..num_validators).flat_map(move |validator_index| {
            let config_overrides = config_overrides.clone();
            (0..num_fullnodes_per_validator).map(move |fullnode_index| {
                let fullnode_config = FullnodeConfig {
                    fullnode_index,
                    num_fullnodes_per_validator,
                    validator_index,
                    num_validators,
                    image_tag: image_tag.to_string(),
                    config_overrides: config_overrides.clone(),
                };
                self.spawn_new_instance(Fullnode(fullnode_config))
            })
        });
        let fullnodes = try_join_all(fullnodes).await?;
        Ok(fullnodes)
    }

    /// Creates a set of validators and fullnodes with the given parameters
    async fn spawn_validator_and_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        enable_lsr: bool,
        lsr_backend: &str,
        image_tag: &str,
    ) -> Result<(Vec<Instance>, Vec<Instance>)> {
        try_join!(
            self.spawn_validator_set(
                num_validators,
                num_fullnodes_per_validator,
                enable_lsr,
                lsr_backend,
                image_tag,
                vec![],
            ),
            self.spawn_fullnode_set(
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                vec![],
            ),
        )
    }

    /// Deletes all validators and fullnodes in this cluster
    async fn delete_all(&self) -> Result<()>;

    async fn get_grafana_baseurl(&self) -> Result<String>;
}
