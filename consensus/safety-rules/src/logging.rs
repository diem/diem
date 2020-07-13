// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::StructuredLogEntry;

pub fn safety_log(entry: LogEntry, event: LogEvent) -> StructuredLogEntry {
    StructuredLogEntry::new_named("safety_rules", entry.as_str())
        .data(LogField::Event.as_str(), event.as_str())
}

#[derive(Clone, Copy)]
pub enum LogEntry {
    ConsensusState,
    ConstructAndSignVote,
    Initialize,
    SignProposal,
    SignTimeout,
}

impl LogEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEntry::ConsensusState => "consensus_state",
            LogEntry::ConstructAndSignVote => "construct_and_sign_vote",
            LogEntry::Initialize => "initialize",
            LogEntry::SignProposal => "sign_proposal",
            LogEntry::SignTimeout => "sign_timeout",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LogEvent {
    Error,
    Request,
    Success,
}

impl LogEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEvent::Error => "error",
            LogEvent::Request => "request",
            LogEvent::Success => "success",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LogField {
    Event,
    Message,
    Round,
}

impl LogField {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogField::Event => "event",
            LogField::Message => "message",
            LogField::Round => "round",
        }
    }
}
