// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use structopt::StructOpt;
use test_generation::{config::Args, run_generation};

pub fn main() {
    let args = Args::from_args();
    run_generation(args);
}
