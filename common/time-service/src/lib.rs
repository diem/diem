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
#[cfg(any(test, feature = "testing"))]
pub mod mock;
pub mod real;
#[cfg(any(test, feature = "async"))]
pub mod timeout;

#[cfg(any(test, feature = "testing"))]
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
/// all return `Future`s and `Stream`s. That said, `TimeService` supports non-async
/// clients as well; simply use the `sleep_blocking` method instead of `sleep`.
/// Note that the blocking call will actually block the current thread until the
/// sleep time has elapsed.
///
/// `TimeService` tries to mirror the API provided by [`tokio::time`] to an
/// extent. The primary difference is that all time is expressed in relative
/// [`Duration`]s. In other words, "sleep for 5s" vs "sleep until unix time
/// 1607734460". Absolute time is provided by [`TimeService::now`] which returns
/// the current unix time.
///
/// Note: you must also include the [`TimeServiceTrait`] to use the actual
/// time-related functionality.
///
/// Note: we have to provide our own [`Timeout`] and [`Interval`] types that
/// use the [`Sleep`] future, since tokio's implementations are coupled to its
/// internal Sleep future.
///
/// Note: `TimeService`'s should be free (or very cheap) to clone and send around
/// between threads. In production (without test features), this enum is a
/// zero-sized type.
#[enum_dispatch(TimeServiceTrait)]
#[derive(Clone, Debug)]
pub enum TimeService {
    RealTimeService(RealTimeService),

    #[cfg(any(test, feature = "testing"))]
    MockTimeService(MockTimeService),
}

impl TimeService {
    /// Create a new real, production time service that actually uses the systemtime.
    ///
    /// See [`RealTimeService`].
    pub fn real() -> Self {
        RealTimeService::new().into()
    }

    /// Create a mock, simulated time service that does not query the system time
    /// and allows fine-grained control over advancing time and waking sleeping
    /// tasks.
    ///
    /// See [`MockTimeService`].
    #[cfg(any(test, feature = "testing"))]
    pub fn mock() -> Self {
        MockTimeService::new().into()
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn into_mock(self) -> MockTimeService {
        match self {
            TimeService::MockTimeService(inner) => inner,
            ts => panic!("Unexpected TimeService, expected MockTimeService: {:?}", ts),
        }
    }
}

impl Default for TimeService {
    fn default() -> Self {
        Self::real()
    }
}

#[enum_dispatch]
pub trait TimeServiceTrait: Send + Sync + Clone + Debug {
    /// Query the current unix timestamp as a [`Duration`].
    ///
    /// When used on a `TimeService::real()`, this is equivalent to
    /// `SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)`.
    ///
    /// Note: the [`Duration`] returned from this function is _NOT_ guaranteed to
    /// be monotonic.
    ///
    /// From the [`SystemTime`] docs:
    ///
    /// > Distinct from the [`Instant`] type, this time measurement is
    /// > not monotonic. This means that you can save a file to the file system,
    /// > then save another file to the file system, and the second file has a
    /// > [`SystemTime`] measurement earlier than the first. In other words, an
    /// > operation that happens after another operation in real time may have
    /// > an earlier SystemTime!
    ///
    /// For example, the system administrator could [`clock_settime`] into the
    /// past, breaking clock time monotonicity.
    ///
    /// [`Duration`]: std::time::Duration
    /// [`Instant`]: std::time::Instant
    /// [`SystemTime`]: std::time::SystemTime
    /// [`clock_settime`]: https://linux.die.net/man/3/clock_settime
    fn now(&self) -> Duration;

    /// Query the current unix timestamp in seconds.
    ///
    /// Equivalent to `self.now().as_secs()`. See [`now`](#method.now).
    fn now_secs(&self) -> u64 {
        self.now().as_secs()
    }

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

    #[cfg(any(test, feature = "testing"))]
    MockSleep(MockSleep),
}

#[cfg(any(test, feature = "async"))]
impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            Sleep::RealSleep(inner) => Pin::new(inner).poll(cx),
            #[cfg(any(test, feature = "testing"))]
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
