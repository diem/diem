// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use anyhow::Result;
use async_trait::async_trait;
use futures::{future::try_join_all, try_join};

#[async_trait]
pub trait ClusterSwarm {
    /// Inserts a validator into the ClusterSwarm if it doesn't exist. If it
    /// exists, then updates the validator.
    async fn upsert_validator(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()>;

    /// Deletes a validator from the ClusterSwarm
    async fn delete_validator(&self, index: u32) -> Result<()>;

    /// Creates a set of validators with the given `image_tag`
    async fn create_validator_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()> {
        let validators = (0..num_validators).map(|i| {
            self.upsert_validator(
                i,
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                delete_data,
            )
        });
        try_join_all(validators).await?;
        Ok(())
    }

    /// Deletes a set of validators
    async fn delete_validator_set(&self, num_validators: u32) -> Result<()> {
        let validators = (0..num_validators).map(|i| self.delete_validator(i));
        try_join_all(validators).await?;
        Ok(())
    }

    /// Inserts a fullnode into the ClusterSwarm if it doesn't exist. If it
    /// exists, then updates the fullnode.
    async fn upsert_fullnode(
        &self,
        fullnode_index: u32,
        num_fullnodes_per_validator: u32,
        validator_index: u32,
        num_validators: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()>;

    /// Deletes a fullnode from the ClusterSwarm
    async fn delete_fullnode(&self, fullnode_index: u32, validator_index: u32) -> Result<()>;

    /// Creates a set of fullnodes with the given `image_tag`
    async fn create_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()> {
        let fullnodes = (0..num_validators).flat_map(move |validator_index| {
            (0..num_fullnodes_per_validator).map(move |fullnode_index| {
                self.upsert_fullnode(
                    fullnode_index,
                    num_fullnodes_per_validator,
                    validator_index,
                    num_validators,
                    image_tag,
                    delete_data,
                )
            })
        });
        try_join_all(fullnodes).await?;
        Ok(())
    }

    /// Deletes a set of fullnodes
    async fn delete_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
    ) -> Result<()> {
        let fullnodes = (0..num_validators).flat_map(move |validator_index| {
            (0..num_fullnodes_per_validator)
                .map(move |fullnode_index| self.delete_fullnode(fullnode_index, validator_index))
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
                delete_data
            ),
            self.create_fullnode_set(
                num_validators,
                num_fullnodes_per_validator,
                image_tag,
                delete_data
            ),
        )
    }

    /// Deletes a set of validators and fullnodes with the given parameters
    async fn delete_validator_and_fullnode_set(
        &self,
        num_validators: u32,
        num_fullnodes_per_validator: u32,
    ) -> Result<((), ())> {
        try_join!(
            self.delete_validator_set(num_validators),
            self.delete_fullnode_set(num_validators, num_fullnodes_per_validator),
        )
    }

    /// Deletes all validators and fullnodes in this cluster
    async fn delete_all(&self) -> Result<()>;
}
