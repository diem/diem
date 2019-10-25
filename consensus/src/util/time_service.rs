// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel;
use futures::{Future, FutureExt, SinkExt};
use libra_logger::prelude::*;
use std::{
    pin::Pin,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{runtime::TaskExecutor, timer::delay_for};

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
    executor: TaskExecutor,
}

impl ClockTimeService {
    /// Creates new TimeService that runs tasks based on actual clock
    /// It needs executor to schedule internal tasks that facilitates it's work
    pub fn new(executor: TaskExecutor) -> ClockTimeService {
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
#[derive(Debug, PartialEq, Eq, Fail)]
pub enum WaitingError {
    /// The waiting period exceeds the maximum allowed duration, returning immediately
    #[fail(display = "MaxWaitExceeded")]
    MaxWaitExceeded,
    /// Waiting to ensure the current time exceeds min_duration_since_epoch failed
    #[fail(display = "WaitFailed")]
    WaitFailed {
        current_duration_since_epoch: Duration,
        wait_duration: Duration,
    },
}

/// Attempt to wait until the current time exceeds the timestamp_then if possible
///
/// If the waiting time exceeds the deadline, then fail immediately.
/// There are 4 potential outcomes, 2 successful and 2 errors, each represented by
/// WaitingSuccess and WaitingError.
pub async fn wait_if_possible(
    time_service: &dyn TimeService,
    timestamp_then: Duration,
    deadline: Instant,
) -> Result<WaitingSuccess, WaitingError> {
    // are we already passed the deadline?
    let instant_now = Instant::now();
    if instant_now >= deadline {
        return Err(WaitingError::WaitFailed {
            current_duration_since_epoch: 0,
            wait_duration: 0,
        });
    }
    // get current timestamp
    let timestamp_now = time_service.get_current_timestamp();
    // we don't need to wait
    if timestamp_now > timestamp_then {
        return Ok(WaitingSuccess::NoWaitRequired {
            timestamp_now,
            early_duration: timestamp_now - timestamp_then,
        });
    }
    // we need to wait, but the deadline will be reached first
    let time_to_timeout = deadline.duration_since(instant_now);
    let time_to_wait = timestamp_then - timestamp_now + Duration::from_millis(1);
    if time_to_timeout < time_to_wait {
        return Err(WaitingError::MaxWaitExceeded);
    }
    // we wait
    time_service.sleep(time_to_wait).await;
    Ok(WaitingSuccess::WaitWasRequired {
        current_duration_since_epoch: timestamp_now,
        wait_duration: time_to_wait,
    })
}
