// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Provides ways to control how the KV values passed to slog macros are printed

use slog::{Key, Level};

/// The KV value is being processed based on the category it is bucketed in
#[derive(Debug, PartialEq, Eq)]
pub enum KVCategory {
    /// KV value is not printed at all
    Ignore,
    /// KV value is inlined with the main message passed to slog macro
    Inline,
    /// KV value is printed as a separate line with the provided log level
    LevelLog(Level),
}

/// Structures implementing this trait are being used to categorize the KV values into one of the
/// `KVCategory`.
pub trait KVCategorizer {
    /// For a given key from KV decide which category it belongs to
    fn categorize(&self, key: Key) -> KVCategory;
    /// For a given key from KV return a name that should be printed for it
    fn name(&self, key: Key) -> &'static str;
    /// True if category of a given key is KVCategory::Ignore
    fn ignore(&self, key: Key) -> bool {
        self.categorize(key) == KVCategory::Ignore
    }
}

/// Placeholder categorizer that inlines all KV values with names equal to key
pub struct InlineCategorizer;
impl KVCategorizer for InlineCategorizer {
    fn categorize(&self, _key: Key) -> KVCategory {
        KVCategory::Inline
    }

    fn name(&self, key: Key) -> &'static str {
        key
    }
}

/// Used to properly print `error_chain` `Error`s. It displays the error and it's causes in
/// separate log lines as well as backtrace if provided.
/// The `error_chain` `Error` must implement `KV` trait. It is recommended to use `impl_kv_error`
/// macro to generate the implementation.
pub struct ErrorCategorizer;
impl KVCategorizer for ErrorCategorizer {
    fn categorize(&self, key: Key) -> KVCategory {
        match key {
            "error" => KVCategory::LevelLog(Level::Error),
            "cause" => KVCategory::LevelLog(Level::Debug),
            "backtrace" => KVCategory::LevelLog(Level::Trace),
            _ => InlineCategorizer.categorize(key),
        }
    }

    fn name(&self, key: Key) -> &'static str {
        match key {
            "error" => "Error",
            "cause" => "Caused by",
            "backtrace" => "Originated in",
            "root_cause" => "Root cause",
            _ => InlineCategorizer.name(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline() {
        let categorizer = InlineCategorizer;
        let values = vec!["test", "test2"];
        for v in values {
            assert_eq!(categorizer.categorize(v), KVCategory::Inline);
            assert_eq!(categorizer.name(v), v);
        }
    }
}
