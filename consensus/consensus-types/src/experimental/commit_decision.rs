// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::Round;
use anyhow::Context;
use diem_types::{ledger_info::LedgerInfoWithSignatures, validator_verifier::ValidatorVerifier};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct CommitDecision {
    ledger_info: LedgerInfoWithSignatures,
    // TOOD: we can include a full state here.
}

// this is required by structured log
impl Debug for CommitDecision {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for CommitDecision {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "CommitDecision: [{}]", self.ledger_info)
    }
}

impl CommitDecision {
    /// Generates a new CommitDecision
    pub fn new(ledger_info: LedgerInfoWithSignatures) -> Self {
        Self { ledger_info }
    }

    pub fn round(&self) -> Round {
        self.ledger_info.ledger_info().round()
    }

    pub fn epoch(&self) -> u64 {
        self.ledger_info.ledger_info().epoch()
    }

    /// Return the LedgerInfo associated with this commit proposal
    pub fn ledger_info(&self) -> &LedgerInfoWithSignatures {
        &self.ledger_info
    }

    /// Verifies that the signatures carried in the message forms a valid quorum,
    /// and then verifies the signature.
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        // We do not need to check the author because as long as the signature tree
        // is valid, the message should be valid.
        self.ledger_info
            .verify_signatures(validator)
            .context("Failed to verify Commit Decision")
    }
}
