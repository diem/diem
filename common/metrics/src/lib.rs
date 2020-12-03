// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! # Metrics
//!
//! ## Counters
//!
//! Used to measure values that are added to over time, rates
//! can then be used to check how quickly it changes in graphs.
//! An example would be to add every time an incoming message occurs.
//! ```
//! use prometheus::register_int_counter_vec;
//!
//! register_int_counter_vec!(
//!     "name",
//!     "description",
//!     &["dimension_1", "dimension_2"]
//! );
//! ```
//!
//! ## Gauges
//! Used to measure values that change level over time.  An example
//! would be to set the number of connected peers.
//! ```
//! use prometheus::register_int_gauge_vec;
//!
//! register_int_gauge_vec!(
//!     "name",
//!     "description",
//!     &["dimension_1", "dimension_2"]
//! );
//! ```
//!
//! ## Histograms
//! Used to measure histogram values.  An example is network
//! connection latency.
//! ```
//! use prometheus::register_histogram_vec;
//!
//! register_histogram_vec!(
//!     "name",
//!     "description",
//!     &["dimension_1", "dimension_2"]
//! );
//! ```

#![forbid(unsafe_code)]
#![recursion_limit = "128"]

mod json_encoder;
mod json_metrics;
pub mod metric_server;
mod public_metrics;

mod op_counters;
pub use op_counters::{DurationHistogram, OpMetrics};

#[cfg(test)]
mod unit_tests;

// Re-export counter types from prometheus crate
pub use diem_metrics_core::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, Histogram, HistogramTimer, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
};

use anyhow::Result;
use diem_logger::prelude::*;
use once_cell::sync::Lazy;
use prometheus::{proto::MetricType, Encoder, TextEncoder};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::Write,
    path::Path,
    thread, time,
};

pub static NUM_METRICS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "diem_metrics",
        "Number of metrics in certain states",
        &["type"]
    )
    .unwrap()
});

fn get_metrics_file<P: AsRef<Path>>(dir_path: &P, file_name: &str) -> File {
    create_dir_all(dir_path).expect("Create metrics dir failed");

    let metrics_file_path = dir_path.as_ref().join(file_name);

    info!("Using metrics file {}", metrics_file_path.display());

    OpenOptions::new()
        .append(true)
        .create(true)
        .open(metrics_file_path)
        .expect("Open metrics file failed")
}

pub fn gather_metrics() -> Vec<prometheus::proto::MetricFamily> {
    let metric_families = diem_metrics_core::gather();
    let mut total: u64 = 0;
    let mut families_over_1000: u64 = 0;

    // Take metrics of metric gathering so we know possible overhead of this process
    for metric_family in &metric_families {
        let family_count = metric_family.get_metric().len();
        if family_count > 1000 {
            families_over_1000 = families_over_1000.saturating_add(1);
            let name = metric_family.get_name();
            warn!(
                count = family_count,
                metric_family = name,
                "Metric Family '{}' over 1000 dimensions '{}'",
                name,
                family_count
            );
        }
        total = total.saturating_add(family_count as u64);
    }

    // These metrics will be reported on the next pull, rather than create a new family
    NUM_METRICS.with_label_values(&["total"]).inc_by(total);
    NUM_METRICS
        .with_label_values(&["families_over_1000"])
        .inc_by(families_over_1000);

    metric_families
}

fn get_all_metrics_as_serialized_string() -> Result<Vec<u8>> {
    let all_metrics = gather_metrics();

    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&all_metrics, &mut buffer)?;
    Ok(buffer)
}

pub fn get_all_metrics() -> HashMap<String, String> {
    // TODO: use an existing metric encoder (same as used by
    // prometheus/metric-server)
    let all_metric_families = gather_metrics();
    let mut all_metrics = HashMap::new();
    for metric_family in all_metric_families {
        let values: Vec<_> = match metric_family.get_field_type() {
            MetricType::COUNTER => metric_family
                .get_metric()
                .iter()
                .map(|m| m.get_counter().get_value().to_string())
                .collect(),
            MetricType::GAUGE => metric_family
                .get_metric()
                .iter()
                .map(|m| m.get_gauge().get_value().to_string())
                .collect(),
            MetricType::SUMMARY => panic!("Unsupported Metric 'SUMMARY'"),
            MetricType::UNTYPED => panic!("Unsupported Metric 'UNTYPED'"),
            MetricType::HISTOGRAM => metric_family
                .get_metric()
                .iter()
                .map(|m| m.get_histogram().get_sample_count().to_string())
                .collect(),
        };
        let metric_names = metric_family.get_metric().iter().map(|m| {
            let label_strings: Vec<String> = m
                .get_label()
                .iter()
                .map(|l| format!("{}={}", l.get_name(), l.get_value()))
                .collect();
            let labels_string = format!("{{{}}}", label_strings.join(","));
            format!("{}{}", metric_family.get_name(), labels_string)
        });

        for (name, value) in metric_names.zip(values.into_iter()) {
            all_metrics.insert(name, value);
        }
    }

    all_metrics
}

// Launches a background thread which will periodically collect metrics
// every interval and write them to the provided file
pub fn dump_all_metrics_to_file_periodically<P: AsRef<Path>>(
    dir_path: &P,
    file_name: &str,
    interval: u64,
) {
    let mut file = get_metrics_file(dir_path, file_name);
    thread::spawn(move || loop {
        let mut buffer = get_all_metrics_as_serialized_string().expect("Error gathering metrics");
        if !buffer.is_empty() {
            buffer.push(b'\n');
            file.write_all(&buffer).expect("Error writing metrics");
        }
        thread::sleep(time::Duration::from_millis(interval));
    });
}

/// Helper function to record metrics for external calls.
/// Include call counts, time, and whether it's inside or not (1 or 0).
/// It assumes a OpMetrics defined as OP_COUNTERS in crate::counters;
#[macro_export]
macro_rules! monitor {
    ( $name:literal, $fn:expr ) => {{
        use crate::counters::OP_COUNTERS;
        let _timer = OP_COUNTERS.timer($name);
        let gauge = OP_COUNTERS.gauge(concat!($name, "_running"));
        gauge.inc();
        let result = $fn;
        gauge.dec();
        result
    }};
}
