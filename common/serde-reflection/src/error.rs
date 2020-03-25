// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, ser};
use std::fmt;
use thiserror::Error;

/// Result type used in this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type used in this crate.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("{0}")]
    NotSupported(&'static str),
    #[error("Failed to deserialize {0}")]
    DeserializationError(&'static str),
    #[error("Incompatible formats detected: {0} {1}")]
    Incompatible(String, String),
    #[error("Incomplete tracing detected")]
    UnknownFormat,
    #[error("Incomplete tracing detected inside container: {0}")]
    UnknownFormatInContainer(&'static str),
    #[error("Missing variants detected for specific enums")]
    MissingVariants(Vec<String>),
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
