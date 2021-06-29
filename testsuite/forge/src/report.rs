// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TxnStats;
use std::{fmt, time::Duration};

#[derive(Default, Debug)]
pub struct TestReport {
    metrics: Vec<ReportedMetric>,
    text: String,
}

#[derive(Debug)]
pub struct ReportedMetric {
    pub test_name: String,
    pub metric: String,
    pub value: f64,
}

impl TestReport {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn report_metric<E: ToString, M: ToString>(&mut self, test: E, metric: M, value: f64) {
        self.metrics.push(ReportedMetric {
            test_name: test.to_string(),
            metric: metric.to_string(),
            value,
        });
    }

    pub fn report_text(&mut self, text: String) {
        if !self.text.is_empty() {
            self.text.push('\n');
        }
        self.text.push_str(&text);
    }

    pub fn report_txn_stats(&mut self, test_name: String, stats: TxnStats, window: Duration) {
        let submitted_txn = stats.submitted;
        let expired_txn = stats.expired;
        let avg_tps = stats.committed / window.as_secs();
        let avg_latency_client = if stats.committed == 0 {
            0u64
        } else {
            stats.latency / stats.committed
        };
        let p99_latency = stats.latency_buckets.percentile(99, 100);
        self.report_metric(test_name.clone(), "submitted_txn", submitted_txn as f64);
        self.report_metric(test_name.clone(), "expired_txn", expired_txn as f64);
        self.report_metric(test_name.clone(), "avg_tps", avg_tps as f64);
        self.report_metric(test_name.clone(), "avg_latency", avg_latency_client as f64);
        self.report_metric(test_name.clone(), "p99_latency", p99_latency as f64);
        let expired_text = if expired_txn == 0 {
            "no expired txns".to_string()
        } else {
            format!("(!) expired {} out of {} txns", expired_txn, submitted_txn)
        };
        self.report_text(format!(
            "{} : {:.0} TPS, {:.1} ms latency, {:.1} ms p99 latency,{}",
            test_name, avg_tps, avg_latency_client, p99_latency, expired_text
        ));
    }

    pub fn print_report(&self) {
        println!("Test Statistics: {}", self,);
    }
}

impl fmt::Display for TestReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
