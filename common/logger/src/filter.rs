// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Filtering definitions for controlling what modules and levels are logged

use crate::{Level, Metadata};
use std::{env, str::FromStr};

pub struct FilterParseError;

/// A definition of the most verbose `Level` allowed, or completely off.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LevelFilter {
    /// Returns the most verbose logging level filter.
    pub fn max() -> Self {
        LevelFilter::Trace
    }
}

impl FromStr for LevelFilter {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let level = if s.eq_ignore_ascii_case("OFF") {
            LevelFilter::Off
        } else {
            s.parse::<Level>().map_err(|_| FilterParseError)?.into()
        };

        Ok(level)
    }
}

impl From<Level> for LevelFilter {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => LevelFilter::Error,
            Level::Warn => LevelFilter::Warn,
            Level::Info => LevelFilter::Info,
            Level::Debug => LevelFilter::Debug,
            Level::Trace => LevelFilter::Trace,
        }
    }
}

/// A builder for `Filter` deriving it's `Directive`s from specified modules
#[derive(Default, Debug)]
pub struct Builder {
    directives: Vec<Directive>,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Populates the filter builder from an environment variable
    pub fn with_env(&mut self, env: &str) -> &mut Self {
        if let Ok(s) = env::var(env) {
            self.parse(&s);
        }

        self
    }

    /// Adds a directive to the filter for a specific module.
    pub fn filter_module(&mut self, module: &str, level: LevelFilter) -> &mut Self {
        self.filter(Some(module), level)
    }

    /// Adds a directive to the filter for all modules.
    pub fn filter_level(&mut self, level: LevelFilter) -> &mut Self {
        self.filter(None, level)
    }

    /// Adds a directive to the filter.
    ///
    /// The given module (if any) will log at most the specified level provided.
    /// If no module is provided then the filter will apply to all log messages.
    pub fn filter(&mut self, module: Option<&str>, level: LevelFilter) -> &mut Self {
        self.directives.push(Directive::new(module, level));
        self
    }

    /// Parses a directives string.
    pub fn parse(&mut self, filters: &str) -> &mut Self {
        self.directives.extend(
            filters
                .split(',')
                .map(Directive::from_str)
                .filter_map(Result::ok),
        );
        self
    }

    pub fn build(&mut self) -> Filter {
        if self.directives.is_empty() {
            // Add the default filter if none exist
            self.filter_level(LevelFilter::Error);
        } else {
            // Sort the directives by length of their name, this allows a
            // little more efficient lookup at runtime.
            self.directives.sort_by(|a, b| {
                let alen = a.name.as_ref().map(|a| a.len()).unwrap_or(0);
                let blen = b.name.as_ref().map(|b| b.len()).unwrap_or(0);
                alen.cmp(&blen)
            });
        }

        Filter {
            directives: ::std::mem::take(&mut self.directives),
        }
    }
}

/// A logging filter to determine which logs to keep or remove based on `Directive`s
#[derive(Debug)]
pub struct Filter {
    directives: Vec<Directive>,
}

impl Filter {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn enabled(&self, metadata: &Metadata) -> bool {
        // Search for the longest match, the vector is assumed to be pre-sorted.
        for directive in self.directives.iter().rev() {
            match &directive.name {
                Some(name) if !metadata.module_path().starts_with(name) => {}
                Some(..) | None => return LevelFilter::from(metadata.level()) <= directive.level,
            }
        }
        false
    }
}

/// A `Filter` directive for which logs to keep based on a module `name` based filter
#[derive(Debug)]
struct Directive {
    name: Option<String>,
    level: LevelFilter,
}

impl Directive {
    fn new<T: Into<String>>(name: Option<T>, level: LevelFilter) -> Self {
        Self {
            name: name.map(Into::into),
            level,
        }
    }
}

impl FromStr for Directive {
    type Err = FilterParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('=').map(str::trim);
        let (name, level) = match (parts.next(), parts.next(), parts.next()) {
            // Only a level or module is provided, e.g. 'debug' or 'crate::foo'
            (Some(level_or_module), None, None) => match level_or_module.parse() {
                Ok(level) => (None, level),
                Err(_) => (Some(level_or_module), LevelFilter::max()),
            },
            // Only a name is provided, e.g. 'crate='
            (Some(name), Some(""), None) => (Some(name), LevelFilter::max()),
            // Both a name and level is provided, e.g. 'crate=debug'
            (Some(name), Some(level), None) => (Some(name), level.parse()?),
            _ => return Err(FilterParseError),
        };

        Ok(Directive::new(name, level))
    }
}

#[cfg(test)]
mod tests {
    use super::{Builder, Level, LevelFilter, Metadata};

    fn make_metadata(level: Level, target: &'static str) -> Metadata {
        Metadata::new(level, target, target, "", 0, "")
    }

    #[test]
    fn filter_info() {
        let logger = Builder::new().filter_level(LevelFilter::Info).build();
        assert!(logger.enabled(&make_metadata(Level::Info, "crate1")));
        assert!(!logger.enabled(&make_metadata(Level::Debug, "crate1")));
    }

