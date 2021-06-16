// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates provides an API for logging in diem.
//! # Instrumenting with Logs
//! ## Basic instrumenting with Logs
//!
//! A set of logging macros (`info!`, `error!`, `warn!`, `debug!`, and `trace!`) is provided for
//! emitting logs at different levels. All of these macros support the addition of providing
//! structured data along with a formatted text message.  For guidelines on which level to use,
//! see the [coding guidelines](https://developers.diem.com/docs/core/coding-guidelines#logging).
//!
//! The below examples do no type checking for structured log fields, and instead just serialize
//! whatever is given.
//! ```
//! use diem_logger::info;
//!
//! let world = "world!";
//!
//! // Formatted message, similar to `printf!`
//! info!("hello {}", world);
//! // => '{"level":"info", "message": "hello world!"}'
//!
//! // Structured data can be logged using the format 'key = value'
//! // where value implements Serialize.  This can be used for indexing and search later.
//! let value1 = 5;
//! info!(key1 = value1);
//! // => '{"level":"info", "data": {"key1": 5}}'
//!
//! // You can even set multiple key/value pairs and a format message together
//! let value2 = false;
//! info!(key1 = value1, key2 = value2, "hello {}", world);
//! // => '{"level":"info", "data": {"key1": 5, "key2": false}, "message": "hello world!"}'
//!
//! // Structured data can also use `Display` or `Debug` outputs instead.
//! // Using the sigil `?` for debug and `%` for display.
//! let value1 = 5;
//! info!(debug_key = ?value1, display_key = %value1);
//! // => '{"level":"info", "data": {"display_key": 5, "debug_key": 5}}'
//! ```
//!
//! ### Note
//!
//! Arguments used in a formatted message are **not** captured and included as structured data.
//! Everything after the format string literal e.g. `"hello {}"` are only used in the format string.
//!
//! ## Preferred instrumenting with Logs (Typed Schemas)
//!
//! The `Schema` trait can be used to implement typed logging schemas. This can either be
//! implemented by hand or derived using the `Schema` derive proc-macro, implementing the `Schema`
//! trait for the struct as well as providing setters for all fields.
//!
//! ```
//! use diem_logger::{info, Schema};
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
//!
//! let log = LogSchema { a: 5, b: None, c: None };
//!
//! // Automatic setters are named based on the field names, and handle `Option`
//! // None fields will be ignored
//! info!(log.c("radiant"));
//! // => '{"level":"info", "data": { "a": 5, "c": "radiant"}}'
//!
//!
//! #[derive(Schema)]
//! struct OtherSchema<'a> {
//!   val: Option<&'a str>
//! }
//!
//! let log = LogSchema { a: 5, b: None, c: None };
//! let other = OtherSchema { val: None };
//!
//! // Schemas can be combined
//! info!(
//!   other.val("awesome"), // First schema
//!   log // Second schema has fields added to it all
//! );
//! // => '{"level":"info", "data": { "a": 5, "val":"awesome"}}'
//!
//! let log = LogSchema { a: 5, b: None, c: None };
//! let other = OtherSchema { val: None };
//!
//! // Schemas can be combined with one off fields and messages like above
//! info!(
//!    other.val("awesome"), // First schema
//!    log, // Second schema has fields added to it all
//!    new_field = "new", // Basic structured fields
//!    "Message: {}", // Format messages
//!    "Some message" // Format message fields (not added to indexed fields)
//! );
//! // => {"level":"info", "message": "Message: Some message",
//! //     "data": { "a": 5, "val":"awesome", "new_field": "new"}}'
//! ```
//!
//! ## Sampling logs
//!
//! Sometimes logging a large amount of data is expensive.  In order to log information only part
//! of the time, we've added a `sample!` macro that's configurable on how often we want to execute some code.
//!
//! `SampleRate` determines how often the sampled statement will occur.
//!
//! ```
//! use diem_logger::{info, sample, sample::{SampleRate, Sampling}};
//! use std::time::Duration;
//!
//! // Sampled based on frequency of events, log only every 2 logs
//! sample!(SampleRate::Frequency(2), info!("Long log"));
//!
//! // Sampled based on time passed, log at most once a minute
//! sample!(SampleRate::Duration(Duration::from_secs(60)), info!("Long log"));
//! ```
//! # Configuration
//!
//! In order for logs to be captured and emitted a Logger needs to be instantiated. This can be
//! done by using the `Logger` type:
//!
//! ```
//! use diem_logger::{Level, Logger};
//!
//! Logger::builder().level(Level::Info).build();
//! ```

#![forbid(unsafe_code)]

pub mod prelude {
    pub use crate::{
        debug,
        diem_logger::FileWriter,
        error, event, info, sample,
        sample::{SampleRate, Sampling},
        security::SecurityEvent,
        trace, warn,
    };
}
pub mod json_log;

mod diem_logger;
mod event;
mod filter;
mod kv;
mod logger;
mod macros;
mod metadata;
pub mod sample;
pub mod tracing_adapter;

mod security;
mod struct_log;

pub use crate::diem_logger::{
    DiemLogger, DiemLogger as Logger, DiemLoggerBuilder, Writer, CHANNEL_SIZE,
};
pub use event::Event;
pub use filter::{Filter, LevelFilter};
pub use logger::flush;
pub use metadata::{Level, Metadata};

pub use diem_log_derive::Schema;
pub use kv::{Key, KeyValue, Schema, Value, Visitor};
pub use security::SecurityEvent;

mod counters;
