// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Local storage unavailable, please check your configuration")]
    LocalStorageUnavailable,
    #[error("Failed to read local storage: {0}")]
    LocalStorageReadError(String),
    #[error("Failed to sign data using local storage: {0}")]
    LocalStorageSigningError(String),
    #[error("Unexpected command, expected {0}, found {1}")]
    UnexpectedCommand(String, String),
}
