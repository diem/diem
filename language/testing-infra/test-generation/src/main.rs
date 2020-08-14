// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::env;
use structopt::StructOpt;
use test_generation::{config::Args, run_generation};

fn setup_log() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    ::libra_logger::Logger::new().init();
}

pub fn main() {
    setup_log();
    let args = Args::from_args();
    run_generation(args);
}
