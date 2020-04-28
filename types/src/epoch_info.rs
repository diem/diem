// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_verifier::ValidatorVerifier;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

/// EpochInfo represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with EpochChangeProof.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct EpochInfo {
    pub epoch: u64,
    pub verifier: ValidatorVerifier,
}

impl EpochInfo {
    pub fn empty() -> Self {
        Self {
            epoch: 0,
            verifier: ValidatorVerifier::new(BTreeMap::new()),
        }
    }
}

impl fmt::Display for EpochInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EpochInfo [epoch: {}, validator: {}]",
            self.epoch, self.verifier
        )
    }
}
