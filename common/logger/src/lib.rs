// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crates provides API for both structured and not structured(text) logging
//!
//! # Text logging
//!
//! Text logging is configured via RUST_LOG macro and have exactly same facade as rust log crate
//!
//! # Structured logging
//!
//! This crate contains two levels of API for structured logging
//!
//! 1) StructuredLogEntry class and send_struct_log! macro for directly composing structured log
//! 2) Bridge between traditional log! macro and structured logging API
//!
//! ## Configuration
//!
//! Structured logger has separate log levels, configured with `STRUCT_LOG_LEVEL`.
//! It is set to debug by default, but it only has effect if structured logger is initialized.
//!
//! Structured logger can be initialized manually with one of `init_XXX_struct_log` functions.
//! Preferred way to initialize structured logging is by using `init_struct_log_from_env`.
//! In this case `STRUCT_LOG_FILE` environment variable is used to set file name for structured logs
//!
//! ## Direct API
//!
//! ```pseudo
//! use std::collections::HashMap;
//! use serde_json::Value;
//!
//! pub struct StructuredLogEntry {
//!     text: Option<String>,
//!     pattern: Option<&'static str>,
//!     name: Option<&'static str>,
//!     module: Option<&'static str>,
//!     location: Option<&'static str>,
//!     git_rev: Option<&'static str>,
//!     data: HashMap<&'static str, Value>,
//! }
//!
//! impl StructuredLogEntry {
//!     pub fn new_unnamed() -> Self { /* ... */ }
//!     pub fn new_named(name: &'static str) -> Self { /* ... */ }
//!     /* ..Bunch of builder style setters for chained initialization such as
//!         entry.data(a, b).data(x, y).. */
//! }
//!
//! // Usage:
//! send_struct_log!(StructuredLogEntry::new_named("Committed")
//!    .data("block", &block)
//!    .data("autor", &author))
//! ```
//!
//! Arguments passed to .data will be serialized into json, and as such should implement Serialize.
//! Only static strings are allowed as field names.
//!
//! send_struct_log! should be used to send structured log entry.
//! This macro populates metadata such as git_rev, location and module, and also skips evaluation of StructuredLogEntry entirely, if structured logging is disabled
//!
//! ## Log macro bridge
//!
//! Crate owners are not required to rewrite their code right away to support new structured logging.
//! Importing logger crate will automatically emit structured logging on every log(debug!, info!, etc) macro invocation, unless no_struct_log feature was enabled on this crate.
//!
//! Note: Only enable 'no_struct_log' on leaf binary crates, never on library, as it might disable structured logging for other crates, due to how cargo handles features at this moment.
//!
//! So
//! ```pseudo
//! info!("Committing {}", block);
//! // Will emit(in addition to regular text log) structured log such as
//! // {
//! //   pattern: "Committing {}",
//! //   data: {
//! //     block: "<id>"
//! //   },
//! //   ...metadata...
//! // }
//! ```
//!
//! There are few caveats to automatic structured logging generation
//!
//! 1) Argument values for structured logging will be serialized with Debug implementation(vs proper json serialization if send_struct_log! macro is used), regardless of what formatter is used for textual log. As a consequence, this means, that every log argument must implement Debug. In theory, this is extra limitation, but in practice currently in libra node crate and it's dependencies there was no single log argument that would not satisfy this criteria.
//!
//! 2) Field names will be automatically evaluated if expression is a single identifier, as in example above field block will get human readable name in json. However, if more complex expression is passed, structured log field name will be based on position of argument: "_0", "_1", etc:
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
//! Another way to set proper name to json fields is to use named format arguments:
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
//! Structured log sink
//! Application must define implementation of StructLogSink interface in order to direct structured logs emitted by send_struct_log and other macros. This sink can be only initialized once, by calling set_struct_logger function.
//!
//! Currently 3 implementations for StructLogSink exist:
//!
//! 1) `NopStructLog` ignores structured logs
//! 2) `PrintStructLog` immediately prints structured logs to stdout
//! 3) `FileStructLog` prints structured logs into provided file. Using this logger creates separate thread for writing files and structured logging itself is asynchronous in this case.

pub use log;

pub mod prelude {
    pub use crate::{crit, debug, error, info, send_struct_log, trace, warn};
}

mod struct_log;

pub use struct_log::{
    init_file_struct_log, init_println_struct_log, init_struct_log_from_env, set_struct_logger,
    struct_logger_enabled, struct_logger_set, StructLogSink, StructuredLogEntry,
};

mod text_log;
pub use log::Level;
pub use text_log::{Logger, CHANNEL_SIZE, DEFAULT_TARGET};

/// Define crit macro that specify libra as the target
// TODO Remove historical crit from code base since it isn't supported in Rust Log.
#[macro_export]
macro_rules! crit {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Error) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::error!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

/// Define debug macro that specify libra as the target
// TODO Remove historical crit from code base since it isn't supported in Rust Log.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Debug) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::debug!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

/// Define  macro that specify libra as the target
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Error) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::error!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

/// Define info macro that specify libra as the target
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Info) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::info!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

/// Define trace macro that specify libra as the target
#[macro_export]
macro_rules! trace {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Trace) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::trace!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

