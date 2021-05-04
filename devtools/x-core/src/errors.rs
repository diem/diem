// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use camino::{Utf8Path, Utf8PathBuf};
use guppy::TargetSpecError;
use hex::FromHexError;
use serde::{de, ser};
use std::{borrow::Cow, error, fmt, io, process::ExitStatus, result, str::Utf8Error};

/// Type alias for the return type for `run` methods.
pub type Result<T, E = SystemError> = result::Result<T, E>;

/// A system error that happened while running a lint.
#[derive(Debug)]
#[non_exhaustive]
pub enum SystemError {
    CwdNotInProjectRoot {
        current_dir: Utf8PathBuf,
        project_root: &'static Utf8Path,
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
    Guppy {
        context: Cow<'static, str>,
        err: guppy::Error,
    },
    HakariCargoToml {
        context: Cow<'static, str>,
        err: hakari::CargoTomlError,
    },
    HakariTomlOut {
        context: Cow<'static, str>,
        err: hakari::TomlOutError,
    },
    Io {
        context: Cow<'static, str>,
        err: io::Error,
    },
    NonUtf8Path {
        path: Vec<u8>,
        err: Utf8Error,
    },
    Serde {
        context: Cow<'static, str>,
        err: Box<dyn error::Error + Send + Sync>,
    },
    TargetSpec {
        context: Cow<'static, str>,
        err: TargetSpecError,
    },
}

impl SystemError {
    pub fn io(context: impl Into<Cow<'static, str>>, err: io::Error) -> Self {
        SystemError::Io {
            context: context.into(),
            err,
        }
    }

    pub fn guppy(context: impl Into<Cow<'static, str>>, err: guppy::Error) -> Self {
        SystemError::Guppy {
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

    pub fn hakari_cargo_toml(
        context: impl Into<Cow<'static, str>>,
        err: hakari::CargoTomlError,
    ) -> Self {
        SystemError::HakariCargoToml {
            context: context.into(),
            err,
        }
    }

    pub fn hakari_toml_out(
        context: impl Into<Cow<'static, str>>,
        err: hakari::TomlOutError,
    ) -> Self {
        SystemError::HakariTomlOut {
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

    pub fn target_spec(context: impl Into<Cow<'static, str>>, err: TargetSpecError) -> Self {
        SystemError::TargetSpec {
            context: context.into(),
            err,
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
                current_dir, project_root,
            ),
            SystemError::Exec { cmd, status } => match status.code() {
                Some(code) => write!(f, "'{}' failed with exit code {}", cmd, code),
                None => write!(f, "'{}' terminated by signal", cmd),
            },
            SystemError::GitRoot(s) => write!(f, "git root error: {}", s),
            SystemError::NonUtf8Path { path, .. } => {
                write!(f, "non-UTF-8 path \"{}\"", String::from_utf8_lossy(path))
            }
            SystemError::FromHex { context, .. }
            | SystemError::Io { context, .. }
            | SystemError::Serde { context, .. }
            | SystemError::Guppy { context, .. }
            | SystemError::HakariCargoToml { context, .. }
            | SystemError::HakariTomlOut { context, .. }
            | SystemError::TargetSpec { context, .. } => write!(f, "while {}", context),
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
            SystemError::Guppy { err, .. } => Some(err),
            SystemError::HakariCargoToml { err, .. } => Some(err),
            SystemError::HakariTomlOut { err, .. } => Some(err),
            SystemError::NonUtf8Path { err, .. } => Some(err),
            SystemError::TargetSpec { err, .. } => Some(err),
            SystemError::Serde { err, .. } => Some(err.as_ref()),
        }
    }
}
