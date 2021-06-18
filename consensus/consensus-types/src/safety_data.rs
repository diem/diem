// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::vote::Vote;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Data structure for safety rules to ensure consensus safety.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Default)]
pub struct SafetyData {
    pub epoch: u64,
    pub last_voted_round: u64,
    // highest 2-chain round, used for 3-chain
    pub preferred_round: u64,
    // highest 1-chain round, used for 2-chain
    #[serde(default)]
    pub one_chain_round: u64,
    pub last_vote: Option<Vote>,
}

impl SafetyData {
    pub fn new(
        epoch: u64,
        last_voted_round: u64,
        preferred_round: u64,
        one_chain_round: u64,
        last_vote: Option<Vote>,
    ) -> Self {
        Self {
            epoch,
            last_voted_round,
            preferred_round,
            one_chain_round,
            last_vote,
        }
    }
}

impl fmt::Display for SafetyData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "SafetyData: [epoch: {}, last_voted_round: {}, preferred_round: {}, one_chain_round: {}]",
            self.epoch, self.last_voted_round, self.preferred_round, self.one_chain_round
        )
    }
}

#[test]
fn test_safety_data_upgrade() {
    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Default)]
    struct OldSafetyData {
        pub epoch: u64,
        pub last_voted_round: u64,
        pub preferred_round: u64,
        pub last_vote: Option<Vote>,
    }
    let old_data = OldSafetyData {
        epoch: 1,
        last_voted_round: 10,
        preferred_round: 100,
        last_vote: None,
    };
    let value = serde_json::to_value(&old_data).unwrap();
    let _: SafetyData = serde_json::from_value(value).unwrap();
}
