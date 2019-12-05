// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config,
    event::EventKey,
    identifier::{IdentStr, Identifier},
    validator_set::validator_set_module_name,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref DISCOVERY_SET_STRUCT_NAME: Identifier = Identifier::new("DiscoverySet").unwrap();
}

pub fn discovery_set_module_name() -> &'static IdentStr {
    validator_set_module_name()
}

pub fn discovery_set_struct_name() -> &'static IdentStr {
    &*DISCOVERY_SET_STRUCT_NAME
}

// TODO(philiphayes): actually implement this
pub struct DiscoverySet;

impl DiscoverySet {
    pub fn change_event_key() -> EventKey {
        EventKey::new_from_address(&account_config::discovery_set_address(), 2)
    }
}
