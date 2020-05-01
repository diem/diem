// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use futures::{
    future::{FusedFuture, Future},
    ready,
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// Effectively a size=1 future queue, or a slot with at-most-one future inside.
/// This struct is mostly useful for modeling an optional path in `futures::select!`.
///
/// For now, we require `F: Unpin` which mandates boxing of `async fn` futures
/// but simplifies the `OptionFuture` implementation.
pub struct OptionFuture<F> {
    inner: Option<F>,
}

impl<F> OptionFuture<F>
where
    F: Future + Unpin,
{
    pub fn new(inner: Option<F>) -> Self {
        Self { inner }
    }

    /// If the slot is empty, call the given closure to generate a new future to
    /// put into the slot.
    ///
    /// Note: the caller is responsible for polling the `OptionFuture` afterwards.
    pub fn or_insert_with<G>(&mut self, fun: G)
    where
        G: FnOnce() -> Option<F>,
    {
        if self.inner.is_none() {
            self.inner = fun();
        }
    }
}

impl<F> Unpin for OptionFuture<F> where F: Unpin {}

impl<F> Future for OptionFuture<F>
where
    F: Future + Unpin,
{
    type Output = <F as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // try to poll whatever future is in the inner slot, if there is one.
        let out = match Pin::new(&mut self.as_mut().get_mut().inner).as_pin_mut() {
            Some(f) => ready!(f.poll(cx)),
            None => return Poll::Pending,
        };

        // the inner future is complete so we can drop it now and just return the
        // results.
        self.set(OptionFuture { inner: None });

        Poll::Ready(out)
    }
}

impl<F> FusedFuture for OptionFuture<F>
where
    F: Future + Unpin,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_none()
    }
}
