// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;
use toml;

#[derive(Debug, Deserialize, Error, PartialEq, Serialize)]
pub enum Error {
    #[error("Internal error: {}", 0)]
    InternalError(String),
    #[error("Key not set: {}", 0)]
    KeyAlreadyExists(String),
    #[error("Key already exists: {}", 0)]
    KeyNotSet(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Unexpected value type")]
    UnexpectedValueType,
}

impl From<base64::DecodeError> for Error {
    fn from(error: base64::DecodeError) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::InternalError(format!("{}", error))
    }
}

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}
