// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Provides an mpsc (multi-producer single-consumer) channel wrapped in an
//! [`IntGauge`](diem_metrics::IntGauge) that counts the number of currently
//! queued items. While there is only one [`channel::Receiver`], there can be
//! many [`channel::Sender`]s, which are also cheap to clone.
//!
//! This channel differs from our other channel implementation, [`channel::diem_channel`],
//! in that it is just a single queue (vs. different queues for different keys)
//! with backpressure (senders will block if the queue is full instead of evicting
//! another item in the queue) that only implements FIFO (vs. LIFO or KLAST).

use diem_metrics::IntGauge;
use futures::{
    channel::mpsc,
    sink::Sink,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use std::pin::Pin;

#[cfg(test)]
mod test;

pub mod diem_channel;
#[cfg(test)]
mod diem_channel_test;

pub mod message_queues;
#[cfg(test)]
mod message_queues_test;

/// An [`mpsc::Sender`](futures::channel::mpsc::Sender) with an [`IntGauge`]
/// counting the number of currently queued items.
pub struct Sender<T> {
    inner: mpsc::Sender<T>,
    gauge: IntGauge,
}

/// An [`mpsc::Receiver`](futures::channel::mpsc::Receiver) with an [`IntGauge`]
/// counting the number of currently queued items.
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
    gauge: IntGauge,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            gauge: self.gauge.clone(),
        }
    }
}

/// `Sender` implements `Sink` in the same way as `mpsc::Sender`, but it increments the
/// associated `IntGauge` when it sends a message successfully.
impl<T> Sink<T> for Sender<T> {
    type Error = mpsc::SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self).inner.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, msg: T) -> Result<(), Self::Error> {
        (*self).inner.start_send(msg).map(|_| self.gauge.inc())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl<T> Sender<T> {
    pub fn try_send(&mut self, msg: T) -> Result<(), mpsc::SendError> {
        (*self)
            .inner
            .try_send(msg)
            .map(|_| self.gauge.inc())
            .map_err(mpsc::TrySendError::into_send_error)
    }
}

impl<T> FusedStream for Receiver<T>
where
    T: std::fmt::Debug,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

/// `Receiver` implements `Stream` in the same way as `mpsc::Stream`, but it decrements the
/// associated `IntGauge` when it gets polled successfully.
impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = Pin::new(&mut self.inner).poll_next(cx);
        if let Poll::Ready(Some(_)) = next {
            self.gauge.dec();
        }
        next
    }
}

/// Similar to `mpsc::channel`, `new` creates a pair of `Sender` and `Receiver`
pub fn new<T>(size: usize, gauge: &IntGauge) -> (Sender<T>, Receiver<T>) {
    gauge.set(0);
    let (sender, receiver) = mpsc::channel(size);
    (
        Sender {
            inner: sender,
            gauge: gauge.clone(),
        },
        Receiver {
            inner: receiver,
            gauge: gauge.clone(),
        },
    )
}

pub fn new_test<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let gauge = IntGauge::new("TEST_COUNTER", "test").unwrap();
    new(size, &gauge)
}
