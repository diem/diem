// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use backtrace::Backtrace;
use std::{error::Error, fmt};

/// The AnnotatedError struct is a wrapper around std::error:Error. AnnotatedError provides an
/// additional struct in which to hold auxiliary information about the context of an error object,
/// e.g., where the error originated from in the source code, such as the error backtrace. This
/// is useful for debugging, e.g., when an error is thrown or displayed on the console but the error
/// message isn't helpful enough to identify exactly what went wrong.
pub struct AnnotatedError<T: Error> {
    pub error: T,
    backtrace: Backtrace,
}

impl<T: Error> AnnotatedError<T> {
    /// Creates and returns an AnnotatedError struct.
    pub fn new(error: T) -> AnnotatedError<T> {
        AnnotatedError {
            error,
            backtrace: Backtrace::new(),
        }
    }

    /// Formats the AnnotatedError object in a way that can be printed out using fmt::Result. This
    /// is useful for implementing fmt::Display and fmt::Debug.
    fn format_output(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error: {:?},\nBacktrace:\n {:?}",
            self.error, self.backtrace
        )
    }
}

impl<T: Error> fmt::Debug for AnnotatedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_output(f)
    }
}

impl<T: Error> fmt::Display for AnnotatedError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.format_output(f)
    }
}

impl<T: Error> From<T> for AnnotatedError<T> {
    fn from(error: T) -> Self {
        AnnotatedError::new(error)
    }
}

/// Equality is derived using only the underlying error type and not auxiliary information (e.g.,
/// the backtrace).
impl<T: Error + PartialEq> PartialEq for AnnotatedError<T> {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

#[cfg(test)]
mod tests {
    use crate::AnnotatedError;
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("This is a TestError for testing the AnnotatedError implementation: {0}")]
        TestError(String),
    }

    #[test]
    fn test_error_implements_debug() {
        let error_message = "test_error_implements_debug";
        let error = Error::TestError(error_message.into());
        let annotated_error = AnnotatedError::new(error);

        let debug_output = format!("AnnotatedError debug: {:?}", annotated_error);
        assert!(debug_output.contains(error_message));
    }

    #[test]
    fn test_error_implements_display() {
        let error_message = "test_error_implements_display";
        let error = Error::TestError(error_message.into());
        let annotated_error = AnnotatedError::new(error);

        let display_output = format!("AnnotatedError display: {}", annotated_error);
        assert!(display_output.contains(error_message));
    }
}
