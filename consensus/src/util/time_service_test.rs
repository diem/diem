// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::util::{
    mock_time_service::SimulatedTimeService,
    time_service::{wait_if_possible, TimeService, WaitingError, WaitingSuccess},
};
use std::time::{Duration, Instant};

#[test]
fn wait_if_possible_test_waiting_required() {
    let simulated_time = SimulatedTimeService::new();
    let min_duration_since_epoch = Duration::from_secs(1);
    let max_instant = Instant::now() + Duration::from_secs(2);
    let result = wait_if_possible(&simulated_time, min_duration_since_epoch, max_instant);

    assert_eq!(
        result.ok().unwrap(),
        WaitingSuccess::WaitWasRequired {
            current_duration_since_epoch: min_duration_since_epoch + Duration::from_millis(1),
            wait_duration: min_duration_since_epoch + Duration::from_millis(1),
        }
    );
}

#[test]
fn wait_if_possible_test_no_wait_required() {
    let simulated_time = SimulatedTimeService::new();
    simulated_time.sleep(Duration::from_secs(3));
    let min_duration_since_epoch = Duration::from_secs(1);
    let max_instant = Instant::now() + Duration::from_secs(5);
    let result = wait_if_possible(&simulated_time, min_duration_since_epoch, max_instant);

    assert_eq!(
        result.ok().unwrap(),
        WaitingSuccess::NoWaitRequired {
            current_duration_since_epoch: Duration::from_secs(3),
            early_duration: Duration::from_secs(2)
        }
    );
}

#[test]
fn wait_if_possible_test_max_duration_exceeded() {
    let simulated_time = SimulatedTimeService::new();
    let min_duration_since_epoch = Duration::from_secs(3);
    let max_instant = Instant::now() + Duration::from_secs(2);
    let result = wait_if_possible(&simulated_time, min_duration_since_epoch, max_instant);

    assert_eq!(result.err().unwrap(), WaitingError::MaxWaitExceeded);
}

#[test]
fn wait_if_possible_test_sleep_failed() {
    let simulated_time = SimulatedTimeService::max(Duration::from_secs(1));
    let min_duration_since_epoch = Duration::from_secs(2);
    let max_instant = Instant::now() + Duration::from_secs(3);
    let result = wait_if_possible(&simulated_time, min_duration_since_epoch, max_instant);

    assert_eq!(
        result.err().unwrap(),
        WaitingError::WaitFailed {
            wait_duration: min_duration_since_epoch + Duration::from_millis(1),
            current_duration_since_epoch: Duration::from_secs(1)
        }
    );
}
