// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, ser};
use std::fmt;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    #[error("unexpected end of input")]
    Eof,
    #[error("I/O error: {0}")]
    Io(String),
    #[error("exceeded max sequence length: {0}")]
    ExceededMaxLen(usize),
    #[error("exceeded max container depth while entering: {0}")]
    ExceededContainerDepthLimit(&'static str),
    #[error("expected boolean")]
    ExpectedBoolean,
    #[error("expected map key")]
    ExpectedMapKey,
    #[error("expected map value")]
    ExpectedMapValue,
    #[error("keys of serialized maps must be unique and in increasing order")]
    NonCanonicalMap,
    #[error("expected option type")]
    ExpectedOption,
    #[error("{0}")]
    Custom(String),
    #[error("sequence missing length")]
    MissingLen,
    #[error("not supported: {0}")]
    NotSupported(&'static str),
    #[error("remaining input")]
    RemainingInput,
    #[error("malformed utf8")]
    Utf8,
    #[error("ULEB128 encoding was not minimal in size")]
    NonCanonicalUleb128Encoding,
    #[error("ULEB128-encoded integer did not fit in the target size")]
    IntegerOverflowDuringUleb128Decoding,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
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
