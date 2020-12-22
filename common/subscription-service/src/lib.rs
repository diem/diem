// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Generic pub/sub service framework

use anyhow::Result;
use channel::{
    diem_channel::{self, Receiver, Sender},
    message_queues::QueueStyle,
};
use diem_types::{
    event::EventKey,
    on_chain_config::{ConfigID, OnChainConfigPayload},
};
use std::collections::HashSet;

pub struct SubscriptionService<T, U> {
    pub name: String,
    subscribed_items: T,
    sender: Sender<(), U>,
}

impl<T: Clone, U> SubscriptionService<T, U> {
    /// Constructs an subscription object for `items`
    /// Returns the subscription object, and the receiving end of a channel that subscription will be sent to
    pub fn subscribe(name: &str, items: T) -> (Self, Receiver<(), U>) {
        let (sender, receiver) = diem_channel::new(QueueStyle::LIFO, 1, None);
        (
            Self {
                name: name.to_string(),
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
    // Creates a subscription service named `name` that subscribes to changes in configs specified in `configs`
    // and emission of events specified in `events`
    // Returns (subscription service, endpoint that listens to the service)
    pub fn subscribe_all(
        name: &str,
        configs: Vec<ConfigID>,
        events: Vec<EventKey>,
    ) -> (Self, Receiver<(), OnChainConfigPayload>) {
        let bundle = SubscriptionBundle::new(configs, events);
        Self::subscribe(name, bundle)
    }
}
