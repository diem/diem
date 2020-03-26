// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::on_chain_config::{ConfigID, OnChainConfigPayload};
use std::collections::HashSet;

/// Trait that must be implemented by a subscription to reconfiguration events emitted
/// by state sync
pub trait EventSubscription: Send + Sync {
    fn subscribed_configs(&self) -> HashSet<ConfigID>;
    fn publish(&mut self, payload: OnChainConfigPayload);
}
