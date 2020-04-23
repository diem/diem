// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_management::Command;
use structopt::StructOpt;

fn main() {
    let args = Command::from_args();
    println!("{}", args.verify());
}
