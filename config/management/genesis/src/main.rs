// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use libra_genesis_tool::command::Command;
use structopt::StructOpt;

fn main() {
    match Command::from_args().execute() {
        Ok(output) => println!("{}", output),
        Err(err) => {
            println!("Operation unsuccessful: {}", err);
            std::process::exit(1);
        }
    }
}
