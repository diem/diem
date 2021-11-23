// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_logger::prelude::*;
use futures::{
    future::{AbortHandle, Abortable},
    Future, FutureExt, SinkExt,
};
use std::{pin::Pin, thread, time::Duration};

use crate::counters;
use tokio::{runtime::Handle, time::sleep};

/// Time service is an abstraction for operations that depend on time
/// It supports implementations that can simulated time or depend on actual time
/// We can use simulated time in tests so tests can run faster and be more stable.
/// see SimulatedTime for implementation that tests should use
/// Time service also supports opportunities for future optimizations
/// For example instead of scheduling O(N) tasks in TaskExecutor we could have more optimal code
/// that only keeps single task in TaskExecutor
pub trait TimeService: Send + Sync {
    /// Sends message to given sender after timeout, returns a handle that could use to cancel the task.
    fn run_after(&self, timeout: Duration, task: Box<dyn ScheduledTask>) -> AbortHandle;

    /// Retrieve the current time stamp as a Duration (assuming it is on or after the UNIX_EPOCH)
    fn get_current_timestamp(&self) -> Duration;

    /// Makes a future that will sleep for given Duration
    /// This function guarantees that get_current_timestamp will increase at least by
    /// given duration, e.g.
    /// X = time_service::get_current_timestamp();
    /// time_service::sleep(Y).await;
    /// Z = time_service::get_current_timestamp();
    /// assert(Z >= X + Y)
    fn sleep(&self, t: Duration);

    /// Wait until the Duration t since UNIX_EPOCH pass at least 1ms.
    fn wait_until(&self, t: Duration) {
        while let Some(mut wait_duration) = t.checked_sub(self.get_current_timestamp()) {
            wait_duration += Duration::from_millis(1);
            if wait_duration > Duration::from_secs(10) {
                error!(
                    "[TimeService] long wait time {} seconds required",
                    wait_duration.as_secs()
                );
            }
            counters::WAIT_DURATION_S.observe_duration(wait_duration);
            self.sleep(wait_duration);
        }
    }
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
    fn run_after(&self, timeout: Duration, mut t: Box<dyn ScheduledTask>) -> AbortHandle {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let task = Abortable::new(
            async move {
                sleep(timeout).await;
                t.run().await;
            },
            abort_registration,
        );
        self.executor.spawn(task);
        abort_handle
    }

    fn get_current_timestamp(&self) -> Duration {
        diem_infallible::duration_since_epoch()
    }

    fn sleep(&self, t: Duration) {
        thread::sleep(t)
    }
}

#[tokio::test]
async fn test_time_service_abort() {
    use futures::StreamExt;

    let time_service = ClockTimeService::new(tokio::runtime::Handle::current());
    let (tx, mut rx) = channel::new_test(10);
    let task1 = SendTask::make(tx.clone(), 1);
    let task2 = SendTask::make(tx.clone(), 2);
    let handle1 = time_service.run_after(Duration::from_millis(100), task1);
    let handle2 = time_service.run_after(Duration::from_millis(200), task2);
    handle1.abort();
    // task 1 is aborted
    assert_eq!(rx.next().await, Some(2));
    drop(tx);
    assert_eq!(rx.next().await, None);
    // abort an already finished task is no-op
    handle2.abort();
}
