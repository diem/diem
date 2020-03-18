// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use crate::instance::{
    FullnodeConfig, Instance, InstanceConfig,
    InstanceConfig::{Fullnode, Validator},
    ValidatorConfig,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::{future::try_join_all, try_join};

#[async_trait]
pub trait ClusterSwarm {
    async fn remove_all_network_effects(&self) -> Result<()>;

    /// Runs the given command in a container against the given Instance
    async fn run(
        &self,
        instance: &Instance,
        docker_image: &str,
        command: String,
        job_name: &str,
    ) -> Result<()>;

    async fn validator_instances(&self) -> Vec<Instance>;

    async fn fullnode_instances(&self) -> Vec<Instance>;

    /// Inserts an into the ClusterSwarm if it doesn't exist. If it
    /// exists, then updates the instance.
    async fn upsert_node(&self, instance_config: InstanceConfig, delete_data: bool) -> Result<()>;

    /// Deletes a node from the ClusterSwarm
    async fn delete_node(&self, instance_config: InstanceConfig) -> Result<()>;

    /// Creates a set of validators with the given `image_tag`
    async fn create_validator_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        config_overrides: Vec<String>,
        delete_data: bool,
    ) -> Result<()> {
        let validators = (0..num_validators).map(|i| {
            let validator_config = ValidatorConfig {
                index: i,
                num_validators,
                num_fullnodes: num_fullnodes_per_validator,
                image_tag: image_tag.to_string(),
                config_overrides: config_overrides.clone(),
            };
            self.upsert_node(Validator(validator_config), delete_data)
        });
        try_join_all(validators).await?;
        Ok(())
    }

    /// Creates a set of fullnodes with the given `image_tag`
    async fn create_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        config_overrides: Vec<String>,
        delete_data: bool,
    ) -> Result<()> {
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
                self.upsert_node(Fullnode(fullnode_config), delete_data)
            })
        });
        try_join_all(fullnodes).await?;
        Ok(())
    }

    /// Creates a set of validators and fullnodes with the given parameters
    async fn create_validator_and_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<((), ())> {
        try_join!(
            self.create_validator_set(
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                vec![],
                delete_data
            ),
            self.create_fullnode_set(
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                vec![],
                delete_data
            ),
        )
    }

    /// Deletes all validators and fullnodes in this cluster
    async fn delete_all(&self) -> Result<()>;

    async fn get_grafana_baseurl(&self) -> Result<String>;
}
