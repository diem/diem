// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    cluster::Cluster,
    health::{HealthCheck, HealthCheckContext},
    prometheus::Prometheus,
    util::unix_timestamp_now,
};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{collections::HashMap, time::Duration};

static VAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"val-(\d+)").unwrap());
static FULLNODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fn-(\d+)").unwrap());

pub struct FullNodeHealthCheck {
    cluster: Cluster,
    prometheus: Prometheus,
}

impl FullNodeHealthCheck {
    pub fn new(cluster: Cluster, prometheus: Prometheus) -> FullNodeHealthCheck {
        Self {
            cluster,
            prometheus,
        }
    }

    pub fn state_sync_committed_version(
        &self,
        start: Duration,
        end: Duration,
    ) -> Result<HashMap<String, f64>> {
        libra_retrier::retry(libra_retrier::fixed_retry_strategy(1_000, 5), || {
            self.prometheus
                .query_range_max(
                    "libra_state_sync_committed_version{{}}".to_string(),
                    &start,
                    &end,
                    10, /* step */
                )
                .map_err(|e| format_err!("No sync committed version data: {}", e))
        })
    }
}

pub fn convert_index(input: HashMap<String, f64>) -> HashMap<String, Vec<i64>> {
    let mut res = HashMap::new();
    for (peer_name, version) in input {
        if let Some(cap) = VAL_REGEX.captures(peer_name.as_ref()) {
            // this is index of validator
            if let Some(cap) = cap.get(1) {
                let index = cap.as_str();
                // we record version of validator to be a negative number
                // to calculate diff with all full nodes are associated
                res.entry(index.to_string())
                    .or_insert_with(Vec::new)
                    .push(-version as i64);
            }
        } else if let Some(cap) = FULLNODE_REGEX.captures(peer_name.as_ref()) {
            // this is index of fullnode
            if let Some(cap) = cap.get(1) {
                let index = cap.as_str();
                res.entry(index.to_string())
                    .or_insert_with(Vec::new)
                    .push(version as i64);
            }
        } else {
            panic!(
                "Failed to parse peer name {} into validator_index",
                peer_name
            )
        }
    }
    res
}

impl HealthCheck for FullNodeHealthCheck {
    fn verify(&mut self, ctx: &mut HealthCheckContext) {
        let validator = self.cluster.validator_instances();
        let buffer = Duration::from_secs(30);
        let timestamp = unix_timestamp_now() - buffer;
        let commit_version =
            match FullNodeHealthCheck::state_sync_committed_version(&self, timestamp, timestamp) {
                Err(_e) => return,
                Ok(s) => convert_index(s),
            };
        for (index, version) in &commit_version {
            if version.len() < 2 {
                panic!("Missed record to compare fullnode and validator version difference, record length: {}",
                       version.len());
            }
            let mut version = version.clone();
            version.sort();
            // because validator version is the only negative number, it always be sorted at the first slot
            for i in 1..version.len() {
                // we define tolerance of version diff is 20000
                if (version[i] + version[0]) > 20000_i64 || (version[i] + version[0]) < -20000_i64 {
                    ctx.report_failure(
                        validator[index.parse::<usize>().unwrap()]
                            .peer_name()
                            .clone(),
                        format!(
                            "state sync committed version of fullnode: {} is behind validator: {}",
                            version[i], -version[0],
                        ),
                    );
                }
            }
        }
    }

    fn name(&self) -> &'static str {
        "fullnode_check"
    }
}
