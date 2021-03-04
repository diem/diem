// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Associated metadata with every log to identify what kind of log and where it came from
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Metadata {
    /// The level of verbosity of the event
    level: Level,

    /// The part of the system where the event occurred
    target: &'static str,

    /// The name of the Rust module where the event occurred
    module_path: &'static str,

    /// The name of the source code file where the event occurred
    file: &'static str,

    /// The line number in the source code file where the event occurred
    line: u32,

    /// The file name and line number together 'file:line'
    location: &'static str,
}

impl Metadata {
    pub const fn new(
        level: Level,
        target: &'static str,
        module_path: &'static str,
        file: &'static str,
        line: u32,
        location: &'static str,
    ) -> Self {
        Self {
            level,
            target,
            module_path,
            file,
            line,
            location,
        }
    }

    pub fn enabled(&self) -> bool {
        crate::logger::enabled(self)
    }

    pub fn level(&self) -> Level {
        self.level
    }

    pub fn target(&self) -> &'static str {
        self.target
    }

    pub fn module_path(&self) -> &'static str {
        self.module_path
    }

    pub fn file(&self) -> &'static str {
        self.file
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn location(&self) -> &'static str {
        self.location
    }
}

static LOG_LEVEL_NAMES: &[&str] = &["ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

/// Logging levels, used for stratifying logs, and disabling less important ones for performance reasons
#[repr(usize)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 0,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl Level {
    fn from_usize(idx: usize) -> Option<Self> {
        let lvl = match idx {
            0 => Level::Error,
            1 => Level::Warn,
            2 => Level::Info,
            3 => Level::Debug,
            4 => Level::Trace,
            _ => return None,
        };

        Some(lvl)
    }
}

/// An error given when no `Level` matches the inputted string
#[derive(Debug)]
pub struct LevelParseError;

impl FromStr for Level {
    type Err = LevelParseError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        LOG_LEVEL_NAMES
            .iter()
            .position(|name| name.eq_ignore_ascii_case(level))
            .map(|idx| Level::from_usize(idx).unwrap())
            .ok_or(LevelParseError)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}
