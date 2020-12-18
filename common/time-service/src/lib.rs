// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Abstract time service

use enum_dispatch::enum_dispatch;
use std::{fmt::Debug, time::Duration};
#[cfg(any(test, feature = "async"))]
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(any(test, feature = "async"))]
pub mod interval;
#[cfg(any(test, feature = "fuzzing", feature = "testing"))]
pub mod mock;
pub mod real;
#[cfg(any(test, feature = "async"))]
pub mod timeout;

#[cfg(any(test, feature = "fuzzing", feature = "testing"))]
pub use crate::mock::{MockSleep, MockTimeService};
pub use crate::real::RealTimeService;
#[cfg(any(test, feature = "async"))]
pub use crate::{interval::Interval, real::RealSleep, timeout::Timeout};

// TODO(philiphayes): use Duration constants when those stabilize.
#[cfg(any(test, feature = "async"))]
const ZERO_DURATION: Duration = Duration::from_nanos(0);

/// `TimeService` abstracts all time-related operations in one place that can be
/// easily mocked-out and controlled in tests or delegated to the actual
/// underlying runtime (usually tokio). It's provided as an enum so we don't have
/// to infect everything with a generic tag.
///
/// `TimeService` is async-focused: the `sleep`, `interval`, and `timeout` methods
/// all return `Future`s and `Stream`s.
///
/// `TimeService` tries to mirror the API provided by [`tokio::time`] to an
/// extent. The key difference is that all time is expressed in relative
/// [`Duration`]s. In other words, "sleep for 5s" vs "sleep until unix time
/// 1607734460". Absolute time is provided by [`TimeService::now`] which returns
/// the current unix time.
///
/// Note: we have to provide our own [`Timeout`] and [`Interval`] types that
/// use the [`Sleep`] future.
///
/// Note: `TimeService`'s should be free (or very cheap) to clone and send around
/// between threads. In production (without test features), this enum is a
/// zero-sized type.
#[enum_dispatch(TimeServiceTrait)]
#[derive(Clone, Debug)]
pub enum TimeService {
    RealTimeService(RealTimeService),

    #[cfg(any(test, feature = "fuzzing", feature = "testing"))]
    MockTimeService(MockTimeService),
}

impl TimeService {
    pub fn real() -> Self {
        RealTimeService::new().into()
    }

    #[cfg(any(test, feature = "fuzzing", feature = "testing"))]
    pub fn mock() -> Self {
        MockTimeService::new().into()
    }

    #[cfg(any(test, feature = "fuzzing", feature = "testing"))]
    pub fn into_mock(self) -> MockTimeService {
        match self {
            TimeService::MockTimeService(inner) => inner,
            ts => panic!("Unexpected TimeService, expected MockTimeService: {:?}", ts),
        }
    }
}

#[enum_dispatch]
pub trait TimeServiceTrait: Send + Sync + Clone + Debug {
    /// Query the time service for the current unix timestamp.
    fn now(&self) -> Duration;

    /// Return a [`Future`] that waits until `duration` has passed.
    ///
    /// No work is performed while awaiting on the sleep future to complete. `Sleep`
    /// operates at millisecond granularity and should not be used for tasks that
    /// require high-resolution timers.
    ///
    /// # Cancelation
    ///
    /// Canceling a sleep instance is done by dropping the returned future. No
    /// additional cleanup work is required.
    #[cfg(any(test, feature = "async"))]
    fn sleep(&self, duration: Duration) -> Sleep;

    /// Blocks the current thread until `duration` time has passed.
    fn sleep_blocking(&self, duration: Duration);

    /// Creates new [`Interval`] that yields with interval of `period`. The first
    /// tick completes immediately. An interval will tick indefinitely.
    ///
    /// # Cancelation
    ///
    /// At any time, the [`Interval`] value can be dropped. This cancels the interval.
    ///
    /// # Panics
    ///
    /// This function panics if `period` is zero.
    #[cfg(any(test, feature = "async"))]
    fn interval(&self, period: Duration) -> Interval {
        let delay = self.sleep(ZERO_DURATION);
        Interval::new(delay, period)
    }

    /// Require a [`Future`] to complete before the specified duration has elapsed.
    ///
    /// If the future completes before the duration has elapsed, then the completed
    /// value is returned. Otherwise, `Err(Elapsed)` is returned and the future is
    /// canceled.
    ///
    /// # Cancelation
    ///
    /// Cancelling a timeout is done by dropping the future. No additional cleanup
    /// or other work is required.
    ///
    /// The original future may be obtained by calling [`Timeout::into_inner`]. This
    /// consumes the [`Timeout`].
    #[cfg(any(test, feature = "async"))]
    fn timeout<F: Future>(&self, duration: Duration, future: F) -> Timeout<F> {
        let delay = self.sleep(duration);
        Timeout::new(future, delay)
    }
}

/// A [`Future`] that resolves after some time has elapsed (either real or
/// simulated, depending on the parent [`TimeService`]).
///
/// `Sleep` is modeled after [`tokio::time::Delay`].
#[enum_dispatch(SleepTrait)]
#[derive(Debug)]
#[cfg(any(test, feature = "async"))]
pub enum Sleep {
    RealSleep(RealSleep),

    #[cfg(any(test, feature = "fuzzing", feature = "testing"))]
    MockSleep(MockSleep),
}

#[cfg(any(test, feature = "async"))]
impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            Sleep::RealSleep(inner) => Pin::new(inner).poll(cx),
            #[cfg(any(test, feature = "fuzzing", feature = "testing"))]
            Sleep::MockSleep(inner) => Pin::new(inner).poll(cx),
        }
    }
}

#[enum_dispatch]
#[cfg(any(test, feature = "async"))]
pub trait SleepTrait: Future<Output = ()> + Send + Sync + Unpin + Debug {
    /// Returns `true` if this `Sleep`'s requested wait duration has elapsed.
    fn is_elapsed(&self) -> bool;

    /// Resets this `Sleep`'s wait duration.
    fn reset(&mut self, duration: Duration);
}
