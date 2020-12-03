// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    health::{HealthCheck, HealthCheckContext},
    instance::Instance,
};
use async_trait::async_trait;
use futures::future::join_all;
use once_cell::sync::Lazy;
use std::{collections::HashMap, env};

pub static THRESHOLD: Lazy<i64> = Lazy::new(|| {
    if let Ok(v) = env::var("FULL_NODE_HEALTH_THRESHOLD") {
        v.parse()
            .expect("Failed to parse FULL_NODE_HEALTH_THRESHOLD")
    } else {
        15000_i64
    }
});

pub struct FullNodeHealthCheck {
    cluster: Cluster,
}

impl FullNodeHealthCheck {
    pub fn new(cluster: Cluster) -> FullNodeHealthCheck {
        Self { cluster }
    }
}

async fn get_version(instance: &Instance) -> (&Instance, i64) {
    let res = instance
        .debug_interface_client()
        .get_node_metric("diem_state_sync_version{type=committed}")
        .await;
    let content = match res {
        Ok(res) => res.unwrap_or_default(),
        _ => 0i64,
    };
    (instance, content)
}

#[async_trait]
impl HealthCheck for FullNodeHealthCheck {
    async fn verify(&mut self, ctx: &mut HealthCheckContext) {
        let validators = self.cluster.validator_instances();
        let fullnodes = self.cluster.fullnode_instances();

        let futures = validators.iter().map(get_version);
        let val_latest_versions = join_all(futures).await;
        let val_latest_versions: HashMap<_, _> = val_latest_versions
            .into_iter()
            .map(|(instance, version)| (instance.validator_group().index, version))
            .collect();

        let futures = fullnodes.iter().map(get_version);
        let fullnode_latest_versions = join_all(futures).await;

        for (fullnode, fullnode_version) in fullnode_latest_versions {
            let index = fullnode.validator_group().index;
            let val_version = val_latest_versions.get(&index).unwrap();
            if val_version - fullnode_version > *THRESHOLD {
                ctx.report_failure(
                    format!("val-{}", index),
                    format!(
                        "fullnode {} state sync committed version: {} is behind validator: {}",
                        fullnode.peer_name(),
                        fullnode_version,
                        val_version,
                    ),
                );
            }
        }
    }

    fn name(&self) -> &'static str {
        "fullnode_check"
    }
}
