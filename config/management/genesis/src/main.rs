// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use diem_genesis_tool::command::Command;
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
