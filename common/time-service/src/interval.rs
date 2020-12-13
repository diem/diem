// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Sleep, SleepTrait, ZERO_DURATION};
use futures::{future::Future, ready, stream::Stream};
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// Stream returned by [`TimeService::interval`](crate::TimeService::interval).
///
/// Mostly taken from [`tokio::time::Interval`] but uses our `Sleep` future.
#[must_use = "streams do nothing unless you `.await` or poll them"]
pub struct Interval {
    delay: Sleep,
    period: Duration,
}

impl Interval {
    pub fn new(delay: Sleep, period: Duration) -> Self {
        assert!(period > ZERO_DURATION, "`period` must be non-zero.");

        Self { delay, period }
    }
}

impl Stream for Interval {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Wait for the delay to be done
        ready!(Pin::new(&mut this.delay).poll(cx));

        // Reset the delay before next round
        this.delay.reset(this.period);

        Poll::Ready(Some(()))
    }
}
