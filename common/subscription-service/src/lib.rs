// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Generic pub/sub service framework

use anyhow::Result;
use channel::{
    libra_channel::{self, Receiver, Sender},
    message_queues::QueueStyle,
};
use libra_types::on_chain_config::{ConfigID, OnChainConfigPayload};
use std::{collections::HashSet, num::NonZeroUsize};

pub struct SubscriptionService<T, U> {
    subscribed_items: HashSet<T>,
    sender: Sender<(), U>,
}

/// A subscription service for on-chain reconfiguration notifications from state sync
/// TODO move this to on-chain config crate when ready
pub type ReconfigSubscription = SubscriptionService<ConfigID, OnChainConfigPayload>;
impl SubscriptionService<ConfigID, OnChainConfigPayload> {
    /// Constructs an reconfig subscription object for a set of configs `subscribed_configs`
    /// Returns the subscription object, and the receiving end of a channel that reconfig notifications
    /// will be sent to
    pub fn subscribe(
        subscribed_configs: &[ConfigID],
    ) -> (Self, Receiver<(), OnChainConfigPayload>) {
        let (sender, receiver) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let subscribed_items = subscribed_configs.iter().cloned().collect::<HashSet<_>>();
        (
            Self {
                sender,
                subscribed_items,
            },
            receiver,
        )
    }

    pub fn publish(&mut self, payload: OnChainConfigPayload) -> Result<()> {
        self.sender.push((), payload)
    }

    pub fn subscribed_configs(&self) -> HashSet<ConfigID> {
        self.subscribed_items.clone()
    }
}
