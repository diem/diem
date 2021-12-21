// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod cluster_swarm_kube;

use crate::instance::{Instance, InstanceConfig};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClusterSwarm: Send + Sync {
    /// Spawns a new instance.
    async fn spawn_new_instance(&self, instance_config: InstanceConfig) -> Result<Instance>;

    /// If deleting /opt/diem/data/* is required, call clean_date before calling
    /// spawn_new_instance.
    async fn clean_data(&self, node: &str) -> Result<()>;

    async fn get_node_name(&self, pod_name: &str) -> Result<String>;

    async fn get_grafana_baseurl(&self) -> Result<String>;

    async fn put_file(&self, node: &str, pod_name: &str, path: &str, content: &[u8]) -> Result<()>;
}
