// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use bytecode_source_map::utils::{remap_owned_loc_to_loc, source_map_from_file, OwnedLoc};
use move_coverage::{coverage_map::CoverageMap, source_coverage::SourceCoverageBuilder};
use std::{
    fs,
    fs::File,
    io::{self, Write},
    path::Path,
};
use structopt::StructOpt;
use vm::file_format::CompiledModule;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Source Coverage",
    about = "Annotate Move Source Code with Coverage Information"
)]
struct Args {
    /// The path to the coverage map or trace file
    #[structopt(long = "input-trace-path", short = "t")]
    pub input_trace_path: String,
    /// Whether the passed-in file is a raw trace file or a serialized coverage map
    #[structopt(long = "is-raw-trace", short = "r")]
    pub is_raw_trace_file: bool,
    /// The path to the module binary
    #[structopt(long = "module-path", short = "b")]
    pub module_binary_path: String,
    /// The path to the source file
    #[structopt(long = "source-path", short = "s")]
    pub source_file_path: String,
    /// Optional path to save coverage. Printed to stdout if not present.
    #[structopt(long = "coverage-path", short = "o")]
    pub coverage_path: Option<String>,
}

fn main() {
    let args = Args::from_args();
    let source_map_extension = "mvsm";
    let coverage_map = if args.is_raw_trace_file {
        CoverageMap::from_trace_file(&args.input_trace_path)
    } else {
        CoverageMap::from_binary_file(&args.input_trace_path)
    };

    let bytecode_bytes = fs::read(&args.module_binary_path).expect("Unable to read bytecode file");
    let compiled_module =
        CompiledModule::deserialize(&bytecode_bytes).expect("Module blob can't be deserialized");

    let source_map = source_map_from_file::<OwnedLoc>(
        &Path::new(&args.module_binary_path).with_extension(source_map_extension),
    )
    .map(remap_owned_loc_to_loc)
    .unwrap();
    let source_path = Path::new(&args.source_file_path);
    let source_cov = SourceCoverageBuilder::new(&compiled_module, &coverage_map, &source_map);

    let mut coverage_writer: Box<dyn Write> = match &args.coverage_path {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(&path).unwrap())
        }
        None => Box::new(io::stdout()),
    };

    source_cov
        .compute_source_coverage(&source_path)
        .output_source_coverage(&mut coverage_writer)
        .unwrap();
}
