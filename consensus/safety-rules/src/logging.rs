// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use consensus_types::common::{Author, Round};
use diem_logger::Schema;
use diem_types::waypoint::Waypoint;
use serde::Serialize;

#[derive(Schema)]
pub struct SafetyLogSchema<'a> {
    name: LogEntry,
    event: LogEvent,
    round: Option<Round>,
    preferred_round: Option<u64>,
    last_voted_round: Option<u64>,
    epoch: Option<u64>,
    #[schema(display)]
    error: Option<&'a Error>,
    waypoint: Option<Waypoint>,
    author: Option<Author>,
}

impl<'a> SafetyLogSchema<'a> {
    pub fn new(name: LogEntry, event: LogEvent) -> Self {
        Self {
            name,
            event,
            round: None,
            preferred_round: None,
            last_voted_round: None,
            epoch: None,
            error: None,
            waypoint: None,
            author: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEntry {
    ConsensusState,
    ConstructAndSignVote,
    ConstructAndSignVoteTwoChain,
    Epoch,
    Initialize,
    KeyReconciliation,
    LastVotedRound,
    OneChainRound,
    PreferredRound,
    SignProposal,
    SignTimeout,
    SignTimeoutWithQC,
    State,
    Waypoint,
    SignCommitVote,
}

impl LogEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEntry::ConsensusState => "consensus_state",
            LogEntry::ConstructAndSignVote => "construct_and_sign_vote",
            LogEntry::ConstructAndSignVoteTwoChain => "construct_and_sign_vote_2chain",
            LogEntry::Epoch => "epoch",
            LogEntry::Initialize => "initialize",
            LogEntry::LastVotedRound => "last_voted_round",
            LogEntry::KeyReconciliation => "key_reconciliation",
            LogEntry::OneChainRound => "one_chain_round",
            LogEntry::PreferredRound => "preferred_round",
            LogEntry::SignProposal => "sign_proposal",
            LogEntry::SignTimeout => "sign_timeout",
            LogEntry::SignTimeoutWithQC => "sign_timeout_with_qc",
            LogEntry::State => "state",
            LogEntry::Waypoint => "waypoint",
            LogEntry::SignCommitVote => "sign_commit_vote",
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    Error,
    Request,
    Success,
    Update,
}
