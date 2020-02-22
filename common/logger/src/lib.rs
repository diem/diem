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
//! let _g = libra_logger::set_global_logger(false /* async */, Some(256));
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
use once_cell::sync::Lazy;
use slog::{o, Discard, Drain, FilterLevel, Logger, Never};
pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
use slog_async::Async;
use slog_envlogger::{EnvLogger, LogBuilder};
use slog_scope::GlobalLoggerGuard;
pub use slog_scope::{crit, debug, error, info, trace, warn};
use slog_term::{PlainDecorator, TermDecorator};
use std::sync::Mutex;

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use crate::security::{security_log, SecurityEvent};
    pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
    pub use slog_scope::{crit, debug, error, info, trace, warn};
}

pub use simple_logger::{set_simple_logger, set_simple_logger_prefix};

/// Creates and sets the global logger at the default filter level.
/// Caller must keep the returned guard alive.
pub fn set_global_logger(async_drain: bool, chan_size: Option<usize>) -> GlobalLoggerGuard {
    set_global_logger_with_level(async_drain, chan_size, FilterLevel::Info)
}

/// Creates and sets the global logger with the given filter level.
pub fn set_global_logger_with_level(
    async_drain: bool,
    chan_size: Option<usize>,
    filter_level: FilterLevel,
) -> GlobalLoggerGuard {
    let drain = GlogFormat::new(PlainDecorator::new(::std::io::stderr()), ErrorCategorizer).fuse();
    let env_logger = create_env_logger_with_level(drain, filter_level);
    let logger = get_logger(async_drain, chan_size, env_logger);
    slog_scope::set_global_logger(logger)
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

/// slog_scope::set_global_logger() returns a guard, which must be kept in
/// scope. If the guard is no longer scoped, Slog panics, thinking that
/// set_global_logger() has not been called. As such, to support logging
/// during tests, we define and keep these guards at global scope. This is
/// ugly, but it works.
static TESTING_ENVLOGGER_GUARD: Lazy<GlobalLoggerGuard> = Lazy::new(|| {
    if ::std::env::var("RUST_LOG").is_ok() {
        set_global_logger(false /* async */, None /* chan_size */)
    } else {
        let logger = Logger::root(Discard, o!());
        slog_scope::set_global_logger(logger)
    }
});

/// Creates a root logger with test settings: does not do output if test passes.
/// Caveat: cargo test does not capture output for non main thread. So this logger is not
/// very useful for multithreading scenarios.
static END_TO_END_TESTING_ENVLOGGER_GUARD: Lazy<GlobalLoggerGuard> = Lazy::new(|| {
    let drain = GlogFormat::new(TermDecorator::new().build(), ErrorCategorizer).fuse();
    let env_logger = create_env_logger_with_level(drain, FilterLevel::Debug);
    let logger = Logger::root(Mutex::new(env_logger).fuse(), o!());
    slog_scope::set_global_logger(logger)
});

/// Create and setup default global logger following the env-logger conventions,
///  i.e. configured by environment variable RUST_LOG.
/// This is useful to make logging optional in unit tests.
pub fn try_init_for_testing() {
    Lazy::force(&TESTING_ENVLOGGER_GUARD);
}

/// Create and setup default global logger for use in end to end testing.
pub fn init_for_e2e_testing() {
    Lazy::force(&END_TO_END_TESTING_ENVLOGGER_GUARD);
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
