// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Generic pub/sub service framework

use anyhow::Result;
use channel::{
    libra_channel::{self, Receiver, Sender},
    message_queues::QueueStyle,
};
use std::num::NonZeroUsize;

pub struct SubscriptionService<T, U> {
    subscribed_items: T,
    sender: Sender<(), U>,
}

impl<T: Clone, U> SubscriptionService<T, U> {
    /// Constructs an subscription object for `items`
    /// Returns the subscription object, and the receiving end of a channel that subscription will be sent to
    pub fn subscribe(items: T) -> (Self, Receiver<(), U>) {
        let (sender, receiver) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        (
            Self {
                sender,
                subscribed_items: items,
            },
            receiver,
        )
    }

    pub fn publish(&mut self, payload: U) -> Result<()> {
        self.sender.push((), payload)
    }

    pub fn subscribed_items(&self) -> T {
        self.subscribed_items.clone()
    }
}
