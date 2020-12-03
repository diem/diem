// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_coverage::coverage_map::{output_map_to_file, CoverageMap, TraceMap};
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
    #[structopt(long = "update", short = "u")]
    pub update: Option<String>,
    /// Collect structured trace instead of aggregated coverage information
    #[structopt(long = "use-trace-map", short = "t")]
    pub use_trace_map: bool,
}

fn main() {
    let args = Args::from_args();
    let input_path = Path::new(&args.input_file_path);
    let output_path = Path::new(&args.output_file_path);

    if !args.use_trace_map {
        let coverage_map = if let Some(old_coverage_path) = &args.update {
            let path = Path::new(&old_coverage_path);
            let old_coverage_map = CoverageMap::from_binary_file(&path);
            old_coverage_map.update_coverage_from_trace_file(&input_path)
        } else {
            CoverageMap::from_trace_file(&input_path)
        };

        output_map_to_file(&output_path, &coverage_map)
            .expect("Unable to serialize coverage map to output file")
    } else {
        let trace_map = if let Some(old_trace_path) = &args.update {
            let path = Path::new(&old_trace_path);
            let old_trace_map = TraceMap::from_binary_file(&path);
            old_trace_map.update_from_trace_file(&input_path)
        } else {
            TraceMap::from_trace_file(&input_path)
        };

        output_map_to_file(&output_path, &trace_map)
            .expect("Unable to serialize trace map to output file")
    }
}
