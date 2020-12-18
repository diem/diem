// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Sleep, SleepTrait, TimeServiceTrait};
use std::time::Duration;

/// The real production tokio [`TimeService`].
///
/// Note: `RealTimeService` is just a zero-sized type whose methods only delegate
/// to the respective [`tokio::time`] functions.
#[derive(Copy, Clone, Debug, Default)]
pub struct RealTimeService;

pub type RealSleep = tokio::time::Delay;

impl RealTimeService {
    pub fn new() -> Self {
        Self {}
    }
}

impl TimeServiceTrait for RealTimeService {
    fn now(&self) -> Duration {
        diem_infallible::duration_since_epoch()
    }

    /// See [`tokio::time::delay_for`]
    fn sleep(&self, duration: Duration) -> Sleep {
        tokio::time::delay_for(duration).into()
    }
}

impl SleepTrait for RealSleep {
    fn is_elapsed(&self) -> bool {
        RealSleep::is_elapsed(self)
    }

    fn reset(&mut self, duration: Duration) {
        let deadline = self.deadline() + duration;
        RealSleep::reset(self, deadline);
    }
}
