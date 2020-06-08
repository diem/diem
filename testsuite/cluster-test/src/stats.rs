// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::prometheus::Prometheus;
use anyhow::{format_err, Result};
use std::time::Duration;

pub fn avg_txns_per_block(prometheus: &Prometheus, start: Duration, end: Duration) -> Result<f64> {
    libra_retrier::retry(libra_retrier::fixed_retry_strategy(1_000, 5), || {
        prometheus
            .query_range_avg(
                "irate(libra_consensus_num_txns_per_block_sum[1m])/irate(libra_consensus_num_txns_per_block_count[1m])".to_string(),
                &start,
                &end,
                10, /* step */
            )
            .map_err(|e| format_err!("No txns_per_block data: {}", e))
    })
}
