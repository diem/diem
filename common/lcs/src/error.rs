// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{de, ser};
use std::{error, fmt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Eof,
    ExceededMaxLen(usize),
    ExpectedBoolean,
    ExpectedMapKey,
    ExpectedMapValue,
    ExpectedOption,
    Custom(String),
    MissingLen,
    NotSupported(&'static str),
    RemainingInput,
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

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use Error::*;

        match self {
            Eof => "unexpected end of input",
            ExceededMaxLen(_) => "exceeded max sequence length",
            ExpectedBoolean => "expected boolean",
            ExpectedMapKey => "expected map key",
            ExpectedMapValue => "expected map value",
            ExpectedOption => "expected option type",
            Custom(msg) => msg,
            MissingLen => "sequence missing length",
            NotSupported(_) => "not supported",
            RemainingInput => "remaining input",
            Utf8 => "malformed utf8",
        }
    }
}
