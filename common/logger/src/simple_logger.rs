// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use slog::{o, Drain, FilterLevel, Level, Logger, OwnedKVList, Record};
use slog_envlogger::LogBuilder;
use slog_scope::set_global_logger;
use std::{cell::RefCell, io, mem, sync::Mutex};

/// Simple logger mostly intended for use in test code
/// It contains bunch of boilerplate hacks to make output of test look not too verbose(but still
/// have some logs)
///
/// Simple logger output logs into stdout in simple format:
/// <L><Time><Prefix><Message>
/// Where
/// L: log level(single letter)
/// Time: Short time: minutes, seconds, milliseconds
/// Prefix: thread local prefix that can be set with set_simple_logger_prefix
/// Message: log message
///
/// It sets default log level to INFO for all modules, and debug for specific module,
/// usually module that unit test belongs to
///
/// RUST_LOG env variable can be used to overwrite log levels
pub fn set_simple_logger(debug_module: &str) {
    mem::forget(set_global_logger(create_simple_logger(debug_module)));
}

fn create_simple_logger(debug_module: &str) -> Logger {
    let drain = SimpleDrain.fuse();
    let mut builder = LogBuilder::new(drain);
    builder = builder.filter(None, FilterLevel::Info);
    builder = builder.filter(Some(debug_module), FilterLevel::Debug);

    if let Ok(s) = ::std::env::var("RUST_LOG") {
        builder = builder.parse(&s);
    }

    let envlogger = builder.build();

    Logger::root(Mutex::new(envlogger).fuse(), o!())
}

struct SimpleDrain;

impl Drain for SimpleDrain {
    type Ok = ();
    type Err = io::Error;

    fn log(&self, record: &Record, _values: &OwnedKVList) -> Result<Self::Ok, Self::Err> {
        let now = chrono::Local::now();
        PREFIX.with(|prefix| {
            let borrowed = prefix.borrow();
            let prefix = if let Some(ref prefix) = *borrowed {
                &prefix[..]
            } else {
                ""
            };
            println!(
                "{}{}{} {}",
                char_for_level(record.level()),
                now.format("%M:%S%.6f"),
                prefix,
                record.msg()
            )
        });
        Ok(())
    }
}

fn char_for_level(l: Level) -> char {
    match l {
        Level::Critical => 'C',
        Level::Error => 'E',
        Level::Warning => 'W',
        Level::Info => 'I',
        Level::Debug => 'D',
        Level::Trace => 'T',
    }
}

thread_local! {
    pub static PREFIX: RefCell<Option<String>> = RefCell::new(None);
}

/// Sets thread local prefix for log lines
/// This is useful for tests that simulates multiple machines
/// Logger prefix can be used to separate logs from those multiple simulated instances
pub fn set_simple_logger_prefix(id: String) {
    PREFIX.with(|x| x.replace(Option::Some(id)));
}
