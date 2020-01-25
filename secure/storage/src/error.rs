// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use thiserror::Error;

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

impl From<lcs::Error> for Error {
    fn from(error: lcs::Error) -> Self {
        Self::SerializationError(format!("{}", error))
    }
}
