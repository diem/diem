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
    #[error("Invalid arguments: {0}")]
    CommandArgumentError(String),
    #[error("Unable to load config: {0}")]
    ConfigError(String),
    #[error("Failed to read '{0}' from JSON-RPC: {1}")]
    JsonRpcReadError(&'static str, String),
    #[error("Failed to write '{0}' from JSON-RPC: {1}")]
    JsonRpcWriteError(&'static str, String),
    #[error("{0} storage unavailable, please check your configuration: {1}")]
    StorageUnavailable(&'static str, String),
    #[error("Failed to read, {1}, from {0} storage: {2}")]
    StorageReadError(&'static str, &'static str, String),
    #[error("Failed to sign {1} with {2} using {0} storage: {2}")]
    StorageSigningError(&'static str, &'static str, &'static str, String),
    #[error("Failed to write, {1}, to {0} storage: {2}")]
    StorageWriteError(&'static str, &'static str, String),
    #[error("Unable to parse, {0}, error {1}")]
    UnableToParse(&'static str, String),
    #[error("Unable to parse file, {0}, error {1}")]
    UnableToParseFile(String, String),
    #[error("Unable to read file, {0}, error {1}")]
    UnableToReadFile(String, String),
    #[error("Unexpected command, expected {0}, found {1}")]
    UnexpectedCommand(String, String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}
