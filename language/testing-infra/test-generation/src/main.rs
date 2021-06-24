// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use structopt::StructOpt;
use test_generation::{config::Args, run_generation};

fn setup_log() {
    tracing::subscriber::set_global_default(tracing_subscriber::FmtSubscriber::new()).unwrap();
}

pub fn main() {
    setup_log();
    let args = Args::from_args();
    run_generation(args);
}
