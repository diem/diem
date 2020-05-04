// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! `RateLimiter` converts any [`futures::stream::Stream`] into a rate-limited
//! stream, allowing only a certain number of elements to be polled in a given time interval.
//!
//! RateLimiter uses the token-bucket algorithm for flow control. RateLimiter acquires a token/permit
//! before polling any element from the underlying stream. If there are no more tokens, it must wait
//! for new tokens to be added back through a periodic refill process.
//!
//! The token-bucket is in turn implemented using a futures-aware semapahore. The semaphore is
//! initialized with some capacity, and the periodic refill process adds back permits removed
//! during the previous interval.
//!
//! If fine-grained flow control is desired, the refill interval should be kept small.

use futures::{
    future::{Future, FutureExt},
    ready,
    stream::{Fuse, Stream, StreamExt},
};
use futures_semaphore::Semaphore;
use libra_logger::warn;
use pin_project::pin_project;
use std::{mem::ManuallyDrop, pin::Pin, task, task::Poll, time::Duration};
use tokio::time::{interval, Interval};

/// Config parameters for a rate-limiter.
/// `capacity`: Max elements allowed in an interval.
/// `interval`: Granular duration within which control flow is desired.
pub trait RateLimit: Stream {
    fn into_rate_limited(self, refill_interval: Duration, capacity: usize) -> RateLimiter<Self>
    where
        Self: Sized,
    {
        RateLimiter::new(self, refill_interval, capacity)
    }
}

impl<T: ?Sized> RateLimit for T where T: Stream {}

/// Stream for the [`RateLimit::into_rate_limited`](into_rate_limited) function.
#[pin_project]
#[must_use = "streams do nothing unless polled"]
pub struct RateLimiter<T: Stream> {
    // The stream to rate-limit.
    #[pin]
    inner: T,
    // Period for refilling the semaphore.
    #[pin]
    interval: Fuse<Interval>,
    permit_fut: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
    capacity: usize,
    permits: Semaphore,
    permit_acquired: bool,
}

impl<T: Stream> RateLimiter<T> {
    fn new(stream: T, refill_interval: Duration, capacity: usize) -> RateLimiter<T> {
        RateLimiter {
            inner: stream,
            interval: interval(refill_interval).fuse(),
            // Start with 0 permits because tokio::interval ticks immediately on setup.
            permits: Semaphore::new(0),
            capacity,
            permit_fut: None,
            permit_acquired: false,
        }
    }
}

impl<T: Stream> Stream for RateLimiter<T> {
    type Item = T::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Option<Self::Item>> {
        // If the interval is ready, add permits. This is safe to do in all calls to poll_next().
        if let Poll::Ready(Some(_)) = self.as_mut().project().interval.poll_next(cx) {
            if self.permits.available_permits() == 0 {
                warn!(
                    "Message rate exceeding the limit of {} per {:?}",
                    self.capacity, self.interval
                );
            }
            self.permits
                .add_permits(self.capacity - self.permits.available_permits());
        }
        // Poll to get a permit if not already acquired. Once the permit is received, poll for the
        // next message from stream.
        if !self.permit_acquired {
            // Create the future to acquire the permit if not already created.
            if self.permit_fut.is_none() {
                // Clone the semaphore to pass into future.
                let semaphore = self.permits.clone();
                *self.as_mut().project().permit_fut =
                    Some(Box::pin(semaphore.into_permit().map(|permit| {
                        // Permits should not be added back automatically. They are added on each
                        // interval tick.
                        ManuallyDrop::new(permit);
                    })));
            }
            // Wait to acquire the permit.
            ready!(self
                .as_mut()
                .project()
                .permit_fut
                .as_mut()
                .unwrap()
                .as_mut()
                .poll(cx));
            *self.as_mut().project().permit_acquired = true;
            *self.as_mut().project().permit_fut = None;
        }
        // Wait to read the next element from the stream.
        let next = ready!(self.as_mut().project().inner.poll_next(cx));
        // Set permit_acquired to false for next message.
        *self.project().permit_acquired = false;
        Poll::Ready(next)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::stream;
    use tokio::time::delay_for;

    // Create an infinite stream using stream::repeat() and rate-limit by once using a small bound
    // and the other time using a large bound. In both cases, the number of polled values should be
    // driven by the rate-limit.
    #[tokio::test]
    async fn test_limits() {
        // Test small bound.
        {
            // Create a stream that continously produces the unit value.
            let s = stream::repeat(());

            // Rate-limit the stream to produce only 10 values in 100ms.
            let mut rs = s.into_rate_limited(Duration::from_millis(100), 10).fuse();

            // Poll the stream `rs` for 50ms and count the number of elements received.
            let mut timeout = delay_for(Duration::from_millis(50)).fuse();
            let mut count = 0;
            loop {
                futures::select! {
                    _ = timeout => {
                        break;
                    },
                    _ = rs.select_next_some() => {
                        count += 1;
                    },
                }
            }
            // Only 10 elements should have been allowed.
            assert_eq!(count, 10);
        }

        // Test large bound.
        {
            // Create another stream that continously produces the unit value.
            let s = stream::repeat(());

            // Rate-limit the stream to produce 100 values in 10ms.
            let mut rs = s.into_rate_limited(Duration::from_millis(10), 100).fuse();

            // Poll the stream `rs` for 50ms and count the number of elements received.
            let mut timeout = delay_for(Duration::from_millis(5)).fuse();
            let mut count = 0;
            loop {
                futures::select! {
                    _ = timeout => {
                        break;
                    },
                    _ = rs.select_next_some() => {
                        count += 1;
                    },
                }
            }
            // 100 elements should have been allowed.
            assert_eq!(count, 100);
        }
    }
}
