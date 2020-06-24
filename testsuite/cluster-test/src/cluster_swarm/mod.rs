// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use crate::instance::{
    ApplicationConfig::{Fullnode, Validator, Vault, LSR},
    FullnodeConfig, Instance, InstanceConfig, LSRConfig, ValidatorConfig, VaultConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{
    future::{join, try_join_all},
    try_join,
};

#[async_trait]
pub trait ClusterSwarm {
    /// Spawns a new instance.
    async fn spawn_new_instance(
        &self,
        instance_config: InstanceConfig,
        delete_data: bool,
    ) -> Result<Instance>;

    /// Creates a set of validators with the given `image_tag`
    async fn spawn_validator_set(
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
                        self.spawn_new_instance(
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
                    self.spawn_new_instance(
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
            self.spawn_new_instance(
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
    async fn spawn_fullnode_set(
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
                self.spawn_new_instance(
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
    async fn spawn_validator_and_fullnode_set(
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

    async fn get_grafana_baseurl(&self) -> Result<String>;
}