    #[test]
    fn filter_beginning_longest_match() {
        let logger = Builder::new()
            .filter(Some("crate2"), LevelFilter::Info)
            .filter(Some("crate2::mod"), LevelFilter::Debug)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(logger.enabled(&make_metadata(Level::Debug, "crate2::mod1")));
        assert!(!logger.enabled(&make_metadata(Level::Debug, "crate2")));
    }

    #[test]
    fn parse_default() {
        let logger = Builder::new().parse("info,crate1::mod1=warn").build();
        assert!(logger.enabled(&make_metadata(Level::Warn, "crate1::mod1")));
        assert!(logger.enabled(&make_metadata(Level::Info, "crate2::mod2")));
    }

    #[test]
    fn match_full_path() {
        let logger = Builder::new()
            .filter(Some("crate2"), LevelFilter::Info)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(logger.enabled(&make_metadata(Level::Warn, "crate1::mod1")));
        assert!(!logger.enabled(&make_metadata(Level::Info, "crate1::mod1")));
        assert!(logger.enabled(&make_metadata(Level::Info, "crate2")));
        assert!(!logger.enabled(&make_metadata(Level::Debug, "crate2")));
    }

    #[test]
    fn no_match() {
        let logger = Builder::new()
            .filter(Some("crate2"), LevelFilter::Info)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(!logger.enabled(&make_metadata(Level::Warn, "crate3")));
    }

    #[test]
    fn match_beginning() {
        let logger = Builder::new()
            .filter(Some("crate2"), LevelFilter::Info)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(logger.enabled(&make_metadata(Level::Info, "crate2::mod1")));
    }

    #[test]
    fn match_beginning_longest_match() {
        let logger = Builder::new()
            .filter(Some("crate2"), LevelFilter::Info)
            .filter(Some("crate2::mod"), LevelFilter::Debug)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(logger.enabled(&make_metadata(Level::Debug, "crate2::mod1")));
        assert!(!logger.enabled(&make_metadata(Level::Debug, "crate2")));
    }

    #[test]
    fn match_default() {
        let logger = Builder::new()
            .filter(None, LevelFilter::Info)
            .filter(Some("crate1::mod1"), LevelFilter::Warn)
            .build();
        assert!(logger.enabled(&make_metadata(Level::Warn, "crate1::mod1")));
        assert!(logger.enabled(&make_metadata(Level::Info, "crate2::mod2")));
    }

    #[test]
    fn zero_level() {
        let logger = Builder::new()
            .filter(None, LevelFilter::Info)
            .filter(Some("crate1::mod1"), LevelFilter::Off)
            .build();
        assert!(!logger.enabled(&make_metadata(Level::Error, "crate1::mod1")));
        assert!(logger.enabled(&make_metadata(Level::Info, "crate2::mod2")));
    }

    #[test]
    fn parse_valid() {
        let mut builder = Builder::new();
        builder.parse("crate1::mod1=error,crate1::mod2,crate2=debug");
        let dirs = &builder.directives;

        assert_eq!(dirs.len(), 3);
        assert_eq!(dirs[0].name.as_deref(), Some("crate1::mod1"));
        assert_eq!(dirs[0].level, LevelFilter::Error);

        assert_eq!(dirs[1].name.as_deref(), Some("crate1::mod2"));
        assert_eq!(dirs[1].level, LevelFilter::max());

        assert_eq!(dirs[2].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[2].level, LevelFilter::Debug);
    }

    #[test]
    fn parse_invalid_crate() {
        // test parsing with multiple = in specification
        let mut builder = Builder::new();
        builder.parse("crate1::mod1=warn=info,crate2=debug");
        let dirs = &builder.directives;

        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[0].level, LevelFilter::Debug);
    }

    #[test]
    fn parse_invalid_level() {
        // test parse with 'noNumber' as log level
        let mut builder = Builder::new();
        builder.parse("crate1::mod1=noNumber,crate2=debug");
        let dirs = &builder.directives;
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[0].level, LevelFilter::Debug);
    }

    #[test]
    fn parse_string_level() {
        // test parse with 'warn' as log level
        let mut builder = Builder::new();
        builder.parse("crate1::mod1=wrong,crate2=warn");
        let dirs = &builder.directives;
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[0].level, LevelFilter::Warn);
    }

    #[test]
    fn parse_empty_level() {
        // test parse with '' as log level
        let mut builder = Builder::new();
        builder.parse("crate1::mod1=wrong,crate2=");
        let dirs = &builder.directives;
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[0].level, LevelFilter::max());
    }

    #[test]
    fn parse_global() {
        // test parse with no crate
        let mut builder = Builder::new();
        builder.parse("warn,crate2=debug");
        let dirs = &builder.directives;
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0].name.as_deref(), None);
        assert_eq!(dirs[0].level, LevelFilter::Warn);
        assert_eq!(dirs[1].name.as_deref(), Some("crate2"));
        assert_eq!(dirs[1].level, LevelFilter::Debug);
    }
}
