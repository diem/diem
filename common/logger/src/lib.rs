// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! A default logger for Libra project.
//!
//! ## Usage
//!
//! ```rust, no_run
//! use libra_logger::prelude::*;
//!
//! let _g = libra_logger::set_default_global_logger(false /* async */, Some(256));
//! info!("Starting...");
//! ```

// This is really annoying. The `error!` and other macros in `slog_scope` depend on the
// `slog_error!` and other macros exported by `slog`. They need to be exported into the environment
// for the `slog_scope` macros to pick them up. However if you use `#[macro_use]` then the linter
// complains about unused imports. Ugh.
#[allow(unused_imports)]
#[macro_use]
extern crate slog;

mod collector_serializer;
mod glog_format;
mod kv_categorizer;
mod security;
mod simple_logger;

use crate::kv_categorizer::ErrorCategorizer;
use glog_format::GlogFormat;
use lazy_static::lazy_static;
use slog::{o, Discard, Drain, FilterLevel, Logger, Never};
pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
use slog_async::Async;
use slog_envlogger::{EnvLogger, LogBuilder};
pub use slog_scope::{crit, debug, error, info, trace, warn};
use slog_scope::{set_global_logger, GlobalLoggerGuard};
use slog_term::{PlainDecorator, TermDecorator};
use std::sync::Mutex;

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use crate::security::{security_log, SecurityEvent};
    pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
    pub use slog_scope::{crit, debug, error, info, trace, warn};
}

pub use simple_logger::{set_simple_logger, set_simple_logger_prefix};

/// Creates and sets default global logger.
/// Caller must keep the returned guard alive.
pub fn set_default_global_logger(async_drain: bool, chan_size: Option<usize>) -> GlobalLoggerGuard {
    let logger = create_default_root_logger(async_drain, chan_size);
    set_global_logger(logger)
}

/// Creates a root logger with default settings.
fn create_default_root_logger(async_drain: bool, chan_size: Option<usize>) -> Logger {
    let drain = GlogFormat::new(PlainDecorator::new(::std::io::stderr()), ErrorCategorizer).fuse();
    let logger = create_env_logger(drain);
    get_logger(async_drain, chan_size, logger)
}

/// Creates a logger that respects RUST_LOG environment variable
fn create_env_logger_with_level<D>(drain: D, level: FilterLevel) -> EnvLogger<D>
where
    D: Drain<Err = Never, Ok = ()> + Send + 'static,
{
    let mut builder = LogBuilder::new(drain);
    // Have the default logging level be 'Info'
    builder = builder.filter(None, level);

    // Apply directives from the "RUST_LOG" env var
    if let Ok(s) = ::std::env::var("RUST_LOG") {
        builder = builder.parse(&s);
    }
    builder.build()
}

/// Creates a logger that respects RUST_LOG environment variable
fn create_env_logger<D>(drain: D) -> EnvLogger<D>
where
    D: Drain<Err = Never, Ok = ()> + Send + 'static,
{
    // Have the default logging level be 'Info'
    create_env_logger_with_level(drain, FilterLevel::Info)
}

/// Creates a root logger with test settings: does not do output if test passes.
/// Caveat: cargo test does not capture output for non main thread. So this logger is not
/// very useful for multithreading scenarios.
fn create_test_root_logger() -> Logger {
    let drain = GlogFormat::new(TermDecorator::new().build(), ErrorCategorizer).fuse();
    let envlogger = create_env_logger_with_level(drain, FilterLevel::Debug);
    Logger::root(Mutex::new(envlogger).fuse(), o!())
}

// TODO: redo this
lazy_static! {
    static ref TESTING_ENVLOGGER_GUARD: GlobalLoggerGuard = {
        let logger = {
            if ::std::env::var("RUST_LOG").is_ok() {
                create_default_root_logger(false /* async */, None /* chan_size */)
            } else {
                Logger::root(Discard, o!())
            }
        };
        set_global_logger(logger)
    };

    static ref END_TO_END_TESTING_ENVLOGGER_GUARD: GlobalLoggerGuard = {
        let logger = create_test_root_logger();
        set_global_logger(logger)
    };
}

/// Create and setup default global logger following the env-logger conventions,
///  i.e. configured by environment variable RUST_LOG.
/// This is useful to make logging optional in unit tests.
pub fn try_init_for_testing() {
    ::lazy_static::initialize(&TESTING_ENVLOGGER_GUARD);
}

/// Create and setup default global logger for use in end to end testing.
pub fn init_for_e2e_testing() {
    ::lazy_static::initialize(&END_TO_END_TESTING_ENVLOGGER_GUARD);
}

fn get_logger<D>(is_async: bool, chan_size: Option<usize>, drain: D) -> Logger
where
    D: Drain<Err = Never, Ok = ()> + Send + 'static,
{
    if is_async {
        let async_builder = match chan_size {
            Some(chan_size_inner) => Async::new(drain).chan_size(chan_size_inner),
            None => Async::new(drain),
        };
        Logger::root(async_builder.build().fuse(), o!())
    } else {
        Logger::root(Mutex::new(drain).fuse(), o!())
    }
}
