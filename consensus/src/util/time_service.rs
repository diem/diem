// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{Future, FutureExt, SinkExt};
use libra_logger::prelude::*;
use std::{
    pin::Pin,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tokio::{runtime::Handle, time::delay_for};

/// Time service is an abstraction for operations that depend on time
/// It supports implementations that can simulated time or depend on actual time
/// We can use simulated time in tests so tests can run faster and be more stable.
/// see SimulatedTime for implementation that tests should use
/// Time service also supports opportunities for future optimizations
/// For example instead of scheduling O(N) tasks in TaskExecutor we could have more optimal code
/// that only keeps single task in TaskExecutor
pub trait TimeService: Send + Sync {
    /// Sends message to given sender after timeout
    fn run_after(&self, timeout: Duration, task: Box<dyn ScheduledTask>);

    /// Retrieve the current time stamp as a Duration (assuming it is on or after the UNIX_EPOCH)
    fn get_current_timestamp(&self) -> Duration;

    /// Makes a future that will sleep for given Duration
    /// This function guarantees that get_current_timestamp will increase at least by
    /// given duration, e.g.
    /// X = time_service::get_current_timestamp();
    /// time_service::sleep(Y).await;
    /// Z = time_service::get_current_timestamp();
    /// assert(Z >= X + Y)
    fn sleep(&self, t: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// This trait represents abstract task that can be submitted to TimeService::run_after
pub trait ScheduledTask: Send {
    /// TimeService::run_after will run this method when time expires
    /// It is expected that this function is lightweight and does not take long time to complete
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send>>;
}

/// This tasks send message to given Sender
pub struct SendTask<T>
where
    T: Send + 'static,
{
    sender: Option<channel::Sender<T>>,
    message: Option<T>,
}

impl<T> SendTask<T>
where
    T: Send + 'static,
{
    /// Makes new SendTask for given sender and message and wraps it to Box
    pub fn make(sender: channel::Sender<T>, message: T) -> Box<dyn ScheduledTask> {
        Box::new(SendTask {
            sender: Some(sender),
            message: Some(message),
        })
    }
}

impl<T> ScheduledTask for SendTask<T>
where
    T: Send + 'static,
{
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        let mut sender = self.sender.take().unwrap();
        let message = self.message.take().unwrap();
        let r = async move {
            if let Err(e) = sender.send(message).await {
                error!("Error on send: {:?}", e);
            };
        };
        r.boxed()
    }
}

/// TimeService implementation that uses actual clock to schedule tasks
pub struct ClockTimeService {
    executor: Handle,
}

impl ClockTimeService {
    /// Creates new TimeService that runs tasks based on actual clock
    /// It needs executor to schedule internal tasks that facilitates it's work
    pub fn new(executor: Handle) -> ClockTimeService {
        ClockTimeService { executor }
    }
}

impl TimeService for ClockTimeService {
    fn run_after(&self, timeout: Duration, mut t: Box<dyn ScheduledTask>) {
        let task = async move {
            delay_for(timeout).await;
            t.run().await;
        };
        self.executor.spawn(task);
    }

    fn get_current_timestamp(&self) -> Duration {
        duration_since_epoch()
    }

    fn sleep(&self, t: Duration) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        async move { delay_for(t).await }.boxed()
    }
}

/// Return the duration since the UNIX_EPOCH
pub fn duration_since_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Timestamp generated is before the UNIX_EPOCH!")
}

/// Success states for wait_if_possible
#[derive(Debug, PartialEq, Eq)]
pub enum WaitingSuccess {
    /// No waiting to complete and includes the current duration since epoch and the difference
    /// between the current duration since epoch and min_duration_since_epoch
    NoWaitRequired {
        current_duration_since_epoch: Duration,
        early_duration: Duration,
    },
    /// Waiting was required and includes the current duration since epoch and the duration
    /// slept to finish waiting
    WaitWasRequired {
        current_duration_since_epoch: Duration,
        wait_duration: Duration,
    },
}

/// Error states for wait_if_possible
#[derive(Debug, PartialEq, Eq, Error)]
#[error("{:?}", self)]
pub enum WaitingError {
    /// The waiting period exceeds the maximum allowed duration, returning immediately
    MaxWaitExceeded,
    /// Waiting to ensure the current time exceeds min_duration_since_epoch failed
    WaitFailed {
        current_duration_since_epoch: Duration,
        wait_duration: Duration,
    },
}

/// Attempt to wait until the current time exceeds the min_duration_since_epoch if possible
///
/// If the waiting time exceeds max_instant then fail immediately.
/// There are 4 potential outcomes, 2 successful and 2 errors, each represented by
/// WaitingSuccess and WaitingError.
pub async fn wait_if_possible(
    time_service: &dyn TimeService,
    min_duration_since_epoch: Duration,
    max_instant: Instant,
) -> Result<WaitingSuccess, WaitingError> {
    // Fail early if waiting for min_duration_since_epoch would exceed max_instant
    // Ideally, comparing min_duration_since_epoch and max_instant would be straightforward, but
    // min_duration_since_epoch is relative to UNIX_EPOCH and Instant is not comparable.  Therefore,
    // we use relative differences to do the comparison.
    let current_instant = Instant::now();
    let current_duration_since_epoch = time_service.get_current_timestamp();
    if current_instant <= max_instant {
        let duration_to_max_time = max_instant.duration_since(current_instant);
        if current_duration_since_epoch <= min_duration_since_epoch {
            let duration_to_min_time = min_duration_since_epoch - current_duration_since_epoch;
            if duration_to_max_time < duration_to_min_time {
                return Err(WaitingError::MaxWaitExceeded);
            }
        }
    }

    if current_duration_since_epoch <= min_duration_since_epoch {
        // Delay has millisecond granularity, add 1 millisecond to ensure a higher timestamp
        let sleep_duration =
            min_duration_since_epoch - current_duration_since_epoch + Duration::from_millis(1);
        time_service.sleep(sleep_duration).await;
        let waited_duration_since_epoch = time_service.get_current_timestamp();
        if waited_duration_since_epoch > min_duration_since_epoch {
            Ok(WaitingSuccess::WaitWasRequired {
                current_duration_since_epoch: waited_duration_since_epoch,
                wait_duration: sleep_duration,
            })
        } else {
            Err(WaitingError::WaitFailed {
                current_duration_since_epoch: waited_duration_since_epoch,
                wait_duration: sleep_duration,
            })
        }
    } else {
        Ok(WaitingSuccess::NoWaitRequired {
            current_duration_since_epoch,
            early_duration: current_duration_since_epoch - min_duration_since_epoch,
        })
    }
}
