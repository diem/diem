// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Error;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_logger::Schema;
use serde::Serialize;

// TODO: Fix the leveling of these logs individually. https://github.com/libra/libra/issues/5615
#[derive(Schema)]
pub struct LogSchema<'a> {
    name: LogEntry,
    event: Option<LogEvent>,
    #[schema(display)]
    consensus_key: Option<&'a Ed25519PublicKey>,
    json_rpc_endpoint: Option<&'a str>,
    #[schema(display)]
    liveness_error: Option<&'a Error>,
    sleep_duration: Option<u64>,
    #[schema(display)]
    unexpected_error: Option<&'a Error>,
}

impl<'a> LogSchema<'a> {
    pub fn new(name: LogEntry) -> Self {
        Self {
            name,
            event: None,
            consensus_key: None,
            json_rpc_endpoint: None,
            liveness_error: None,
            sleep_duration: None,
            unexpected_error: None,
        }
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogEvent {
    Error,
    Pending,
    Success,
}
