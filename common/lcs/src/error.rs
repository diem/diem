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
    #[error("exceeded max sequence length")]
    ExceededMaxLen(usize),
    #[error("expected boolean")]
    ExpectedBoolean,
    #[error("expected map key")]
    ExpectedMapKey,
    #[error("expected map value")]
    ExpectedMapValue,
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
