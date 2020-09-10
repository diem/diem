// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Generic pub/sub service framework

use anyhow::Result;
use channel::{
    libra_channel::{self, Receiver, Sender},
    message_queues::QueueStyle,
};
use libra_types::{
    event::EventKey,
    on_chain_config::{ConfigID, OnChainConfigPayload},
};
use std::{collections::HashSet, num::NonZeroUsize};

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

/// A subscription service for on-chain reconfiguration notifications from state sync
/// This is declared/implemented here instead of in `types/on_chain_config` because
/// when `subscription_service` crate is a dependency of `types`, the build-dev fails
pub type ReconfigSubscription = SubscriptionService<SubscriptionBundle, OnChainConfigPayload>;

#[derive(Clone)]
pub struct SubscriptionBundle {
    pub configs: HashSet<ConfigID>,
    pub events: HashSet<EventKey>,
}

impl SubscriptionBundle {
    pub fn new(configs: Vec<ConfigID>, events: Vec<EventKey>) -> Self {
        let configs = configs.into_iter().collect::<HashSet<_>>();
        let events = events.into_iter().collect::<HashSet<_>>();

        Self { configs, events }
    }
}

impl ReconfigSubscription {
    pub fn subscribe_all(
        configs: Vec<ConfigID>,
        events: Vec<EventKey>,
    ) -> (Self, Receiver<(), OnChainConfigPayload>) {
        let bundle = SubscriptionBundle::new(configs, events);
        Self::subscribe(bundle)
    }
}
