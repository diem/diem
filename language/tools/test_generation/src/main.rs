// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_test_generation::{config::Args, run_generation};
use structopt::StructOpt;

pub fn main() {
    let args = Args::from_args();
    run_generation(args);
}
