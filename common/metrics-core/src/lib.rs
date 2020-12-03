// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Re-export counter types from prometheus crate
pub use prometheus::{
    gather, register_histogram, register_histogram_vec, register_int_counter,
    register_int_counter_vec, register_int_gauge, register_int_gauge_vec, Encoder, Histogram,
    HistogramTimer, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec, TextEncoder,
};
