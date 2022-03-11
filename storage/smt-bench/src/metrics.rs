// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_metrics::{register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec};
use once_cell::sync::Lazy;

pub static TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!("smt_bench_timer", "smt bench timer", &["name",]).unwrap()
});

pub static HISTOGRAM: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!("smt_bench_histogram", "smt bench histogram", &["name",]).unwrap()
});

pub static COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!("smt_bench_counter", "smt bench counter", &["name"]).unwrap()
});
