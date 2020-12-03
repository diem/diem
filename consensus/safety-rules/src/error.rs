// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Error, PartialEq, Serialize)]
/// Different reasons for proposal rejection
pub enum Error {
    #[error("Provided epoch, {0}, does not match expected epoch, {1}")]
    IncorrectEpoch(u64, u64),
    #[error("block has next round that wraps around: {0}")]
    IncorrectRound(u64),
    #[error("Provided round, {0}, is incompatible with last voted round, {1}")]
    IncorrectLastVotedRound(u64, u64),
    #[error("Provided round, {0}, is incompatible with preferred round, {1}")]
    IncorrectPreferredRound(u64, u64),
    #[error("Unable to verify that the new tree extends the parent: {0}")]
    InvalidAccumulatorExtension(String),
    #[error("Invalid EpochChangeProof: {0}")]
    InvalidEpochChangeProof(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("No next_epoch_state specified in the provided Ledger Info")]
    InvalidLedgerInfo,
    #[error("Invalid proposal: {0}")]
    InvalidProposal(String),
    #[error("Invalid QC: {0}")]
    InvalidQuorumCertificate(String),
    #[error("{0} is not set, SafetyRules is not initialized")]
    NotInitialized(String),
    #[error("Data not found in secure storage: {0}")]
    SecureStorageMissingDataError(String),
    #[error("Unexpected error returned by secure storage: {0}")]
    SecureStorageUnexpectedError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Validator key not found: {0}")]
    ValidatorKeyNotFound(String),
    #[error("The validator is not in the validator set. Address not in set: {0}")]
    ValidatorNotInSet(String),
    #[error("Vote proposal missing expected signature")]
    VoteProposalSignatureNotFound,
}

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<diem_secure_net::Error> for Error {
    fn from(error: diem_secure_net::Error) -> Self {
        Self::InternalError(error.to_string())
    }
}

impl From<diem_secure_storage::Error> for Error {
    fn from(error: diem_secure_storage::Error) -> Self {
        match error {
            diem_secure_storage::Error::PermissionDenied => {
                // If a storage error is thrown that indicates a permission failure, we
                // want to panic immediately to alert an operator that something has gone
                // wrong. For example, this error is thrown when a storage (e.g., vault)
                // token has expired, so it makes sense to fail fast and require a token
                // renewal!
                panic!(
                    "A permission error was thrown: {:?}. Maybe the storage token needs to be renewed?",
                    error
                );
            }
            diem_secure_storage::Error::KeyVersionNotFound(_, _)
            | diem_secure_storage::Error::KeyNotSet(_) => {
                Self::SecureStorageMissingDataError(error.to_string())
            }
            _ => Self::SecureStorageUnexpectedError(error.to_string()),
        }
    }
}
