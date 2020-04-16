// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Incorrect number of parameters, expected an output path");
        process::exit(1);
    }

    generate_key::generate_and_save_key(&args[1]);
}
