// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use itertools::Itertools;
use prover_lab::benchmark::benchmark;

fn main() {
    let args = std::env::args().collect_vec();
    if args.len() < 2 || args.len() > 1 && matches!(args[1].as_str(), "-h" | "--help") {
        println!("prover-lab: please specify which tool to call. Available tools: `bench`.");
        println!("Use `prover-lab <tool> -h` for tool specific information.");
        std::process::exit(1);
    } else {
        match args[1].as_str() {
            "bench" => benchmark(&args[1..]),
            _ => {
                println!("prover-lab: unknown tool `{}`", args[1]);
                std::process::exit(2);
            }
        }
    }
}
