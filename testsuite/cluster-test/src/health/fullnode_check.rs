// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::prometheus::Prometheus;
use crate::util::unix_timestamp_now;
use crate::{
    cluster::Cluster,
    health::{HealthCheck, HealthCheckContext, ValidatorEvent},
};
use anyhow::{format_err, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration;

const APP_LIBRA_VALIDATOR: &str = "libra-validator";
const APP_LIBRA_FULLNODE: &str = "libra-fullnode";
static VAL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"val-(\d+)").unwrap());
static FULLNODE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"fn-(\d+)").unwrap());

pub struct FullNodeHealthCheck {
    cluster: Cluster,
    prometheus: Option<Prometheus>,
}

impl FullNodeHealthCheck {
    pub fn new(cluster: Cluster, prometheus: Option<Prometheus>) -> FullNodeHealthCheck {
        Self {
            cluster,
            prometheus,
        }
    }

    pub fn state_sync_committed_version(
        prometheus: &Prometheus,
        start: Duration,
        end: Duration,
        app: &str,
    ) -> Result<HashMap<String, f64>> {
        libra_retrier::retry(libra_retrier::fixed_retry_strategy(1_000, 5), || {
            prometheus
                .query_range_max(
                    format!("libra_state_sync_committed_version{{app=\"{}\"}}", app),
                    &start,
                    &end,
                    10, /* step */
                )
                .map_err(|e| format_err!("No sync committed version data: {}", e))
        })
    }
}

pub fn node_index(peer_name: String) -> String {
    if let Some(cap) = VAL_REGEX.captures(peer_name.as_ref()) {
        if let Some(cap) = cap.get(1) {
            return cap.as_str().to_string();
        }
    }
    if let Some(cap) = FULLNODE_REGEX.captures(peer_name.as_ref()) {
        if let Some(cap) = cap.get(1) {
            return cap.as_str().to_string();
        }
    }
    panic!(
        "Failed to parse peer name {} into validator_index",
        peer_name
    )
}

pub fn convert_index(input: HashMap<String, f64>) -> HashMap<String, f64> {
    let mut res = HashMap::new();
    for (peer_name, version) in input {
        let peer_name = node_index(peer_name);
        res.insert(node_index(peer_name), version);
    }
    res
}

impl HealthCheck for FullNodeHealthCheck {
    fn on_event(&mut self, _event: &ValidatorEvent, _ctx: &mut HealthCheckContext) {
        return;
    }

    fn verify(&mut self, ctx: &mut HealthCheckContext) {
        let fullnode = self.cluster.fullnode_instances();
        let validator = self.cluster.validator_instances();
        let prometheus = match self.prometheus.as_ref() {
            Some(p) => p,
            None => return,
        };
        assert_eq!(
            fullnode.len(),
            validator.len(),
            "fullnode and validator must have same size, fullnode: {}, validator: {}",
            fullnode.len(),
            validator.len()
        );

        let buffer = Duration::from_secs(30);
        let timestamp = unix_timestamp_now() - buffer;
        let res_fullnode = match FullNodeHealthCheck::state_sync_committed_version(
            &prometheus,
            timestamp,
            timestamp,
            APP_LIBRA_FULLNODE,
        ) {
            Err(_e) => return,
            Ok(s) => convert_index(s),
        };
        let res_validator = match FullNodeHealthCheck::state_sync_committed_version(
            &prometheus,
            timestamp,
            timestamp,
            APP_LIBRA_VALIDATOR,
        ) {
            Err(_e) => return,
            Ok(s) => convert_index(s),
        };
        for (node_index, version) in &res_fullnode {
            if res_validator.contains_key(node_index) {
                let validator_version = res_validator[node_index];
                // we define tolerance of version diff is 2000
                if (version - validator_version).abs() > 20000 as f64 {
                    ctx.report_failure(
                        validator[node_index.parse::<usize>().unwrap()].clone()
                            .peer_name()
                            .clone(),
                        format!(
                            "state sync committed version of fullnode: {} is behind validator: {}",
                            version,
                            res_validator[node_index],
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
