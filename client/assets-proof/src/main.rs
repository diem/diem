// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_assets_proof::{Args, ResultWrapper};
use std::process::exit;
use structopt::StructOpt;

fn main() {
    let result = Args::from_args().exec();

    match result {
        Ok(out) => println!("{}", out),
        Err(err) => {
            let result: ResultWrapper<()> = ResultWrapper::new(Err(err));
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            exit(1);
        }
    }
}
