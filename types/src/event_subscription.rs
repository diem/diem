// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::{ConfigID, OnChainConfigPayload};
use anyhow::Result;
use channel::{
    libra_channel::{self, Receiver, Sender},
    message_queues::QueueStyle,
};
use std::{collections::HashSet, num::NonZeroUsize};

/// A subscription to on-chain reconfiguration notifications from state sync
pub struct ReconfigSubscription {
    sender: Sender<(), OnChainConfigPayload>,
    subscribed_configs: HashSet<ConfigID>,
}

impl ReconfigSubscription {
    /// Constructs an reconfig subscription object for a set of configs `subscribed_configs`
    /// Returns the subscription object, and the receiving end of a channel that reconfig notifications
    /// will be sent to
    pub fn subscribe(
        subscribed_configs: &[ConfigID],
    ) -> (Self, Receiver<(), OnChainConfigPayload>) {
        let (sender, receiver) =
            libra_channel::new(QueueStyle::LIFO, NonZeroUsize::new(1).unwrap(), None);
        let subscribed_configs = subscribed_configs.iter().cloned().collect::<HashSet<_>>();
        (
            Self {
                sender,
                subscribed_configs,
            },
            receiver,
        )
    }

    /// Function used by reconfiguration notification publisher to send notification to subscriber
    pub fn publish(&mut self, payload: OnChainConfigPayload) -> Result<()> {
        self.sender.push((), payload)
    }

    pub fn subscribed_configs(&self) -> HashSet<ConfigID> {
        self.subscribed_configs.clone()
    }
}
