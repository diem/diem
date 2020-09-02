// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates provides an API for both structured and non-structured(text) logging.
//!
//! # Text logging
//!
//! Text logging is configured via `RUST_LOG` environment variable and has exactly same facade as the rust log crate.
//!
//! # Structured logging
//!
//! This crate contains two types of APIs for structured logging.
//!
//! 1) The `StructuredLogEntry` class and (`sl_info!`, `sl_error!`, ...) macros for directly composing structured logs.
//! 2) A bridge between traditional `log!` macro and the structured logging API (which will be deprecated).
//!
//! ## Configuration
//!
//! Structured logger has separate log levels that are configured with `STRUCT_LOG_LEVEL`.
//! It is set to `DEBUG` by default, but executes if structured logger is initialized.
//!
//! Structured logger can be initialized manually with one of the `init_XXX_struct_log()` functions.
//! The preferred way to initialize structured logging is by using `init_struct_log_from_env()`.
//! In this case, the `STRUCT_LOG_FILE` environment variable is used to set the file name for structured logs.
//!
//! ## Direct API
//!
//! ```pseudo
//! use std::collections::HashMap;
//! use serde_json::Value;
//!
//! pub struct StructuredLogEntry {
//!     log: Option<String>,
//!     pattern: Option<&'static str>,
//!     category: Option<&'static str>,
//!     name: Option<&'static str>,
//!     module: Option<&'static str>,
//!     location: Option<&'static str>,
//!     timestamp: String,
//!     level: Option<log::Level>,
//!     data: HashMap<&'static str, Value>,
//! }
//!
//! impl StructuredLogEntry {
//!     pub fn new_named(category: &'static str, name: &'static str) -> Self { /* ... */ }
//!     /* ... Builder style setters for chained initialization such as
//!         entry.data(a, b).data(x, y) ... */
//! }
//!
//! // Usage:
//! let logging_field: LoggingField<String> = LoggingField::new("String");
//! let string = "test".to_string();
//!
//! sl_info!(StructuredLogEntry::new_named("TransactionEvents", "Committed")
//!    .data("block", &block)
//!    .data_display("author", &author)
//!    .field(&logging_field, &string)
//!    .message(format!("Committed block: {:?} by {}", block, author))
//! ```
//!
//! Arguments passed to `data()` will be serialized into JSON, and must implement `Serialize`.
//! Arguments passed to `data_display()` will instead use display and must implement `Display`.
//! Arguments passed to `field()` allows for typed fields for type checking, and must implement `Serialize`.
//! Only static strings are allowed as field names.
//!
//! (`sl_info!`, `sl_error!`, etc.) should be used to send structured log entries based on log level.
//! This macro populates the metadata such as code location and module, and skips the evaluation of
//! `StructuredLogEntry` entirely if structured logging is disabled for the log level.
//!
//! ## Typed Schema's
//!
//! The `Schema` trait can be used to implement typed logging schemas. This can either be
//! implemented by hand or derived using the `Schema` derive proc-macro, implementing the `Schema`
//! trait for the struct as well as providing setters for all fields.
//!
//! ```rust
//! use libra_logger::Schema;
//!
//! #[derive(Schema)]
//! struct LogSchema<'a> {
//!     // Log using this type's Serialize impl
//!     a: usize,
//!     // Log using this type's Debug impl
//!     #[schema(debug)]
//!     b: Option<Vec<bool>>,
//!     // Log using this type's Display impl
//!     #[schema(display)]
//!     c: Option<&'a str>,
//! }
//! ```
//!
//! ## Log macro bridge
//!
//! Crate owners are not required to rewrite their code right away to support new structured logging.
//! Importing logger crate will automatically emit structured logs on every log(`debug!`, `info!`, etc.) macro invocation.
//!
//! So
//! ```pseudo
//! info!("Committing {}", block);
//! // Will emit(in addition to regular text log) structured log such as
//! // {
//! //   level: "Info",
//! //   pattern: "Committing {}",
//! //   data: {
//! //     block: "<id>"
//! //   },
//! //   ...metadata...
//! // }
//! ```
//!
//! There are few caveats to automatic structured logging generation:
//! 1) Argument values for structured logging will be serialized using `Debug` vs `Serialize`,
//! regardless of what formatter is used for the text log. As a consequence, every log argument must
//! implement `Debug`. This has led to unexpected large logs from the default `Debug` implementations.
//!
//! 2) Field names will be automatically evaluated if the expression is a single identifier, as in
//! the example above, the field `block` will be named `block`. However, if a more complex expression
//! is passed (e.g. `block.id()`), the field name will be based on the position of the argument: `_0`, `_1`, etc.
//!
//! ```pseudo
//! info!("Committing {}", block.id());
//! //->
//! // {
//! //  data: {
//! //    "_0": "<id>"
//! //  },
//! //  ...metadata...
//! // }
//! ```
//! Another way to set the name for fields is to use named format arguments:
//! ```pseudo
//! info!("Committing {id}", id=block.id());
//! //->
//! // {
//! //  data: {
//! //    "id": "<id>"
//! //  },
//! //  ...metadata...
//! // }
//! ```
//!
//! ## Structured Log Sink
//! The application must define implementation of the StructLogSink trait in order to direct structured
//! logs emitted by `sl_info!` and other macros. The global sink can be only initialized once, by
//! calling the `set_struct_logger()` function.
//!
//! Currently 4 implementations for StructLogSink exist:
//!
//! 1) `NopStructLog` ignores structured logs
//! 2) `PrintStructLog` immediately prints structured logs to stdout
//! 3) `FileStructLog` prints structured logs into a file. This logger separate thread for writing files asynchronously.
//! 4) `TCPStructLog` sends structured logs to a TCP endpoint. This logger separate thread for sending logs asynchronously.

