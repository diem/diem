// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! `OpCounters` is a collection of convenience methods to add arbitrary counters to modules.
//! For now, it supports Int-Counters, Int-Gauges, and Histogram.

use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    Histogram, HistogramOpts, HistogramTimer, HistogramVec, IntCounterVec, IntGauge, IntGaugeVec,
    Opts,
};

use std::time::Duration;

/// A small wrapper around Histogram to handle the special case
/// of duration buckets.
/// This Histogram will handle the correct granularity for logging
/// time duration in a way that fits the used buckets.
pub struct DurationHistogram {
    histogram: Histogram,
}

impl DurationHistogram {
    pub fn new(histogram: Histogram) -> DurationHistogram {
        DurationHistogram { histogram }
    }

    pub fn observe_duration(&self, d: Duration) {
        // Duration is full seconds + nanos elapsed from the previous full second
        let v = d.as_secs() as f64 + f64::from(d.subsec_nanos()) / 1e9;
        self.histogram.observe(v);
    }

    pub fn start_timer(&self) -> HistogramTimer {
        self.histogram.start_timer()
    }
}

#[derive(Clone)]
pub struct OpMetrics {
    module: String,
    counters: IntCounterVec,
    gauges: IntGaugeVec,
    peer_gauges: IntGaugeVec,
    duration_histograms: HistogramVec,
}

impl OpMetrics {
    pub fn new<S: Into<String>>(name: S) -> OpMetrics {
        let name_str = name.into();
        OpMetrics {
            module: name_str.clone(),
            counters: IntCounterVec::new(
                Opts::new(name_str.clone(), format!("Counters for {}", name_str)),
                &["op"],
            )
            .unwrap(),
            gauges: IntGaugeVec::new(
                Opts::new(
                    format!("{}_gauge", name_str),
                    format!("Gauges for {}", name_str),
                ),
                &["op"],
            )
            .unwrap(),
            peer_gauges: IntGaugeVec::new(
                Opts::new(
                    format!("{}_peer_gauge", name_str),
                    format!("Gauges of each remote peer for {}", name_str),
                ),
                &["op", "remote_peer_id"],
            )
            .unwrap(),
            duration_histograms: HistogramVec::new(
                HistogramOpts::new(
                    format!("{}_duration", name_str),
                    format!("Histogram values for {}", name_str),
                ),
                &["op"],
            )
            .unwrap(),
        }
    }

    pub fn new_and_registered<S: Into<String>>(name: S) -> OpMetrics {
        let op_metrics = OpMetrics::new(name);
        prometheus::register(Box::new(op_metrics.clone()))
            .expect("OpMetrics registration on Prometheus failed.");
        op_metrics
    }

    #[inline]
    pub fn gauge(&self, name: &str) -> IntGauge {
        self.gauges.with_label_values(&[name])
    }

    #[inline]
    pub fn peer_gauge(&self, name: &str, remote_peer_id: &str) -> IntGauge {
        self.peer_gauges.with_label_values(&[name, remote_peer_id])
    }

    #[inline]
    pub fn inc(&self, op: &str) {
        self.counters.with_label_values(&[op]).inc();
    }

    #[inline]
    pub fn inc_by(&self, op: &str, v: usize) {
        // The underlying method is expecting i64, but most of the types
        // we're going to log are `u64` or `usize`.
        self.counters.with_label_values(&[op]).inc_by(v as u64);
    }

    #[inline]
    pub fn set(&self, op: &str, v: usize) {
        // The underlying method is expecting i64, but most of the types
        // we're going to log are `u64` or `usize`.
        self.gauges.with_label_values(&[op]).set(v as i64);
    }

    #[inline]
    pub fn observe(&self, op: &str, v: f64) {
        self.duration_histograms.with_label_values(&[op]).observe(v);
    }

    pub fn observe_duration(&self, op: &str, d: Duration) {
        // Duration is full seconds + nanos elapsed from the previous full second
        let v = d.as_secs() as f64 + f64::from(d.subsec_nanos()) / 1e9;
        self.duration_histograms.with_label_values(&[op]).observe(v);
    }

    pub fn timer(&self, op: &str) -> HistogramTimer {
        self.duration_histograms
            .with_label_values(&[op])
            .start_timer()
    }
}

impl Collector for OpMetrics {
    fn desc(&self) -> Vec<&Desc> {
        let mut ms = Vec::with_capacity(4);
        ms.extend(self.counters.desc());
        ms.extend(self.gauges.desc());
        ms.extend(self.peer_gauges.desc());
        ms.extend(self.duration_histograms.desc());
        ms
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut ms = Vec::with_capacity(4);
        ms.extend(self.counters.collect());
        ms.extend(self.gauges.collect());
        ms.extend(self.peer_gauges.collect());
        ms.extend(self.duration_histograms.collect());
        ms
    }
}
