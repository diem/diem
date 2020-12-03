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
    Epoch,
    Initialize,
    KeyReconciliation,
    LastVotedRound,
    PreferredRound,
    SignProposal,
    SignTimeout,
    State,
    Waypoint,
}

impl LogEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEntry::ConsensusState => "consensus_state",
            LogEntry::ConstructAndSignVote => "construct_and_sign_vote",
            LogEntry::Epoch => "epoch",
            LogEntry::Initialize => "initialize",
            LogEntry::LastVotedRound => "last_voted_round",
            LogEntry::KeyReconciliation => "key_reconciliation",
            LogEntry::PreferredRound => "preferred_round",
            LogEntry::SignProposal => "sign_proposal",
            LogEntry::SignTimeout => "sign_timeout",
            LogEntry::State => "state",
            LogEntry::Waypoint => "waypoint",
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
