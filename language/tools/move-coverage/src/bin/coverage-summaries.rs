// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use move_coverage::{
    coverage_map::{CoverageMap, TraceMap},
    summary::{self, ModuleSummary},
};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
use structopt::StructOpt;
use vm::file_format::CompiledModule;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move VM Coverage Summary",
    about = "Creates a coverage summary from the trace data collected from the Move VM"
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
    pub module_binary_path: Option<String>,
    /// Optional path for summaries. Printed to stdout if not present.
    #[structopt(long = "summary-path", short = "o")]
    pub summary_path: Option<String>,
    /// Whether function coverage summaries should be displayed
    #[structopt(long = "summarize-functions", short = "f")]
    pub summarize_functions: bool,
    /// The path to the standard library binary directory for Move
    #[structopt(long = "stdlib-path", short = "s")]
    pub stdlib_path: Option<String>,
    /// Whether path coverage should be derived (default is instruction coverage)
    #[structopt(long = "derive-path-coverage", short = "p")]
    pub derive_path_coverage: bool,
    /// Output CSV data of coverage
    #[structopt(long = "csv", short = "c")]
    pub csv_output: bool,
}

fn get_modules(args: &Args) -> Vec<CompiledModule> {
    let mut modules = Vec::new();
    if let Some(stdlib_path) = &args.stdlib_path {
        let stdlib_modules = fs::read_dir(stdlib_path).unwrap().map(|file| {
            let bytes = fs::read(file.unwrap().path()).unwrap();
            CompiledModule::deserialize(&bytes).expect("Module blob can't be deserialized")
        });
        modules.extend(stdlib_modules);
    }

    if let Some(module_binary_path) = &args.module_binary_path {
        let bytecode_bytes = fs::read(module_binary_path).expect("Unable to read bytecode file");
        let compiled_module = CompiledModule::deserialize(&bytecode_bytes)
            .expect("Module blob can't be deserialized");
        modules.push(compiled_module);
    }

    if modules.is_empty() {
        panic!("No modules provided for coverage checking")
    }

    modules
}

fn format_human_summary<M, F, W: Write>(
    args: &Args,
    coverage_map: &M,
    summary_func: F,
    summary_writer: &mut W,
) where
    F: Fn(&CompiledModule, &M) -> ModuleSummary,
{
    writeln!(summary_writer, "+-------------------------+").unwrap();
    writeln!(summary_writer, "| Move Coverage Summary   |").unwrap();
    writeln!(summary_writer, "+-------------------------+").unwrap();

    let mut total_covered = 0;
    let mut total_instructions = 0;

    for module in get_modules(&args).iter() {
        let coverage_summary = summary_func(&module, coverage_map);
        let (total, covered) = coverage_summary
            .summarize_human(summary_writer, args.summarize_functions)
            .unwrap();
        total_covered += covered;
        total_instructions += total;
    }

    writeln!(summary_writer, "+-------------------------+").unwrap();
    writeln!(
        summary_writer,
        "| % Move Coverage: {:.2}  |",
        (total_covered as f64 / total_instructions as f64) * 100f64
    )
    .unwrap();
    writeln!(summary_writer, "+-------------------------+").unwrap();
}

fn format_csv_summary<M, F, W: Write>(
    args: &Args,
    coverage_map: &M,
    summary_func: F,
    summary_writer: &mut W,
) where
    F: Fn(&CompiledModule, &M) -> ModuleSummary,
{
    writeln!(summary_writer, "ModuleName,FunctionName,Covered,Uncovered").unwrap();

    for module in get_modules(&args).iter() {
        let coverage_summary = summary_func(&module, coverage_map);
        coverage_summary.summarize_csv(summary_writer).unwrap();
    }
}

fn main() {
    let args = Args::from_args();
    let input_trace_path = Path::new(&args.input_trace_path);

    let mut summary_writer: Box<dyn Write> = match &args.summary_path {
        Some(x) => {
            let path = Path::new(x);
            Box::new(File::create(&path).unwrap())
        }
        None => Box::new(io::stdout()),
    };

    if args.derive_path_coverage {
        let trace_map = if args.is_raw_trace_file {
            TraceMap::from_trace_file(&input_trace_path)
        } else {
            TraceMap::from_binary_file(&input_trace_path)
        };
        if !args.csv_output {
            format_human_summary(
                &args,
                &trace_map,
                summary::summarize_path_cov,
                &mut summary_writer,
            )
        } else {
            format_csv_summary(
                &args,
                &trace_map,
                summary::summarize_path_cov,
                &mut summary_writer,
            )
        }
    } else {
        let coverage_map = if args.is_raw_trace_file {
            CoverageMap::from_trace_file(&input_trace_path)
        } else {
            CoverageMap::from_binary_file(&input_trace_path)
        };
        let unified_exec_map = coverage_map.to_unified_exec_map();
        if !args.csv_output {
            format_human_summary(
                &args,
                &unified_exec_map,
                summary::summarize_inst_cov,
                &mut summary_writer,
            )
        } else {
            format_csv_summary(
                &args,
                &unified_exec_map,
                summary::summarize_inst_cov,
                &mut summary_writer,
            )
        }
    }
}
