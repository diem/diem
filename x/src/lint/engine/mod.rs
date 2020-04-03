// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Lint engine.
//!
//! The overall design is generally inspired by
//! [Arcanist](https://secure.phabricator.com/book/phabricator/article/arcanist_lint)'s lint engine.

pub mod content;
pub mod file;
pub mod project;
mod runner;

pub use runner::*;

use std::{borrow::Cow, error, fmt, io, path::Path, process::ExitStatus, result};

/// Represents a linter.
pub trait Linter: Send + Sync + fmt::Debug {
    /// Returns the name of the linter.
    fn name(&self) -> &'static str;
}

/// Represents common functionality among various `Context` instances.
pub(super) trait LintContext<'l> {
    /// Returns the kind of this lint context.
    fn kind(&self) -> LintKind<'l>;

    /// Returns a `LintSource` for this lint context.
    fn source(&self, name: &'static str) -> LintSource<'l> {
        LintSource::new(name, self.kind())
    }
}

/// A lint formatter.
///
/// Lints write `LintMessage` instances to this.
pub struct LintFormatter<'l, 'a> {
    source: LintSource<'l>,
    messages: &'a mut Vec<(LintSource<'l>, LintMessage)>,
}

impl<'l, 'a> LintFormatter<'l, 'a> {
    pub fn new(
        source: LintSource<'l>,
        messages: &'a mut Vec<(LintSource<'l>, LintMessage)>,
    ) -> Self {
        Self { source, messages }
    }

    /// Writes a new lint message to this formatter.
    pub fn write(&mut self, message: LintMessage) {
        self.messages.push((self.source, message));
    }
}

/// The run status of a lint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunStatus<'l> {
    /// This lint run was successful, with messages possibly written into the `LintFormatter`.
    Executed,
    /// This lint was skipped.
    Skipped(SkipReason<'l>),
}

/// The reason for why this lint was skipped.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SkipReason<'l> {
    /// This file was not valid UTF-8.
    NonUtf8,
    /// This extension was unsupported.
    UnsupportedExtension(Option<&'l str>),
    // TODO: Add more reasons.
}

/// A message raised by a lint.
#[derive(Debug)]
pub struct LintMessage {
    level: LintLevel,
    message: Cow<'static, str>,
}

impl LintMessage {
    pub fn new(level: LintLevel, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            level,
            message: message.into(),
        }
    }

    pub fn level(&self) -> LintLevel {
        self.level
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum LintLevel {
    Error,
    Warning,
    // TODO: add more levels?
}

impl fmt::Display for LintLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LintLevel::Error => write!(f, "ERROR"),
            LintLevel::Warning => write!(f, "WARNING"),
        }
    }
}

/// Message source for lints.
#[derive(Copy, Clone, Debug)]
pub struct LintSource<'l> {
    name: &'static str,
    kind: LintKind<'l>,
}

impl<'l> LintSource<'l> {
    pub(super) fn new(name: &'static str, kind: LintKind<'l>) -> Self {
        Self { name, kind }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn kind(&self) -> LintKind<'l> {
        self.kind
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LintKind<'l> {
    Project,
    File(&'l Path),
    Content(&'l Path),
}

impl<'l> fmt::Display for LintKind<'l> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LintKind::Project => write!(f, "project"),
            LintKind::File(path) => write!(f, "file {}", path.display()),
            LintKind::Content(path) => write!(f, "content {}", path.display()),
        }
    }
}

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
        }
    }
}

impl error::Error for SystemError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            SystemError::Exec { .. } => None,
            SystemError::Io(err) => Some(err),
        }
    }
}

impl From<io::Error> for SystemError {
    fn from(err: io::Error) -> Self {
        SystemError::Io(err)
    }
}

pub mod prelude {
    pub use super::{
        content::{ContentContext, ContentLinter},
        file::FileContext,
        project::ProjectContext,
        LintFormatter, LintKind, LintLevel, LintMessage, LintSource, Linter, Result, RunStatus,
        SkipReason, SystemError,
    };
}
