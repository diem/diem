// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub use failure::Error;
use failure::Fail;

/// The common result type used in this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Defines all errors in this crate.
#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "ParseError: {}", _0)]
    ParseError(String),
}
