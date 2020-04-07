// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    event::{EventHandle, EventKey},
    language_storage::StructTag,
    validator_info::ValidatorInfo,
};
use anyhow::{Error, Result};
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, vec};

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
pub struct ValidatorSetResource {
    scheme: ConsensusScheme,
    validators: Vec<ValidatorInfo>,
    last_reconfiguration_time: u64,
    change_events: EventHandle,
}

#[cfg(any(test, feature = "fuzzing"))]
impl Default for ValidatorSetResource {
    fn default() -> Self {
        ValidatorSetResource {
            scheme: ConsensusScheme::Ed25519,
            validators: vec![],
            last_reconfiguration_time: 0,
            change_events: EventHandle::new(ValidatorSet::change_event_key(), 0),
        }
    }
}

impl ValidatorSetResource {
    pub fn change_events(&self) -> &EventHandle {
        &self.change_events
    }

    pub fn last_reconfiguration_time(&self) -> u64 {
        self.last_reconfiguration_time
    }

    pub fn validator_set(&self) -> ValidatorSet {
        assert_eq!(self.scheme, ConsensusScheme::Ed25519);
        ValidatorSet::new(self.validators.clone())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[repr(u8)]
pub enum ConsensusScheme {
    Ed25519 = 0,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorSet {
    scheme: ConsensusScheme,
    payload: Vec<ValidatorInfo>,
}

impl fmt::Display for ValidatorSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for validator in self.payload().iter() {
            write!(f, "{} ", validator)?;
        }
        write!(f, "]")
    }
}

impl ValidatorSet {
    /// Constructs a ValidatorSet resource.
    pub fn new(payload: Vec<ValidatorInfo>) -> Self {
        Self {
            scheme: ConsensusScheme::Ed25519,
            payload,
        }
    }

    pub fn scheme(&self) -> ConsensusScheme {
        self.scheme
    }

    pub fn payload(&self) -> &[ValidatorInfo] {
        &self.payload
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

impl TryFrom<crate::proto::types::ValidatorSet> for ValidatorSet {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorSet) -> Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl From<ValidatorSet> for crate::proto::types::ValidatorSet {
    fn from(set: ValidatorSet) -> Self {
        Self {
            bytes: lcs::to_bytes(&set).expect("failed to serialize validator set"),
        }
    }
}
