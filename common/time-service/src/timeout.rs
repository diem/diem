// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Sleep;
use futures::future::Future;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use thiserror::Error;

/// Error returned by [`Timeout`].
#[derive(Debug, Error, Eq, PartialEq)]
#[error("timeout elapsed")]
pub struct Elapsed;

/// Future returned by [`TimeService::timeout`](crate::TimeService::timeout).
///
/// Mostly taken from [`tokio::time::Timeout`] but uses our `Sleep` future.
#[pin_project]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Timeout<F> {
    #[pin]
    future: F,
    #[pin]
    delay: Sleep,
}

impl<F> Timeout<F> {
    pub fn new(future: F, delay: Sleep) -> Self {
        Self { future, delay }
    }

    /// Consumes this timeout, returning the underlying value.
    pub fn into_inner(self) -> F {
        self.future
    }
}

impl<F: Future> Future for Timeout<F> {
    type Output = Result<F::Output, Elapsed>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // First, try polling the future
        if let Poll::Ready(v) = this.future.poll(cx) {
            return Poll::Ready(Ok(v));
        }

        // Now check the timer
        this.delay.poll(cx).map(|_| Err(Elapsed))
    }
}
