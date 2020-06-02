// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::libra_channel::Receiver;
use libra_types::{
    account_config::NewEpochEvent,
    on_chain_config::{
        OnChainConfigPayload, ReconfigSubscription, SubscriptionBundle, ON_CHAIN_CONFIG_REGISTRY,
    },
};

/// Creates consensus's subscription to reconfiguration notification from state sync
pub fn gen_consensus_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    let bundle = SubscriptionBundle::new(
        ON_CHAIN_CONFIG_REGISTRY.to_vec(),
        vec![NewEpochEvent::event_key()],
    );
    ReconfigSubscription::subscribe(bundle)
}
