// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::sink::buffered_send::BufferedSend;
use futures::sink::Sink;

mod buffered_send;

// blanket trait impl for NetworkSinkExt
impl<T: ?Sized, Item> NetworkSinkExt<Item> for T where T: Sink<Item> {}

/// Extension trait for [`Sink`] that provides network crate specific combinator
/// functions.
pub trait NetworkSinkExt<Item>: Sink<Item> {
    /// Like `sink.send()` but without the mandatory flush.
    ///
    /// Specifically, `sink.send()` will do `sink.poll_ready()` then
    /// `sink.start_send(item)` and finally a mandatory `sink.poll_flush()`.
    /// This will only do `sink.poll_ready()` and `sink.start_send(item)`.
    fn buffered_send(&mut self, item: Item) -> BufferedSend<'_, Self, Item>
    where
        Self: Unpin,
    {
        BufferedSend::new(self, item)
    }
}
