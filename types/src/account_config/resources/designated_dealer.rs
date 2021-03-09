// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config::{PreburnQueueResource, PreburnResource},
    event::EventHandle,
};
use move_core_types::{identifier::Identifier, move_resource::MoveResource};
use serde::{Deserialize, Serialize};
use std::collections::btree_map::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct DesignatedDealer {
    received_mint_events: EventHandle,
}

impl DesignatedDealer {
    /// Return the received_mint_events handle for the given DesignatedDealer
    pub fn received_mint_events(&self) -> &EventHandle {
        &self.received_mint_events
    }
}

impl MoveResource for DesignatedDealer {
    const MODULE_NAME: &'static str = "DesignatedDealer";
    const STRUCT_NAME: &'static str = "Dealer";
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DesignatedDealerPreburns {
    Preburn(BTreeMap<Identifier, PreburnResource>),
    PreburnQueue(BTreeMap<Identifier, PreburnQueueResource>),
}
