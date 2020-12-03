// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::prometheus::Prometheus;
use anyhow::format_err;
use std::time::Duration;

pub struct PrometheusRangeView<'a> {
    prometheus: &'a Prometheus,
    start: Duration,
    end: Duration,
}

impl<'a> PrometheusRangeView<'a> {
    pub fn new(prometheus: &'a Prometheus, start: Duration, end: Duration) -> Self {
        Self {
            prometheus,
            start,
            end,
        }
    }

    pub fn avg_txns_per_block(&self) -> Option<f64> {
        self.query_avg(
            "txn_per_block",
            "irate(diem_consensus_num_txns_per_block_sum[1m])/irate(diem_consensus_num_txns_per_block_count[1m])".to_string(),
        )
    }

    pub fn avg_backup_bytes_per_second(&self) -> Option<f64> {
        self.query_avg(
            "backup_bytes_per_second",
            "sum(irate(diem_backup_service_sent_bytes[1m])) by(peer_id)".to_string(),
        )
    }
}

impl<'a> PrometheusRangeView<'a> {
    const STEP: u64 = 10;

    fn query_avg(&self, name: &str, query: String) -> Option<f64> {
        self.prometheus
            .query_range_avg(query, &self.start, &self.end, Self::STEP)
            .map_err(|e| format_err!("No {} data: {}", name, e))
            .ok()
    }
}
