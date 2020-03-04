// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::contract_event::ContractEvent;

/// Trait that must be implemented by a subscription to reconfiguration events emitted
/// by state sync
pub trait EventSubscription: Send + Sync {
    fn publish(&mut self, payload: ContractEvent);
}
