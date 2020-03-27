// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::validator_verifier::ValidatorVerifier;
use std::{collections::BTreeMap, fmt, sync::Arc};

#[derive(Clone, Debug)]
/// EpochInfo represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with ValidatorChangeProof.
pub struct EpochInfo {
    pub epoch: u64,
    pub verifier: Arc<ValidatorVerifier>,
}

impl EpochInfo {
    pub fn empty() -> Self {
        Self {
            epoch: 0,
            verifier: Arc::new(ValidatorVerifier::new(BTreeMap::new())),
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
