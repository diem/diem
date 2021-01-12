// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

use test_explore::analyzer_e2e::AnalyzerE2E;

#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct Args {
    /// type of testing trace to analyze
    #[structopt(subcommand)]
    cmd: Command,

    /// directory for the txn traces
    #[structopt(
        short = "o",
        long,
        default_value = ".",
        parse(from_os_str),
        global = true
    )]
    output: PathBuf,
}

#[derive(StructOpt)]
pub enum Command {
    /// run on e2e tests
    #[structopt(name = "e2e")]
    E2E {
        /// directory for the txn traces
        #[structopt(parse(from_os_str))]
        trace_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let Args { cmd, output } = Args::from_args();
    match cmd {
        Command::E2E { trace_dir } => {
            let analyzer = AnalyzerE2E::new(output);
            analyzer.run_on_trace_dir(trace_dir)
        }
    }
}
