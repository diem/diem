// Copyright 2018 The Grin Developers
//! Cuckatoo specific errors
use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};
use std::io;

/// Cuckatoo solver or validation error
#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Debug, Eq, Fail, PartialEq)]
/// Libwallet error types
pub enum ErrorKind {
    /// Verification error
    #[fail(display = "Verification Error: {}", _0)]
    Verification(String),
    /// Failure to cast from/to generic integer type
    #[fail(display = "IntegerCast")]
    IntegerCast,
    /// IO Error
    #[fail(display = "IO Error")]
    IOError,
    /// Unexpected Edge Error
    #[fail(display = "Edge Addition Error")]
    EdgeAddition,
    /// Path Error
    #[fail(display = "Path Error")]
    Path,
    /// Invalid cycle
    #[fail(display = "Invalid Cycle length: {}", _0)]
    InvalidCycle(usize),
    /// No Cycle
    #[fail(display = "No Cycle")]
    NoCycle,
    /// No Solution
    #[fail(display = "No Solution")]
    NoSolution,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    /// Return errorkind
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<fmt::Error> for Error {
    fn from(_error: fmt::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::IntegerCast),
        }
    }
}

impl From<io::Error> for Error {
    fn from(_error: io::Error) -> Error {
        Error {
            inner: Context::new(ErrorKind::IOError),
        }
    }
}
