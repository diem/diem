// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Provides an mpsc (multi-producer single-consumer) channel wrapped in an
//! [`IntGauge`](libra_metrics::IntGauge)

use futures::{
    channel::mpsc,
    sink::Sink,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use libra_logger::prelude::*;
use libra_metrics::IntGauge;
use once_cell::sync::Lazy;
use std::{
    pin::Pin,
    time::{Duration, Instant},
};

#[cfg(test)]
mod test;

pub mod libra_channel;
#[cfg(test)]
mod libra_channel_test;

pub mod message_queues;
#[cfg(test)]
mod message_queues_test;

const MAX_TIMEOUT: Duration = Duration::from_secs(24 * 60 * 60);

/// Wrapper around a value with an entry timestamp
/// It is used to measure the time waiting in the `mpsc::channel`.
pub struct WithEntryTimestamp<T> {
    entry_time: Instant,
    value: T,
}

impl<T> WithEntryTimestamp<T> {
    fn new(value: T) -> Self {
        Self {
            entry_time: Instant::now(),
            value,
        }
    }
}

/// Similar to `mpsc::Sender`, but with an `IntGauge`
pub struct Sender<T> {
    inner: mpsc::Sender<WithEntryTimestamp<T>>,
    gauge: IntGauge,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            inner: self.inner.clone(),
            gauge: self.gauge.clone(),
        }
    }
}

/// Similar to `mpsc::Receiver`, but with an `IntGauge`
pub struct Receiver<T> {
    inner: mpsc::Receiver<WithEntryTimestamp<T>>,
    gauge: IntGauge,
    timeout: Duration,
}

/// `Sender` implements `Sink` in the same way as `mpsc::Sender`, but it increments the
/// associated `IntGauge` when it sends a message successfully.
impl<T> Sink<T> for Sender<T> {
    type Error = mpsc::SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self).inner.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, msg: T) -> Result<(), Self::Error> {
        self.gauge.inc();
        (*self)
            .inner
            .start_send(WithEntryTimestamp::new(msg))
            .map_err(|e| {
                self.gauge.dec();
                e
            })?;
        Ok(())
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
        self.gauge.inc();
        (*self)
            .inner
            .try_send(WithEntryTimestamp::new(msg))
            .map_err(|e| {
                self.gauge.dec();
                e.into_send_error()
            })
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
impl<T> Stream for Receiver<T>
where
    T: std::fmt::Debug,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(msg)) => {
                    self.gauge.dec();
                    // If the message times out, it gets dropped
                    if Instant::now().duration_since(msg.entry_time) > self.timeout {
                        warn!("Message dropped due to timeout: {:?}", msg.value);
                        continue;
                    } else {
                        return Poll::Ready(Some(msg.value));
                    }
                }
                Poll::Ready(None) => {
                    return Poll::Ready(None);
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

/// Similar to `mpsc::channel`, `new` creates a pair of `Sender` and `Receiver`
pub fn new<T>(size: usize, gauge: &IntGauge) -> (Sender<T>, Receiver<T>) {
    new_with_timeout(size, gauge, MAX_TIMEOUT)
}

pub fn new_with_timeout<T>(
    size: usize,
    gauge: &IntGauge,
    timeout: Duration,
) -> (Sender<T>, Receiver<T>) {
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
            timeout,
        },
    )
}

pub static TEST_COUNTER: Lazy<IntGauge> =
    Lazy::new(|| IntGauge::new("TEST_COUNTER", "Counter of network tests").unwrap());

pub fn new_test<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    new(size, &TEST_COUNTER)
}

pub fn new_test_with_timeout<T>(size: usize, timeout: Duration) -> (Sender<T>, Receiver<T>) {
    new_with_timeout(size, &TEST_COUNTER, timeout)
}
