// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::protocols::health_checker::{
    HealthChecker, HealthCheckerNetworkEvents, HealthCheckerNetworkSender,
};
use diem_config::network_id::NetworkContext;
use diem_time_service::TimeService;
use std::{sync::Arc, time::Duration};
use tokio::runtime::Handle;

pub struct HealthCheckerBuilder {
    service: Option<HealthChecker>,
}

impl HealthCheckerBuilder {
    pub fn new(
        network_context: Arc<NetworkContext>,
        time_service: TimeService,
        ping_interval_ms: u64,
        ping_timeout_ms: u64,
        ping_failures_tolerated: u64,
        network_tx: HealthCheckerNetworkSender,
        network_rx: HealthCheckerNetworkEvents,
    ) -> Self {
        let service = HealthChecker::new(
            network_context,
            time_service,
            network_tx,
            network_rx,
            Duration::from_millis(ping_interval_ms),
            Duration::from_millis(ping_timeout_ms),
            ping_failures_tolerated,
        );
        Self {
            service: Some(service),
        }
    }

    pub fn start(&mut self, executor: &Handle) {
        if let Some(service) = self.service.take() {
            executor.spawn(service.start());
        }
    }
}
