// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_logger::StructuredLogEntry;

pub fn key_manager_log(entry: LogEntry) -> StructuredLogEntry {
    StructuredLogEntry::new_named("key_manager", entry.as_str())
}

#[derive(Clone, Copy)]
pub enum LogEntry {
    CheckKeyStatus,
    Initialized,
    FullKeyRotation,
    KeyRotatedInStorage,
    TransactionSubmission,
    NoAction,
    Sleep,
    WaitForReconfiguration,
}

impl LogEntry {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEntry::CheckKeyStatus => "check_key_status",
            LogEntry::Initialized => "initialized",
            LogEntry::FullKeyRotation => "full_key_rotation",
            LogEntry::KeyRotatedInStorage => "key_rotated_in_storage",
            LogEntry::TransactionSubmission => "transaction_submission",
            LogEntry::NoAction => "no_action",
            LogEntry::Sleep => "sleep_called",
            LogEntry::WaitForReconfiguration => "wait_for_reconfiguration",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LogEvent {
    Error,
    Pending,
    Success,
}

impl LogEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogEvent::Error => "error",
            LogEvent::Pending => "pending",
            LogEvent::Success => "success",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LogField {
    ConsensusKey,
    Event,
    JsonRpcEndpoint,
    LivenessError,
    SleepDuration,
    UnexpectedError,
}

impl LogField {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogField::ConsensusKey => "consensus_public_key",
            LogField::Event => "event",
            LogField::JsonRpcEndpoint => "json_rpc_endpoint",
            LogField::LivenessError => "liveness_error",
            LogField::SleepDuration => "sleep_duration",
            LogField::UnexpectedError => "unexpected_error",
        }
    }
}
