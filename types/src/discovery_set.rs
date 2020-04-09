// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    contract_event::ContractEvent,
    discovery_info::DiscoveryInfo,
    event::{EventHandle, EventKey},
    language_storage::{StructTag, TypeTag},
    move_resource::MoveResource,
};
use anyhow::{ensure, Error, Result};
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, iter::IntoIterator, ops::Deref, vec};

pub static DISCOVERY_SET_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraSystem").unwrap());

pub static DISCOVERY_SET_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("DiscoverySet").unwrap());

pub static DISCOVERY_SET_CHANGE_EVENT_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("DiscoverySetChangeEvent").unwrap());

pub static DISCOVERY_SET_STRUCT_TAG: Lazy<StructTag> = Lazy::new(|| StructTag {
    name: DISCOVERY_SET_STRUCT_NAME.clone(),
    address: account_config::CORE_CODE_ADDRESS,
    module: DISCOVERY_SET_MODULE_NAME.clone(),
    type_params: vec![],
});

pub static DISCOVERY_SET_CHANGE_EVENT_STRUCT_TAG: Lazy<StructTag> = Lazy::new(|| StructTag {
    name: DISCOVERY_SET_CHANGE_EVENT_STRUCT_NAME.clone(),
    address: account_config::CORE_CODE_ADDRESS,
    module: DISCOVERY_SET_MODULE_NAME.clone(),
    type_params: vec![],
});

pub static DISCOVERY_SET_TYPE_TAG: Lazy<TypeTag> =
    Lazy::new(|| TypeTag::Struct(DISCOVERY_SET_STRUCT_TAG.clone()));

pub static DISCOVERY_SET_CHANGE_EVENT_TYPE_TAG: Lazy<TypeTag> =
    Lazy::new(|| TypeTag::Struct(DISCOVERY_SET_CHANGE_EVENT_STRUCT_TAG.clone()));

/// Path to the DiscoverySet resource.
pub static DISCOVERY_SET_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&*DISCOVERY_SET_STRUCT_TAG, &Accesses::empty()));

/// The path to the discovery set change event handle under a DiscoverSetResource.
pub static DISCOVERY_SET_CHANGE_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = DiscoverySetResource::resource_path();
    path.extend_from_slice(b"/change_events_count/");
    path
});

/// The AccessPath pointing to the discovery set change event handle under the system discovery set address.
pub static GLOBAL_DISCOVERY_SET_CHANGE_EVENT_PATH: Lazy<AccessPath> = Lazy::new(|| {
    AccessPath::new(
        account_config::discovery_set_address(),
        DISCOVERY_SET_CHANGE_EVENT_PATH.to_vec(),
    )
});

#[derive(Debug, Deserialize, Serialize)]
pub struct DiscoverySetResource {
    /// The current discovery set. Updated only at epoch boundaries via reconfiguration.
    discovery_set: DiscoverySet,
    /// Handle where discovery set change events are emitted
    change_events: EventHandle,
}

impl DiscoverySetResource {
    pub fn change_events(&self) -> &EventHandle {
        &self.change_events
    }

    pub fn discovery_set(&self) -> &DiscoverySet {
        &self.discovery_set
    }
}

impl MoveResource for DiscoverySetResource {
    const MODULE_NAME: &'static str = "LibraSystem";
    const STRUCT_NAME: &'static str = "DiscoverySet";
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

#[derive(Clone, Debug)]
pub struct DiscoverySetChangeEvent {
    pub event_seq_num: u64,
    pub discovery_set: DiscoverySet,
}

impl TryFrom<&ContractEvent> for DiscoverySetChangeEvent {
    type Error = Error;

    fn try_from(contract_event: &ContractEvent) -> Result<Self> {
        let event_seq_num = contract_event.sequence_number();

        ensure!(
            &*DISCOVERY_SET_CHANGE_EVENT_TYPE_TAG == contract_event.type_tag(),
            "Failed to deserialize DiscoverySetChangeEvent, unexpected type tag: {:?}, expected: {:?}",
            contract_event.type_tag(),
            &*DISCOVERY_SET_CHANGE_EVENT_TYPE_TAG,
        );

        let discovery_set = DiscoverySet::from_bytes(contract_event.event_data())?;

        Ok(DiscoverySetChangeEvent {
            event_seq_num,
            discovery_set,
        })
    }
}

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock {
    use super::*;

    use crate::on_chain_config::ValidatorSet;
    use libra_crypto::{x25519, Uniform};
    use parity_multiaddr::Multiaddr;
    use std::str::FromStr;

    pub fn mock_discovery_set(validator_set: &ValidatorSet) -> DiscoverySet {
        let mock_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
        let mock_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();

        let discovery_set = validator_set
            .payload()
            .iter()
            .map(|validator_pubkeys| DiscoveryInfo {
                account_address: *validator_pubkeys.account_address(),
                validator_network_identity_pubkey: validator_pubkeys.network_identity_public_key(),
                validator_network_address: mock_addr.clone(),
                fullnodes_network_identity_pubkey: mock_pubkey,
                fullnodes_network_address: mock_addr.clone(),
            })
            .collect::<Vec<_>>();
        DiscoverySet::new(discovery_set)
    }
}
