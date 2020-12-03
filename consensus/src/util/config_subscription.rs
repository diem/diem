// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use channel::diem_channel::Receiver;
use diem_types::{
    account_config::NewEpochEvent,
    on_chain_config::{OnChainConfigPayload, ON_CHAIN_CONFIG_REGISTRY},
};
use subscription_service::ReconfigSubscription;

/// Creates consensus's subscription to reconfiguration notification from state sync
pub fn gen_consensus_reconfig_subscription(
) -> (ReconfigSubscription, Receiver<(), OnChainConfigPayload>) {
    ReconfigSubscription::subscribe_all(
        "consensus",
        ON_CHAIN_CONFIG_REGISTRY.to_vec(),
        vec![NewEpochEvent::event_key()],
    )
}
