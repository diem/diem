// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_fuzzer::FuzzTarget;
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra-Fuzzer Investigator",
    author = "The Libra Association",
    about = "Utility tool to investigate fuzzing artifacts"
)]
struct Args {
    /// Admission Control port to connect to.
    #[structopt(short = "i", long)]
    pub input_file: Option<String>,
}

fn main() {
    // args
    let args = Args::from_args();

    // check if it exists
    let input_file = PathBuf::from(args.input_file.expect("input file must be set via -i"));
    if !input_file.exists() {
        println!("usage: cargo run investigate -i <artifacts/.../corpus_input>");
        return;
    }

    // get target from path (input_file = .../<target>/<corpus_input>)
    let mut ancestors = input_file.ancestors();
    ancestors.next(); // skip full path
    let target_name = ancestors
        .next()
        .expect("input file should be inside target directory")
        .iter()
        .last()
        .unwrap()
        .to_str()
        .unwrap();
    let target = FuzzTarget::by_name(target_name)
        .unwrap_or_else(|| panic!("unknown fuzz target: {}", target_name));

    // run the target fuzzer on the file
    let data = fs::read(input_file).expect("failed to read artifact");
    target.fuzz(&data);
}
