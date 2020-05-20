// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::CommandName;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid key value found in backend: {0}")]
    BackendInvalidKeyValue(String),
    #[error("Backend is missing the backend key")]
    BackendMissingBackendKey,
    #[error("Backend parsing error: {0}")]
    BackendParsingError(String),
    #[error("Local storage unavailable, please check your configuration: {0}")]
    LocalStorageUnavailable(String),
    #[error("Failed to read, {0}, from local storage: {1}")]
    LocalStorageReadError(&'static str, String),
    #[error("Failed to sign {0} with {1} using local storage: {2}")]
    LocalStorageSigningError(&'static str, &'static str, String),
    #[error("Failed to read, {0}, from remote storage: {0}")]
    RemoteStorageReadError(&'static str, String),
    #[error("Failed to write, {0}, to remote storage: {0}")]
    RemoteStorageWriteError(&'static str, String),
    #[error("Remote storage unavailable, please check your configuration: {0}")]
    RemoteStorageUnavailable(String),
    #[error("Unexpected command, expected {0}, found {1}")]
    UnexpectedCommand(CommandName, CommandName),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
