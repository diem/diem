// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use crate::instance::{Instance, InstanceConfig};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClusterSwarm: Send + Sync {
    /// Spawns a new instance.
    async fn spawn_new_instance(
        &self,
        instance_config: InstanceConfig,
        delete_data: bool,
    ) -> Result<Instance>;

    async fn get_grafana_baseurl(&self) -> Result<String>;
}
