// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! `OpCounters` is a collection of convenience methods to add arbitrary counters to modules.
//! For now, it supports Int-Counters, Int-Gauges, and Histogram.

use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, Opts,
};

#[derive(Clone)]
pub struct OpMetrics {
    module: String,
    counters: IntCounterVec,
    gauges: IntGaugeVec,
    peer_gauges: IntGaugeVec,
    histograms: HistogramVec,
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
                    format!("{}_gauge", name_str.clone()),
                    format!("Gauges for {}", name_str),
                ),
                &["op"],
            )
            .unwrap(),
            peer_gauges: IntGaugeVec::new(
                Opts::new(
                    format!("{}_peer_gauge", name_str.clone()),
                    format!("Gauges of each remote peer for {}", name_str),
                ),
                &["op", "remote_peer_id"],
            )
            .unwrap(),
            histograms: HistogramVec::new(
                HistogramOpts::new(
                    format!("{}_duration", name_str.clone()),
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
    pub fn counter(&self, name: &str) -> IntCounter {
        self.counters.with_label_values(&[name])
    }

    #[inline]
    pub fn histogram(&self, name: &str) -> Histogram {
        self.histograms.with_label_values(&[name])
    }

    #[inline]
    pub fn inc(&self, op: &str) {
        self.counters.with_label_values(&[op]).inc();
    }

    #[inline]
    pub fn inc_by(&self, op: &str, v: usize) {
        // The underlying method is expecting i64, but most of the types
        // we're going to log are `u64` or `usize`.
        self.counters.with_label_values(&[op]).inc_by(v as i64);
    }

    #[inline]
    pub fn add(&self, op: &str) {
        self.gauges.with_label_values(&[op]).inc();
    }

    #[inline]
    pub fn sub(&self, op: &str) {
        self.gauges.with_label_values(&[op]).dec();
    }

    #[inline]
    pub fn set(&self, op: &str, v: usize) {
        // The underlying method is expecting i64, but most of the types
        // we're going to log are `u64` or `usize`.
        self.gauges.with_label_values(&[op]).set(v as i64);
    }

    #[inline]
    pub fn observe(&self, op: &str, v: f64) {
        self.histograms.with_label_values(&[op]).observe(v);
    }
}

impl Collector for OpMetrics {
    fn desc(&self) -> Vec<&Desc> {
        let mut ms = Vec::with_capacity(4);
        ms.extend(self.counters.desc());
        ms.extend(self.gauges.desc());
        ms.extend(self.peer_gauges.desc());
        ms.extend(self.histograms.desc());
        ms
    }

    fn collect(&self) -> Vec<MetricFamily> {
        let mut ms = Vec::with_capacity(4);
        ms.extend(self.counters.collect());
        ms.extend(self.gauges.collect());
        ms.extend(self.peer_gauges.collect());
        ms.extend(self.histograms.collect());
        ms
    }
}
