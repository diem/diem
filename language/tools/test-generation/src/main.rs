// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use slog::{o, Drain};
use std::env;
use structopt::StructOpt;
use test_generation::{config::Args, run_generation};

fn setup_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    let decorator = slog_term::PlainDecorator::new(std::io::stdout());
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain);
    let drain = std::sync::Mutex::new(drain).fuse();
    let logger = slog::Logger::root(drain, o!());
    let logger_guard = slog_scope::set_global_logger(logger);
    std::mem::forget(logger_guard);
}

pub fn main() {
    setup_log();
    let args = Args::from_args();
    run_generation(args);
}
