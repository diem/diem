// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::health::{Event, HealthCheck, HealthCheckContext, ValidatorEvent};
use std::collections::{hash_map::Entry, HashMap, HashSet};

/// Verifies that commit history produced by validators is 'lineariazble'
/// This means that validators can be behind each other, but commits that they are producing
/// do not contradict each other
pub struct CommitHistoryHealthCheck {
    round_to_commit: HashMap<u64, CommitAndValidators>,
    latest_committed_round: HashMap<String, u64>,
}

struct CommitAndValidators {
    pub hash: String,
    pub validators: HashSet<String>,
}

impl CommitHistoryHealthCheck {
    pub fn new() -> Self {
        Self {
            round_to_commit: HashMap::new(),
            latest_committed_round: HashMap::new(),
        }
    }
}

impl HealthCheck for CommitHistoryHealthCheck {
    fn on_event(&mut self, ve: &ValidatorEvent, ctx: &mut HealthCheckContext) {
        let commit = if let Event::Commit(ref commit) = ve.event {
            commit
        } else {
            return;
        };
        let round_to_commit = self.round_to_commit.entry(commit.round);
        match round_to_commit {
            Entry::Occupied(mut oe) => {
                let commit_and_validators = oe.get_mut();
                if commit_and_validators.hash != commit.commit {
                    ctx.report_failure(
                        ve.validator.clone(),
                        format!(
                            "produced contradicting commit {} at round {}, expected: {}",
                            commit.commit, commit.round, commit_and_validators.hash
                        ),
                    );
                } else {
                    commit_and_validators
                        .validators
                        .insert(ve.validator.clone());
                }
            }
            Entry::Vacant(va) => {
                let mut validators = HashSet::new();
                validators.insert(ve.validator.clone());
                va.insert(CommitAndValidators {
                    hash: commit.commit.clone(),
                    validators,
                });
            }
        }
        let latest_committed_round = self.latest_committed_round.entry(ve.validator.clone());
        match latest_committed_round {
            Entry::Occupied(mut oe) => {
                let previous_round = *oe.get();
                if previous_round > commit.round {
                    ctx.report_failure(
                        ve.validator.clone(),
                        format!(
                            "committed round {} after committing {}",
                            commit.round, previous_round
                        ),
                    );
                }
                oe.insert(commit.round);
            }
            Entry::Vacant(va) => {
                va.insert(commit.round);
            }
        }
        if let Some(min_round) = self.latest_committed_round.values().min() {
            self.round_to_commit.retain(|k, _v| *k >= *min_round);
        }
    }

    fn clear(&mut self) {
        self.round_to_commit.clear();
        self.latest_committed_round.clear();
    }

    fn name(&self) -> &'static str {
        "commit_check"
    }
}
