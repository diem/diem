// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides an mpsc (multi-producer single-consumer) channel wrapped in an
//! [`IntGauge`](metrics::IntGauge)

use futures::{
    channel::mpsc,
    sink::Sink,
    stream::{FusedStream, Stream},
    task::{Context, Poll},
};
use metrics::IntGauge;
use std::pin::Pin;

#[cfg(test)]
mod test;

/// Wrapper around a value with an `IntGauge`
/// It is used to gauge the number of elements in a `mpsc::channel`
#[derive(Clone)]
pub struct WithGauge<T> {
    gauge: IntGauge,
    value: T,
}

/// Similar to `mpsc::Sender`, but with an `IntGauge`
pub type Sender<T> = WithGauge<mpsc::Sender<T>>;
/// Similar to `mpsc::Receiver`, but with an `IntGauge`
pub type Receiver<T> = WithGauge<mpsc::Receiver<T>>;

/// `Sender` implements `Sink` in the same way as `mpsc::Sender`, but it increments the
/// associated `IntGauge` when it sends a message successfully.
impl<T> Sink<T> for Sender<T> {
    type Error = mpsc::SendError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        (*self).value.poll_ready(cx)
    }

    fn start_send(mut self: Pin<&mut Self>, msg: T) -> Result<(), Self::Error> {
        self.gauge.inc();
        (*self).value.start_send(msg).map_err(|e| {
            self.gauge.dec();
            e
        })?;
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.value).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.value).poll_close(cx)
    }
}

impl<T> Sender<T> {
    pub fn try_send(&mut self, msg: T) -> Result<(), mpsc::SendError> {
        self.gauge.inc();
        (*self).value.try_send(msg).map_err(|e| {
            self.gauge.dec();
            e.into_send_error()
        })
    }
}

impl<T> FusedStream for Receiver<T> {
    fn is_terminated(&self) -> bool {
        self.value.is_terminated()
    }
}

/// `Receiver` implements `Stream` in the same way as `mpsc::Stream`, but it decrements the
/// associated `IntGauge` when it gets polled successfully.
impl<T> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        let poll = Pin::new(&mut self.value).poll_next(cx);
        if let Poll::Ready(Some(_)) = poll {
            self.gauge.dec();
        }
        poll
    }
}

/// Similar to `mpsc::channel`, `new` creates a pair of `Sender` and `Receiver`
pub fn new<T>(size: usize, gauge: &IntGauge) -> (Sender<T>, Receiver<T>) {
    gauge.set(0);
    let (sender, receiver) = mpsc::channel(size);
    (
        WithGauge {
            gauge: gauge.clone(),
            value: sender,
        },
        WithGauge {
            gauge: gauge.clone(),
            value: receiver,
        },
    )
}

lazy_static::lazy_static! {
    pub static ref TEST_COUNTER: IntGauge =
        IntGauge::new("TEST_COUNTER", "Counter of network tests").unwrap();
}

pub fn new_test<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    new(size, &TEST_COUNTER)
}
