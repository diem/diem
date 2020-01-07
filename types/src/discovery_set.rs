// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    discovery_info::DiscoveryInfo,
    event::EventKey,
    identifier::{IdentStr, Identifier},
    language_storage::StructTag,
    validator_set::validator_set_module_name,
};
use anyhow::Result;
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{iter::IntoIterator, ops::Deref, vec};

static DISCOVERY_SET_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("DiscoverySet").unwrap());

pub fn discovery_set_module_name() -> &'static IdentStr {
    validator_set_module_name()
}

pub fn discovery_set_struct_name() -> &'static IdentStr {
    &*DISCOVERY_SET_STRUCT_NAME
}

pub fn discovery_set_tag() -> StructTag {
    StructTag {
        name: discovery_set_struct_name().to_owned(),
        address: account_config::core_code_address(),
        module: discovery_set_module_name().to_owned(),
        type_params: vec![],
    }
}

#[allow(dead_code)]
pub(crate) fn discovery_set_path() -> Vec<u8> {
    AccessPath::resource_access_vec(&discovery_set_tag(), &Accesses::empty())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct DiscoverySet(Vec<DiscoveryInfo>);

impl DiscoverySet {
    pub fn new(discovery_set: Vec<DiscoveryInfo>) -> Self {
        Self(discovery_set)
    }

    pub fn change_event_key() -> EventKey {
        EventKey::new_from_address(&account_config::discovery_set_address(), 2)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl Deref for DiscoverySet {
    type Target = [DiscoveryInfo];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for DiscoverySet {
    type Item = DiscoveryInfo;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
