// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid key value found in backend: {0}")]
    BackendInvalidKeyValue(String),
    #[error("Backend is missing the backend key")]
    BackendMissingBackendKey,
    #[error("Backend parsing error: {0}")]
    BackendParsingError(String),
    #[error("Local storage unavailable, please check your configuration")]
    LocalStorageUnavailable,
    #[error("Failed to read from local storage: {0}")]
    LocalStorageReadError(String),
    #[error("Failed to sign data using local storage: {0}")]
    LocalStorageSigningError(String),
    #[error("Failed to read from remote storage: {0}")]
    RemoteStorageReadError(String),
    #[error("Failed to write to remote storage: {0}")]
    RemoteStorageWriteError(String),
    #[error("Remote storage unavailable, please check your configuration")]
    RemoteStorageUnavailable,
    #[error("Unexpected command, expected {0}, found {1}")]
    UnexpectedCommand(String, String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
