// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// BaseMetric is a struct used by `Counter` and `Gauge`
struct BaseMetric {
    counter_name: String,
    counter_help: String,
    counter: AtomicI64,
}

impl BaseMetric {
    pub fn new(counter_name: String, counter_help: String) -> Self {
        Self {
            counter_name,
            counter_help,
            counter: std::sync::atomic::AtomicI64::new(0),
        }
    }

    pub fn inc_by(&self, i: i64) {
        self.counter.fetch_add(i, Ordering::Relaxed);
    }

    pub fn inc(&self) {
        self.inc_by(1);
    }

    pub fn set(&self, i: i64) {
        self.counter.store(i, Ordering::Relaxed);
    }
}

/// PushMetricFormat contains a single function which converts the
/// metric into a string format which can be pushed to Prometheus
pub trait PushMetricFormat {
    fn get_push_metric_format(&self) -> String;
}

/// A Counter is a cumulative metric that represents a single
/// monotonically increasing counter whose value can only
/// increase or be reset to zero on restart.
pub struct Counter {
    base_counter: BaseMetric,
}

impl Counter {
    pub fn new(counter_name: String, counter_help: String) -> Self {
        Self {
            base_counter: BaseMetric::new(counter_name, counter_help),
        }
    }

    pub fn inc_by(&self, i: i64) {
        self.base_counter.inc_by(i);
    }

    pub fn inc(&self) {
        self.base_counter.inc();
    }
}

impl PushMetricFormat for Counter {
    fn get_push_metric_format(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n{} {}\n",
            self.base_counter.counter_name,
            self.base_counter.counter_help,
            self.base_counter.counter_name,
            "counter",
            self.base_counter.counter_name,
            self.base_counter.counter.load(Ordering::Relaxed)
        )
    }
}

/// A Gauge is a metric that represents a single
/// numerical value that can arbitrarily go up and down.
pub struct Gauge {
    base_counter: BaseMetric,
    has_been_set: AtomicBool,
}

impl Gauge {
    pub fn new(counter_name: String, counter_help: String) -> Self {
        Self {
            base_counter: BaseMetric::new(counter_name, counter_help),
            has_been_set: AtomicBool::new(false),
        }
    }

    pub fn set(&self, i: i64) {
        self.has_been_set.store(true, Ordering::Relaxed);
        self.base_counter.set(i);
    }
}

impl PushMetricFormat for Gauge {
    fn get_push_metric_format(&self) -> String {
        if self.has_been_set.load(Ordering::Relaxed) {
            format!(
                "# HELP {} {}\n# TYPE {} {}\n{} {}\n",
                self.base_counter.counter_name,
                self.base_counter.counter_help,
                self.base_counter.counter_name,
                "gauge",
                self.base_counter.counter_name,
                self.base_counter.counter.load(Ordering::Relaxed)
            )
        } else {
            String::new()
        }
    }
}

pub trait Metrics {
    fn get_metrics(&self) -> Vec<&dyn PushMetricFormat>;
}
