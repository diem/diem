// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

// Re-export counter types from prometheus crate
pub use diem_metrics_core::{
    register_histogram, register_histogram_vec, register_int_counter, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, Histogram, HistogramTimer, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
};

use diem_logger::{error, info};
use diem_metrics_core::{Encoder, TextEncoder};
use std::{env, sync::mpsc, thread, thread::JoinHandle, time::Duration};

const DEFAULT_PUSH_FREQUENCY_SECS: u64 = 15;

/// MetricsPusher provides a function to push a list of Metrics to a configurable
/// pushgateway endpoint.
#[must_use = "Assign the contructed pusher to a variable, \
              otherwise the worker thread is joined immediately."]
pub struct MetricsPusher {
    worker_thread: Option<JoinHandle<()>>,
    quit_sender: mpsc::Sender<()>,
}

impl MetricsPusher {
    fn push(push_metrics_endpoint: &str) {
        let mut buffer = Vec::new();

        if let Err(e) = TextEncoder::new().encode(&diem_metrics_core::gather(), &mut buffer) {
            error!("Failed to encode push metrics: {}.", e.to_string());
        } else {
            let response = ureq::post(&push_metrics_endpoint)
                .timeout_connect(10_000)
                .send_bytes(&buffer);
            if let Some(error) = response.synthetic_error() {
                error!(
                    "Failed to push metrics to {}. Error: {}",
                    push_metrics_endpoint, error
                );
            }
        }
    }

    fn worker(
        quit_receiver: mpsc::Receiver<()>,
        push_metrics_endpoint: String,
        push_metrics_frequency_secs: u64,
    ) {
        while quit_receiver
            .recv_timeout(Duration::from_secs(push_metrics_frequency_secs))
            .is_err()
        {
            // Timeout, no quit signal received.
            Self::push(&push_metrics_endpoint);
        }
        // final push
        Self::push(&push_metrics_endpoint);
    }

    fn start_worker_thread(quit_receiver: mpsc::Receiver<()>) -> Option<JoinHandle<()>> {
        // eg value for PUSH_METRICS_ENDPOINT: "http://pushgateway.server.com:9091/metrics/job/safety_rules"
        let push_metrics_endpoint = match env::var("PUSH_METRICS_ENDPOINT") {
            Ok(s) => s,
            Err(_) => {
                info!("PUSH_METRICS_ENDPOINT env var is not set. Skipping sending metrics.");
                return None;
            }
        };
        let push_metrics_frequency_secs = match env::var("PUSH_METRICS_FREQUENCY_SECS") {
            Ok(s) => match s.parse::<u64>() {
                Ok(i) => i,
                Err(_) => {
                    error!("Invalid value for PUSH_METRICS_FREQUENCY_SECS: {}", s);
                    return None;
                }
            },
            Err(_) => DEFAULT_PUSH_FREQUENCY_SECS,
        };
        info!(
            "Starting push metrics loop. Sending metrics to {} with a frequency of {} seconds",
            push_metrics_endpoint, push_metrics_frequency_secs
        );
        Some(thread::spawn(move || {
            Self::worker(
                quit_receiver,
                push_metrics_endpoint,
                push_metrics_frequency_secs,
            )
        }))
    }

    /// start starts a new thread and periodically pushes the metrics to a pushgateway endpoint
    pub fn start() -> Self {
        let (tx, rx) = mpsc::channel();
        let worker_thread = Self::start_worker_thread(rx);

        Self {
            worker_thread,
            quit_sender: tx,
        }
    }

    pub fn join(&mut self) {
        if let Some(worker_thread) = self.worker_thread.take() {
            if let Err(e) = self.quit_sender.send(()) {
                error!(
                    "Failed to send quit signal to metric pushing worker thread: {:?}",
                    e
                );
            }
            if let Err(e) = worker_thread.join() {
                error!("Failed to join metric pushing worker thread: {:?}", e);
            }
        }
    }
}

impl Drop for MetricsPusher {
    fn drop(&mut self) {
        self.join()
    }
}
