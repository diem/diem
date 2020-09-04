// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates provides an API for logging in libra.
//!
//! # Instrumenting with Logs
//!
//! A set of logging macros (`info!`, `error!`, `warn!`, `debug!`, and `trace!`) is provided for
//! emitting logs at different levels. All of these macros support the addition of providing
//! structured data along with a formatted text message.
//!
//! ```
//! use libra_logger::info;
//!
//! let world = "world!";
//!
//! // formatted message
//! info!("hello {}", world);
//!
//! // structured data can be logged using the format 'key = value'
//! // where value implements Serialize
//! let value1 = 5;
//! info!(key1 = value1);
//!
//! // You can even set multiple key/value pairs and a format message
//! let value2 = false;
//! info!(key1 = value1, key2 = value2, "hello {}", world);
//! ```
//!
//! ### Note
//!
//! Arguments used in a formatted message are **not** captured and included as structured data.
//!
//! ## Typed Schema's
//!
//! The `Schema` trait can be used to implement typed logging schemas. This can either be
//! implemented by hand or derived using the `Schema` derive proc-macro, implementing the `Schema`
//! trait for the struct as well as providing setters for all fields.
//!
//! ```
//! use libra_logger::{info, Schema};
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
//! info!(log.c("radiant"));
//! ```
//!
//! # Configuration
//!
//! In order for logs to be captured and emitted a Logger needs to be instantiated. This can be
//! done by using the `Logger` type:
//!
//! ```
//! use libra_logger::{Level, Logger};
//!
//! Logger::builder().level(Level::Info).build();
//! ```

#![forbid(unsafe_code)]

pub mod prelude {
    pub use crate::{
        debug, error, event, info, security::SecurityEvent, trace, warn, StructuredLogEntry,
    };
}
pub mod json_log;

mod event;
mod filter;
mod kv;
mod libra_logger;
mod logger;
mod macros;
mod metadata;
mod security;
mod struct_log;

pub use crate::libra_logger::{
    LibraLogger, LibraLogger as Logger, LibraLoggerBuilder, Writer, CHANNEL_SIZE,
};
pub use event::Event;
pub use filter::{Filter, LevelFilter};
pub use metadata::{Level, Metadata};

pub use struct_log::{LoggingField, StructuredLogEntry};

pub use kv::{Key, KeyValue, Schema, Value, Visitor};
pub use libra_log_derive::Schema;
pub use security::SecurityEvent;

pub mod counters;
