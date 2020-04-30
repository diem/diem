// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{on_chain_config::OnChainConfig, validator_info::ValidatorInfo};
use anyhow::{Error, Result};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, iter::IntoIterator, vec};

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

impl OnChainConfig for ValidatorSet {
    // validator_set_address
    const ADDRESS: &'static str = "0x1D8";
    const IDENTIFIER: &'static str = "LibraSystem";
}

impl IntoIterator for ValidatorSet {
    type Item = ValidatorInfo;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.payload.into_iter()
    }
}
