// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;
use std::fmt;

#[derive(Serialize)]
pub struct SuiteReport {
    metrics: Vec<ReportedMetric>,
    text: String,
}

#[derive(Serialize)]
pub struct ReportedMetric {
    pub experiment: String,
    pub metric: String,
    pub value: f64,
}

impl SuiteReport {
    pub fn new() -> Self {
        SuiteReport {
            metrics: vec![],
            text: String::new(),
        }
    }

    pub fn report_metric<E: ToString, M: ToString>(
        &mut self,
        experiment: E,
        metric: M,
        value: f64,
    ) {
        self.metrics.push(ReportedMetric {
            experiment: experiment.to_string(),
            metric: metric.to_string(),
            value,
        });
    }

    pub fn report_text(&mut self, text: String) {
        if !self.text.is_empty() {
            self.text.push_str("\n");
        }
        self.text.push_str(&text);
    }
}

impl fmt::Display for SuiteReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
