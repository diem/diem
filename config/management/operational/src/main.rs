// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]
use diem_operational_tool::command::{Command, ResultWrapper};
use std::process::exit;
use structopt::StructOpt;

fn main() {
    let result = Command::from_args().execute();

    match result {
        Ok(val) => println!("{}", val),
        Err(err) => {
            let result: ResultWrapper<()> = ResultWrapper::Error(err.to_string());
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            exit(1);
        }
    }
}
