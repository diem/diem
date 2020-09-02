// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use backtrace::{Backtrace, BacktraceFrame};
use std::{error::Error, fmt};

/// The ErrorWithContext struct is a wrapper around std::error:Error and provides an
/// additional struct in which to hold auxiliary information about the context of an error object,
/// e.g., where the error originated from in the source code, such as the error backtrace. This
/// is useful for debugging, e.g., when an error is thrown or displayed on the console but the error
/// message isn't helpful enough to identify exactly what went wrong.
pub struct ErrorWithContext<T: Error> {
    pub error: T,
    pub backtrace: Backtrace,
}

/// The string displayed in the backtrace frame that shows where the ErrorWithContext object was
/// initialized. We use this for truncating the backtrace to display only the most helpful backtrace
/// frames.
const CREATION_FRAME_STRING: &str = "ErrorWithContext<T>::new";

impl<T: Error> ErrorWithContext<T> {
    /// Creates and returns an ErrorWithContext struct.
    pub fn new(error: T) -> ErrorWithContext<T> {
        // Grab the current backtrace
        let mut backtrace = Backtrace::new();

        // Truncate the backtrace (if possible) so that when the backtrace is displayed the most
        // relevant frames are at the top. If we can't find the frame where the ErrorWithContext is
        // created, fall back to the entire backtrace.
        let backtrace_frames: Vec<BacktraceFrame> = backtrace.frames().into();
        if !backtrace_frames.is_empty() {
            let mut creation_frame_index = 0;
            for index in 0..backtrace_frames.len() {
                if let Some(frame) = backtrace_frames.get(index) {
                    let frame_string = format!("{:?}", frame.symbols());
                    if frame_string.contains(CREATION_FRAME_STRING) {
                        creation_frame_index = index;
                    }
                }
            }
            backtrace = backtrace_frames[creation_frame_index..].to_vec().into();
        }

        ErrorWithContext { error, backtrace }
    }

    /// Returns a string that displays the error and the context.
    pub fn format_error_and_context(&self) -> String {
        format!(
            "{}\n{}",
            self.format_error_no_context(),
            self.format_context_no_error()
        )
    }

    /// Returns a string that displays the error without a context (i.e., just displays the error).
    pub fn format_error_no_context(&self) -> String {
        format!("Error:\n {:?}", self.error)
    }

    /// Returns a string that displays just the error context (not the error).
    pub fn format_context_no_error(&self) -> String {
        format!("Backtrace:\n {:?}", self.backtrace)
    }
}

/// Debug shows the ErrorWithContext by displaying the error as well as the underlying context.
impl<T: Error> fmt::Debug for ErrorWithContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error_and_context(),)
    }
}

/// Display shows the ErrorWithContext by excluding the underlying context (e.g., backtrace). This
/// is useful for scenarios where the context would only add noise (e.g., logging).
impl<T: Error> fmt::Display for ErrorWithContext<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_error_no_context())
    }
}

impl<T: Error> From<T> for ErrorWithContext<T> {
    fn from(error: T) -> Self {
        ErrorWithContext::new(error)
    }
}

/// Equality is derived using only the underlying error type and not auxiliary information (e.g.,
/// the backtrace).
impl<T: Error + PartialEq> PartialEq for ErrorWithContext<T> {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

#[cfg(test)]
mod tests {
    use crate::{ErrorWithContext, CREATION_FRAME_STRING};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum Error {
        #[error("This is a TestError for testing the ErrorWithContext implementation: {0}")]
        TestError(String),
    }

    #[test]
    fn test_error_implements_debug() {
        let error_message = "test_error_implements_debug";
        let error = Error::TestError(error_message.into());
        let annotated_error = ErrorWithContext::new(error);

        let debug_output = format!("ErrorWithContext debug: {:?}", annotated_error);
        assert!(debug_output.contains(error_message));
        assert!(debug_output.contains(CREATION_FRAME_STRING));
    }

    #[test]
    fn test_error_implements_display() {
        let error_message = "test_error_implements_display";
        let error = Error::TestError(error_message.into());
        let annotated_error = ErrorWithContext::new(error);

        let display_output = format!("ErrorWithContext display: {}", annotated_error);
        assert!(display_output.contains(error_message));
    }
}
