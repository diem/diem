// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    discovery_info::DiscoveryInfo,
    event::{EventHandle, EventKey},
    language_storage::StructTag,
    validator_set::validator_set_module_name,
};
use anyhow::Result;
use move_core_types::identifier::{IdentStr, Identifier};
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
        address: account_config::CORE_CODE_ADDRESS,
        name: discovery_set_struct_name().to_owned(),
        module: discovery_set_module_name().to_owned(),
        type_params: vec![],
    }
}

/// Path to the DiscoverySet resource.
pub static DISCOVERY_SET_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&discovery_set_tag(), &Accesses::empty()));

/// The path to the discovery set change event handle under a DiscoverSetResource.
pub static DISCOVERY_SET_CHANGE_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = DISCOVERY_SET_RESOURCE_PATH.to_vec();
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

#[cfg(any(test, feature = "fuzzing"))]
pub mod mock {
    use super::*;

    use crate::crypto_proxies::ValidatorSet;
    use libra_crypto::x25519::X25519StaticPrivateKey;
    use parity_multiaddr::Multiaddr;
    use std::str::FromStr;

    pub fn mock_discovery_set(validator_set: &ValidatorSet) -> DiscoverySet {
        let salt = None;
        let seed = [69u8; 32];
        let app_info = None;
        let (_, mock_pubkey) =
            X25519StaticPrivateKey::derive_keypair_from_seed(salt, &seed, app_info);
        let mock_addr = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();

        let discovery_set = validator_set
            .iter()
            .map(|validator_pubkeys| DiscoveryInfo {
                account_address: *validator_pubkeys.account_address(),
                validator_network_identity_pubkey: validator_pubkeys
                    .network_identity_public_key()
                    .clone(),
                validator_network_address: mock_addr.clone(),
                fullnodes_network_identity_pubkey: mock_pubkey.clone(),
                fullnodes_network_address: mock_addr.clone(),
            })
            .collect::<Vec<_>>();
        DiscoverySet::new(discovery_set)
    }
}
