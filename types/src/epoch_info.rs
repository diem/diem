// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    epoch_change::Verifier,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_verifier::ValidatorVerifier,
};
use anyhow::ensure;
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

impl Verifier for EpochInfo {
    fn verify(&self, ledger_info: &LedgerInfoWithSignatures) -> anyhow::Result<()> {
        ensure!(
            self.epoch == ledger_info.ledger_info().epoch(),
            "LedgerInfo has unexpected epoch {}, expected {}",
            ledger_info.ledger_info().epoch(),
            self.epoch
        );
        ledger_info.verify_signatures(&self.verifier)?;
        Ok(())
    }

    fn epoch_change_verification_required(&self, epoch: u64) -> bool {
        self.epoch < epoch
    }

    fn is_ledger_info_stale(&self, ledger_info: &LedgerInfo) -> bool {
        ledger_info.epoch() < self.epoch
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