pub use log;

pub mod prelude {
    pub use crate::{
        debug, error, event, info,
        security::{security_events, security_log},
        sl_debug, sl_error, sl_info, sl_level, sl_trace, sl_warn, trace, warn, StructuredLogEntry,
    };
}
pub mod json_log;

mod event;
mod kv;
mod logger;
mod macros;
mod metadata;
mod security;
mod struct_log;

pub use event::Event;
pub use metadata::{Level, Metadata};

pub use struct_log::{
    init_file_struct_log, init_println_struct_log, init_struct_log_from_env, set_struct_logger,
    struct_logger_enabled, struct_logger_set, LoggingField, StructLogSink, StructuredLogEntry,
};

pub use kv::{Key, KeyValue, Schema, Value, Visitor};
pub use libra_log_derive::Schema;

mod text_log;
pub use text_log::{Logger, CHANNEL_SIZE, DEFAULT_TARGET};
pub mod counters;

#[cfg(test)]
mod tests;

#[macro_export]
macro_rules! struct_log_enabled {
    ($level:expr) => {
        $crate::struct_logger_enabled($level)
    };
}

#[macro_export]
macro_rules! sl_debug {
    ($entry:expr) => {{
        $crate::sl_level!($crate::Level::Debug, $entry);
    }};
}

#[macro_export]
macro_rules! sl_error {
    ($entry:expr) => {{
        $crate::sl_level!($crate::Level::Error, $entry);
    }};
}

#[macro_export]
macro_rules! sl_info {
    ($entry:expr) => {{
        $crate::sl_level!($crate::Level::Info, $entry);
    }};
}

#[macro_export]
macro_rules! sl_trace {
    ($entry:expr) => {{
        $crate::sl_level!($crate::Level::Trace, $entry);
    }};
}

#[macro_export]
macro_rules! sl_warn {
    ($entry:expr) => {{
        $crate::sl_level!($crate::Level::Warn, $entry);
    }};
}

/// Allows for dynamic macro levels, but still filtering
/// Use of this is highly discouraged, and you should stick to the `sl_info!` type macros.
#[macro_export]
macro_rules! sl_level {
    ($level:expr, $entry:expr) => {
        if $crate::struct_log_enabled!($level) {
            let mut entry = $entry;
            entry.add_category(module_path!().split("::").next().unwrap());
            entry.add_module(module_path!());
            entry.add_location($crate::location!());
            entry = entry.level($level);
            entry.send();
            $crate::counters::STRUCT_LOG_COUNT.inc();
        }
    };
}

#[macro_export]
macro_rules! location {
    () => {
        concat!(file!(), ":", line!())
    };
}
