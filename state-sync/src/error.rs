// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::Error::UnexpectedError;
use diem_types::transaction::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Failed to send callback: {0}")]
    CallbackSendFailed(String),
    #[error("No transactions were committed, but received a commit notification!")]
    NoTransactionsCommitted,
    #[error("Received an old sync request for version {0}, but our known version is: {1}")]
    OldSyncRequestVersion(Version, Version),
    #[error("Synced beyond the target version. Synced version: {0}, target version: {1}")]
    SyncedBeyondTarget(Version, Version),
    #[error("State sync is uninitialized! Error: {0}")]
    UninitializedError(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

// TODO(joshlind): remove this once we move from anyhow error to thiserror in state sync!
impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        UnexpectedError(format!("{}", error))
    }
}
