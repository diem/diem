// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::Round;
use std::fmt::{Debug, Display, Formatter};
use serde::{Deserialize, Serialize};
use diem_types::ledger_info::LedgerInfoWithSignatures;

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
        write!(
            f,
            "CommitProposal: [{}]",
            self.ledger_info
        )
    }
}

impl CommitDecision {
    /// Generates a new CommitDecision
    pub fn new(
        ledger_info: LedgerInfoWithSignatures,
    ) -> Self {
        Self {
            ledger_info
        }
    }

    pub fn round(&self) -> Round { self.ledger_info.ledger_info().round() }

    pub fn epoch(&self) -> u64 { self.ledger_info.ledger_info().epoch() }

    /// Return the LedgerInfo associated with this commit proposal
    pub fn ledger_info(&self) -> &LedgerInfoWithSignatures {
        &self.ledger_info
    }

}
