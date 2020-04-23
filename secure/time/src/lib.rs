// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::sleep,
    time::{Duration, SystemTime},
};

/// A generic service for providing time related operations (e.g., returning the current time and
/// sleeping).
pub trait TimeService {
    /// Returns the current time since the UNIX_EPOCH in seconds as a u64.
    fn now(&self) -> u64;

    /// Sleeps the calling thread for (at least) the specified number of seconds. This call may
    /// sleep longer than specified, never less.
    fn sleep(&self, seconds: u64);
}

/// A real-time TimeService
#[derive(Default)]
pub struct RealTimeService;

impl RealTimeService {
    pub fn new() -> Self {
        Self {}
    }
}

impl TimeService for RealTimeService {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn sleep(&self, seconds: u64) {
        sleep(Duration::new(seconds, 0));
    }
}

/// A mock-time TimeService
#[derive(Clone, Default)]
pub struct MockTimeService {
    now: Arc<AtomicU64>,
}

impl MockTimeService {
    pub fn new() -> Self {
        Self {
            now: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment(&self) {
        self.now.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_by(&self, value: u64) {
        self.now.fetch_add(value, Ordering::Relaxed);
    }
}

impl TimeService for MockTimeService {
    fn now(&self) -> u64 {
        self.now.load(Ordering::Relaxed)
    }

    fn sleep(&self, seconds: u64) {
        self.increment_by(seconds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_real_time() {
        test_time_service(&RealTimeService::new());
    }

    #[test]
    fn verify_mock_time() {
        let service = MockTimeService::new();
        test_time_service(&service);

        assert_eq!(service.now(), 0);
        service.increment();
        assert_eq!(service.now(), 1);
    }

    #[test]
    fn test_sleep() {
        // This time shouldn't be too large because it actually sleeps the testing thread when
        // using the RealTimeService!
        let sleep_time = 1;

        // Test real service
        let service = RealTimeService::new();
        verify_sleep(&service, sleep_time);

        // Test mock service
        let service = MockTimeService::new();
        verify_sleep(&service, sleep_time);
    }

    fn verify_sleep<T: TimeService>(service: &T, sleep_time: u64) {
        let current_time = service.now();
        service.sleep(sleep_time);

        assert!(service.now() >= current_time + sleep_time);
    }

    fn test_time_service<T: TimeService>(service: &T) {
        service.now();
    }
}
