// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vote::Vote;
use serde::{Deserialize, Serialize};

/// Data structure for safety rules to ensure consensus safety.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Default)]
pub struct SafetyData {
    pub epoch: u64,
    pub last_voted_round: u64,
    pub preferred_round: u64,
    pub last_vote: Option<Vote>,
}

impl SafetyData {
    pub fn new(
        epoch: u64,
        last_voted_round: u64,
        preferred_round: u64,
        last_vote: Option<Vote>,
    ) -> Self {
        Self {
            epoch,
            last_voted_round,
            preferred_round,
            last_vote,
        }
    }
}
