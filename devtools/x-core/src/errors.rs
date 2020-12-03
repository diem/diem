// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use hex::FromHexError;
use serde::{de, ser};
use std::{
    borrow::Cow,
    error, fmt, io,
    path::{Path, PathBuf},
    process::ExitStatus,
    result,
};

/// Type alias for the return type for `run` methods.
pub type Result<T, E = SystemError> = result::Result<T, E>;

/// A system error that happened while running a lint.
#[derive(Debug)]
#[non_exhaustive]
pub enum SystemError {
    CwdNotInProjectRoot {
        current_dir: PathBuf,
        project_root: &'static Path,
    },
    Exec {
        cmd: &'static str,
        status: ExitStatus,
    },
    GitRoot(Cow<'static, str>),
    FromHex {
        context: Cow<'static, str>,
        err: FromHexError,
    },
    Guppy(guppy::Error),
    Io {
        context: Cow<'static, str>,
        err: io::Error,
    },
    Serde {
        context: Cow<'static, str>,
        err: Box<dyn error::Error + Send + Sync>,
    },
}

impl SystemError {
    pub fn io(context: impl Into<Cow<'static, str>>, err: io::Error) -> Self {
        SystemError::Io {
            context: context.into(),
            err,
        }
    }

    pub fn git_root(msg: impl Into<Cow<'static, str>>) -> Self {
        SystemError::GitRoot(msg.into())
    }

    pub fn from_hex(context: impl Into<Cow<'static, str>>, err: FromHexError) -> Self {
        SystemError::FromHex {
            context: context.into(),
            err,
        }
    }

    pub fn de(
        context: impl Into<Cow<'static, str>>,
        err: impl de::Error + Send + Sync + 'static,
    ) -> Self {
        SystemError::Serde {
            context: context.into(),
            err: Box::new(err),
        }
    }

    pub fn ser(
        context: impl Into<Cow<'static, str>>,
        err: impl ser::Error + Send + Sync + 'static,
    ) -> Self {
        SystemError::Serde {
            context: context.into(),
            err: Box::new(err),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SystemError::CwdNotInProjectRoot {
                current_dir,
                project_root,
            } => write!(
                f,
                "current dir {} not in project root {}",
                current_dir.display(),
                project_root.display(),
            ),
            SystemError::Exec { cmd, status } => match status.code() {
                Some(code) => write!(f, "'{}' failed with exit code {}", cmd, code),
                None => write!(f, "'{}' terminated by signal", cmd),
            },
            SystemError::GitRoot(s) => write!(f, "git root error: {}", s),
            SystemError::FromHex { context, .. }
            | SystemError::Io { context, .. }
            | SystemError::Serde { context, .. } => write!(f, "while {}", context),
            SystemError::Guppy(err) => write!(f, "guppy error: {}", err),
        }
    }
}

impl error::Error for SystemError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            SystemError::CwdNotInProjectRoot { .. }
            | SystemError::Exec { .. }
            | SystemError::GitRoot(_) => None,
            SystemError::FromHex { err, .. } => Some(err),
            SystemError::Io { err, .. } => Some(err),
            SystemError::Guppy(err) => Some(err),
            SystemError::Serde { err, .. } => Some(err.as_ref()),
        }
    }
}

impl From<guppy::Error> for SystemError {
    fn from(err: guppy::Error) -> Self {
        SystemError::Guppy(err)
    }
}
