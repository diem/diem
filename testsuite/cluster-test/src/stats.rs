// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::prometheus::Prometheus;
use anyhow::{format_err, Result};
use std::time::Duration;

pub fn avg_tps(prometheus: &Prometheus, start: Duration, end: Duration) -> Result<f64> {
    prometheus
        .query_range_avg(
            "irate(libra_consensus_last_committed_version[1m])".to_string(),
            &start,
            &end,
            10, /* step */
        )
        .map_err(|e| format_err!("No tps data: {}", e))
}

pub fn avg_txns_per_block(prometheus: &Prometheus, start: Duration, end: Duration) -> Result<f64> {
    prometheus
        .query_range_avg(
            "irate(libra_consensus_num_txns_per_block_sum[1m])/irate(libra_consensus_num_txns_per_block_count[1m])".to_string(),
            &start,
            &end,
            10, /* step */
        )
        .map_err(|e| format_err!("No txns_per_block data: {}", e))
}

pub fn avg_latency(prometheus: &Prometheus, start: Duration, end: Duration) -> Result<f64> {
    prometheus.query_range_avg(
        "irate(mempool_duration_sum{op='e2e.latency'}[1m])/irate(mempool_duration_count{op='e2e.latency'}[1m])"
            .to_string(),
        &start,
        &end,
        10 /* step */,
    ).map(|x| x * 1000.).map_err(|e| format_err!("No latency data: {}", e))
}

pub fn txn_stats(prometheus: &Prometheus, start: Duration, end: Duration) -> Result<(f64, f64)> {
    Ok((
        avg_tps(prometheus, start, end)?,
        avg_latency(prometheus, start, end)?,
    ))
}
