// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use slog::{o, Drain};
use std::env;
use structopt::StructOpt;
use test_generation::{config::Args, run_generation};

fn setup_log(log_file: &Option<String>) {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    if let Some(log_file_path) = log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_file_path)
            .unwrap();
        let flog = slog_json::Json::default(file).fuse();
        let drain = slog_envlogger::new(flog);
        let drain = std::sync::Mutex::new(drain).fuse();
        let logger = slog::Logger::root(drain, o!());
        let logger_guard = slog_scope::set_global_logger(logger);
        std::mem::forget(logger_guard);
    } else {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_envlogger::new(drain);
        let drain = std::sync::Mutex::new(drain).fuse();
        let logger = slog::Logger::root(drain, o!());
        let logger_guard = slog_scope::set_global_logger(logger);
        std::mem::forget(logger_guard);
    }
}

pub fn main() {
    let args = Args::from_args();
    setup_log(&args.log_file_path);
    run_generation(args);
}