/// Define warn macro that specify libra as the target
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => ({
        if $crate::struct_log_enabled!($crate::log::Level::Warn) {
            $crate::struct_log!($($arg)+);
        }
        $crate::log::warn!(target: $crate::DEFAULT_TARGET, $($arg)+);
    })
}

#[macro_export]
#[cfg(feature = "no_struct_log")]
macro_rules! struct_log_enabled {
    ($level:expr) => {
        false
    };
}

#[macro_export]
#[cfg(not(feature = "no_struct_log"))]
macro_rules! struct_log_enabled {
    ($level:expr) => {
        $crate::struct_logger_enabled($level)
    };
}

#[macro_export]
#[cfg(feature = "no_struct_log")]
macro_rules! struct_log {
    ($($arg:tt)+) => {};
}

#[macro_export]
#[cfg(not(feature = "no_struct_log"))]
macro_rules! struct_log {
    ($($arg:tt)+) => {
        let mut entry = $crate::StructuredLogEntry::new_unnamed();
        $crate::format_struct_args_and_pattern!(entry, $($arg)+);
        $crate::send_struct_log!(entry);
    }
}

#[macro_export]
macro_rules! send_struct_log {
    ($entry:expr) => {
        if $crate::struct_logger_set() {
            let mut entry = $entry;
            entry.module(module_path!());
            entry.location($crate::location!());
            entry.git_rev($crate::git_rev!());
            entry.send();
        }
    };
}

#[macro_export]
macro_rules! location {
    () => {
        concat!(file!(), ":", line!())
    };
}

// GIT_REV env need to be set during _compile_ time to have this var populated
#[macro_export]
macro_rules! git_rev {
    () => {
        option_env!("GIT_REV")
    };
}

#[macro_export]
macro_rules! format_struct_args_and_pattern {
    ($entry:ident, $fmt:expr) => {
        $entry.log(format!($fmt));
        $entry.pattern($fmt);
    };
    ($entry:ident, $fmt:expr,) => {
        $entry.log(format!($fmt));
        $entry.pattern($fmt);
    };
    ($entry:ident, $fmt:expr, $($arg:tt)+) => {
        $entry.log(format!($fmt, $($arg)+));
        $entry.pattern($fmt);
        $crate::format_struct_args!($entry, 0, $($arg)+);
    }
}

#[macro_export]
macro_rules! format_struct_args {
    ($entry:ident, $acc:tt, $arg:ident) => {$crate::format_struct_arg!($entry, $acc, $arg)};
    ($entry:ident, $acc:tt, $arg:ident,) => {$crate::format_struct_arg!($entry, $acc, $arg)};
    ($entry:ident, $acc:tt, $arg:ident,$($rest:tt)+) => {
        $crate::format_struct_arg!($entry, $acc, $arg);
        $crate::format_struct_args!($entry, ($acc + 1), $($rest)+);
    };
    // Block below is same as block above except arg is expr instead of ident.
    // This is needed because of how rust handles idents/expressions
    ($entry:ident, $acc:tt, $arg:expr) => {$crate::format_struct_arg!($entry, $acc, $arg)};
    ($entry:ident, $acc:tt, $arg:expr,) => {$crate::format_struct_arg!($entry, $acc, $arg)};
    ($entry:ident, $acc:tt, $arg:expr,$($rest:tt)+) => {
        $crate::format_struct_arg!($entry, $acc, $arg);
        $crate::format_struct_args!($entry, ($acc + 1), $($rest)+);
    };
    // And one more repetition for name: expr
    ($entry:ident, $acc:tt, $arg_name:ident=$arg:expr) => {$crate::format_struct_arg!($entry, $acc, $arg_name: $arg)};
    ($entry:ident, $acc:tt, $arg_name:ident=$arg:expr,) => {$crate::format_struct_arg!($entry, $acc, $arg_name: $arg)};
    ($entry:ident, $acc:tt, $arg_name:ident=$arg:expr,$($rest:tt)+) => {
        $crate::format_struct_arg!($entry, $acc, $arg_name=$arg);
        $crate::format_struct_args!($entry, ($acc + 1), $($rest)+);
    };
}

#[macro_export]
macro_rules! format_struct_arg {
    ($entry:ident, $acc:tt, $arg_name:ident=$arg:expr) => {
        $entry.data_mutref(stringify!($arg_name), format!("{:?}", $arg));
    };
    ($entry:ident, $acc:tt, $arg:ident) => {
        $entry.data_mutref(stringify!($arg), format!("{:?}", $arg));
    };
    ($entry:ident, $acc:tt, $arg:expr) => {
        $entry.data_mutref($crate::format_index!($acc), format!("{:?}", $arg));
    };
}

// This is a very fun macro indeed
#[macro_export]
#[cfg(not(feature = "names_required"))]
macro_rules! format_index {
    (0) => ("_0");
    ((0+1)) => ("_1");
    (((0+1)+1)) => ("_2");
    ((((0+1)+1)+1)) => ("_3");
    (((((0+1)+1)+1)+1)) => ("_4");
    ($expr:tt) => (compile_error!("You have surprisingly long list of log args, please use named args [e.g. log!(\"{x}\", x=value)], instead of indexes"));
}

#[macro_export]
#[cfg(feature = "names_required")]
macro_rules! format_index {
    ($idx:expr) => {
        compile_error!("names_required feature is set, all log entries require a name")
    };
}
