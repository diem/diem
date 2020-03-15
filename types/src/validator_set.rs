// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    event::{EventHandle, EventKey},
    language_storage::StructTag,
    validator_public_keys::ValidatorPublicKeys,
};
use anyhow::{Error, Result};
use libra_crypto::VerifyingKey;
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
    iter::IntoIterator,
    ops::Deref,
    vec,
};

static LIBRA_SYSTEM_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("LibraSystem").unwrap());
static VALIDATOR_SET_STRUCT_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ValidatorSet").unwrap());

pub fn validator_set_module_name() -> &'static IdentStr {
    &*LIBRA_SYSTEM_MODULE_NAME
}

pub fn validator_set_struct_name() -> &'static IdentStr {
    &*VALIDATOR_SET_STRUCT_NAME
}

pub fn validator_set_tag() -> StructTag {
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        name: validator_set_struct_name().to_owned(),
        module: validator_set_module_name().to_owned(),
        type_params: vec![],
    }
}

/// The access path where the Validator Set resource is stored.
pub static VALIDATOR_SET_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&validator_set_tag(), &Accesses::empty()));

/// The path to the validator set change event handle under a ValidatorSetResource.
pub static VALIDATOR_SET_CHANGE_EVENT_PATH: Lazy<Vec<u8>> = Lazy::new(|| {
    let mut path = VALIDATOR_SET_RESOURCE_PATH.to_vec();
    path.extend_from_slice(b"/change_events_count/");
    path
});

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorSetResource<PublicKey> {
    validator_set: ValidatorSet<PublicKey>,
    last_reconfiguration_time: u64,
    change_events: EventHandle,
}

impl<PublicKey> ValidatorSetResource<PublicKey> {
    pub fn change_events(&self) -> &EventHandle {
        &self.change_events
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorSet<PublicKey>(Vec<ValidatorPublicKeys<PublicKey>>);

impl<PublicKey> fmt::Display for ValidatorSet<PublicKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for validator in &self.0 {
            write!(f, "{} ", validator)?;
        }
        write!(f, "]")
    }
}

impl<PublicKey: VerifyingKey> ValidatorSet<PublicKey> {
    /// Constructs a ValidatorSet resource.
    pub fn new(payload: Vec<ValidatorPublicKeys<PublicKey>>) -> Self {
        ValidatorSet(payload)
    }

    pub fn empty() -> Self {
        ValidatorSet::new(Vec::new())
    }

    pub fn change_event_key() -> EventKey {
        EventKey::new_from_address(&account_config::validator_set_address(), 2)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        lcs::from_bytes(bytes).map_err(Into::into)
    }
}

impl<PublicKey> Deref for ValidatorSet<PublicKey> {
    type Target = [ValidatorPublicKeys<PublicKey>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<PublicKey> IntoIterator for ValidatorSet<PublicKey> {
    type Item = ValidatorPublicKeys<PublicKey>;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<PublicKey: VerifyingKey> TryFrom<crate::proto::types::ValidatorSet>
    for ValidatorSet<PublicKey>
{
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorSet) -> Result<Self> {
        Ok(ValidatorSet::new(
            proto
                .validator_public_keys
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>>>()?,
        ))
    }
}

impl<PublicKey: VerifyingKey> From<ValidatorSet<PublicKey>> for crate::proto::types::ValidatorSet {
    fn from(set: ValidatorSet<PublicKey>) -> Self {
        Self {
            validator_public_keys: set.0.into_iter().map(Into::into).collect(),
        }
    }
}
