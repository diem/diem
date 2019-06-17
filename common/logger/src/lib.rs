// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A default logger for Libra project.
//!
//! ## Usage
//!
//! ```rust, no_run
//! use logger::prelude::*;
//!
//! pub fn main() {
//!     let _g = logger::set_default_global_logger(false /* async */, Some(256));
//!     info!("Starting...");
//! }
//! ```

mod collector_serializer;
mod glog_format;
mod http_local_slog_drain;
mod http_log_client;
mod kv_categorizer;
mod security;
mod simple_logger;

use crate::{
    http_local_slog_drain::HttpLocalSlogDrain, http_log_client::HttpLogClient,
    kv_categorizer::ErrorCategorizer,
};
use crossbeam::atomic::ArcCell;
use failure::prelude::*;
use glog_format::GlogFormat;
use lazy_static::lazy_static;
use slog::{o, Discard, Drain, FilterLevel, Logger, Never};
pub use slog::{slog_crit, slog_debug, slog_error, slog_info, slog_trace, slog_warn};
use slog_async::Async;
use slog_envlogger::{EnvLogger, LogBuilder};
pub use slog_scope::{crit, debug, error, info, trace, warn};
use slog_scope::{set_global_logger, GlobalLoggerGuard};
use slog_term::{PlainDecorator, TermDecorator};
use std::sync::{Arc, Mutex};

/// Logger prelude which includes all logging macros.
pub mod prelude {
    pub use crate::{
        log_collector_crit, log_collector_debug, log_collector_error, log_collector_info,
        log_collector_trace, log_collector_warn,
        security::{security_log, SecurityEvent},
    };
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

lazy_static! {
    static ref GLOBAL_LOG_COLLECTOR: ArcCell<Logger> =
        ArcCell::new(Arc::new(Logger::root(Discard, o!())));
}

#[derive(Clone, Debug)]
pub enum LoggerType {
    // Logger sending data to http destination. Data includes endpoint.
    Http(String),
    // Logger sending data to stdio and stderr, which will be used along with
    // kv_categorizer
    StdOutput,
}

pub fn set_global_log_collector(collector: LoggerType, is_async: bool, chan_size: Option<usize>) {
    // Log collector should be available at this time.
    let log_collector = get_log_collector(collector, is_async, chan_size).unwrap();
    GLOBAL_LOG_COLLECTOR.set(Arc::new(log_collector));
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

// Get external logger according to config.
fn get_log_collector(
    collector: LoggerType,
    is_async: bool,
    chan_size: Option<usize>,
) -> Result<Logger> {
    match collector {
        LoggerType::Http(http_endpoint) => {
            let client = HttpLogClient::new(http_endpoint);
            let drain = HttpLocalSlogDrain::new(client?);
            Ok(get_logger(is_async, chan_size, drain))
        }
        LoggerType::StdOutput => Ok(create_default_root_logger(is_async, chan_size)),
    }
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

/// Access the `Global Logger Collector` for the current logging scope
///
/// This function doesn't have to clone the Logger
/// so it might be a bit faster.
pub fn with_logger<F, R>(f: F) -> R
where
    F: FnOnce(&Logger) -> R,
{
    f(&(*GLOBAL_LOG_COLLECTOR.get()))
}

/// Log a critical level message using current log collector
#[macro_export]
macro_rules! log_collector_crit( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_crit![logger, $($args)+])
};);
/// Log a error level message using current log collector
#[macro_export]
macro_rules! log_collector_error( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_error![logger, $($args)+])
};);
/// Log a warning level message using current log collector
#[macro_export]
macro_rules! log_collector_warn( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_warn![logger, $($args)+])
};);
/// Log a info level message using current log collector
#[macro_export]
macro_rules! log_collector_info( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_info![logger, $($args)+])
};);
/// Log a debug level message using current log collector
#[macro_export]
macro_rules! log_collector_debug( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_debug![logger, $($args)+])
};);
/// Log a trace level message using current log collector
#[macro_export]
macro_rules! log_collector_trace( ($($args:tt)+) => {
    $crate::with_logger(|logger| slog_trace![logger, $($args)+])
};);
