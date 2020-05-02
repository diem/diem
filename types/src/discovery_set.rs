// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_config,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    discovery_info::DiscoveryInfo,
    event::{EventHandle, EventKey},
    move_resource::MoveResource,
};
use anyhow::{format_err, Error, Result};
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, iter::IntoIterator, ops::Deref, vec};

/// The path to the discovery set change event handle under a DiscoverSetResource.
pub static DISCOVERY_SET_CHANGE_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = DiscoverySetResource::resource_path();
    path.extend_from_slice(b"/change_events_count/");
    path
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

impl TryFrom<&AccountStateBlob> for DiscoverySetResource {
    type Error = Error;

    fn try_from(blob: &AccountStateBlob) -> Result<DiscoverySetResource> {
        AccountState::try_from(blob)?
            .get_discovery_set_resource()?
            .ok_or_else(|| format_err!("discovery set resource cannot be missing"))
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

impl From<DiscoverySetResource> for DiscoverySet {
    fn from(resource: DiscoverySetResource) -> Self {
        resource.discovery_set
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

    use crate::on_chain_config::ValidatorSet;
    use libra_crypto::{x25519, Uniform};
    use libra_network_address::{NetworkAddress, RawNetworkAddress};
    use std::str::FromStr;

    pub fn mock_discovery_set(validator_set: &ValidatorSet) -> DiscoverySet {
        let mock_pubkey = x25519::PrivateKey::generate_for_testing().public_key();
        let mock_addr = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let mock_addr = RawNetworkAddress::try_from(&mock_addr).unwrap();

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
