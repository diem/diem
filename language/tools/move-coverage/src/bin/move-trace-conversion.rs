// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_coverage::coverage_map::{output_map_to_file, CoverageMap};
use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move VM Coverage",
    about = "Creates a coverage map from the raw data collected from the Move VM"
)]
struct Args {
    /// The path to the input file
    #[structopt(long = "input-file-path", short = "f")]
    pub input_file_path: String,
    /// The path to the output file location
    #[structopt(long = "output-file-path", short = "o")]
    pub output_file_path: String,
    /// Add traces from `input_file_path` to an existing coverage map at `update_coverage_map`
    #[structopt(long = "update-coverage-map", short = "u")]
    pub update_coverage_map: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let input_path = Path::new(&args.input_file_path);
    let output_path = Path::new(&args.output_file_path);
    let coverage_map = if let Some(old_coverage_path) = &args.update_coverage_map {
        let path = Path::new(&old_coverage_path);
        let old_coverage_map = CoverageMap::from_binary_file(&path);
        old_coverage_map.update_coverage_from_trace_file(&input_path)
    } else {
        CoverageMap::from_trace_file(&input_path)
    };

    output_map_to_file(&output_path, &coverage_map)
        .expect("Unable to serialize coverage map to output file")
}
