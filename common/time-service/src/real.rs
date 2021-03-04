// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TimeServiceTrait;
#[cfg(any(test, feature = "async"))]
use crate::{Sleep, SleepTrait};
#[cfg(any(test, feature = "async"))]
use std::pin::Pin;
use std::{
    thread,
    time::{Duration, Instant},
};

/// The real production tokio [`TimeService`](crate::TimeService).
///
/// Note: `RealTimeService` is just a zero-sized type whose methods only delegate
/// to the respective [`tokio::time`] functions.
#[derive(Copy, Clone, Debug, Default)]
pub struct RealTimeService;

#[cfg(any(test, feature = "async"))]
pub type RealSleep = tokio::time::Sleep;

impl RealTimeService {
    pub fn new() -> Self {
        Self {}
    }
}

impl TimeServiceTrait for RealTimeService {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn now_unix_time(&self) -> Duration {
        diem_infallible::duration_since_epoch()
    }

    #[cfg(any(test, feature = "async"))]
    fn sleep(&self, duration: Duration) -> Sleep {
        tokio::time::sleep(duration).into()
    }

    fn sleep_blocking(&self, duration: Duration) {
        thread::sleep(duration);
    }
}

#[cfg(any(test, feature = "async"))]
impl SleepTrait for RealSleep {
    fn is_elapsed(&self) -> bool {
        RealSleep::is_elapsed(self)
    }

    fn reset(self: Pin<&mut Self>, duration: Duration) {
        let deadline = self.deadline() + duration;
        RealSleep::reset(self, deadline);
    }

    fn reset_until(self: Pin<&mut Self>, deadline: Instant) {
        RealSleep::reset(self, tokio::time::Instant::from_std(deadline));
    }
}
