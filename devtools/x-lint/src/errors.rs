// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{error, fmt, io, process::ExitStatus, result};

/// Type alias for the return type for `run` methods.
pub type Result<T, E = SystemError> = result::Result<T, E>;

/// A system error that happened while running a lint.
#[derive(Debug)]
#[non_exhaustive]
pub enum SystemError {
    Exec {
        cmd: &'static str,
        status: ExitStatus,
    },
    Guppy(guppy::Error),
    Io(io::Error),
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SystemError::Exec { cmd, status } => match status.code() {
                Some(code) => write!(f, "'{}' failed with exit code {}", cmd, code),
                None => write!(f, "'{}' terminated by signal", cmd),
            },
            SystemError::Io(err) => write!(f, "IO error: {}", err),
            SystemError::Guppy(err) => write!(f, "guppy error: {}", err),
        }
    }
}

impl error::Error for SystemError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            SystemError::Exec { .. } => None,
            SystemError::Io(err) => Some(err),
            SystemError::Guppy(err) => Some(err),
        }
    }
}

impl From<guppy::Error> for SystemError {
    fn from(err: guppy::Error) -> Self {
        SystemError::Guppy(err)
    }
}

impl From<io::Error> for SystemError {
    fn from(err: io::Error) -> Self {
        SystemError::Io(err)
    }
}
