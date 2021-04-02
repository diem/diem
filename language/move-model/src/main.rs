// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use structopt::StructOpt;

use move_model::{run_model_builder, run_spec_instrumenter};

#[derive(StructOpt)]
struct Options {
    /// The source files to check and compile
    #[structopt(global = true)]
    source_files: Vec<String>,

    /// The library files needed as dependencies
    #[structopt(short = "d", global = true)]
    dependencies: Vec<String>,

    /// If set, show warnings and fail if there are warnings in the operations
    #[structopt(short = "s", long = "strict")]
    strict: bool,

    /// The subcommand
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    /// Build and check the move model
    #[structopt(name = "build")]
    Build,

    /// Instrument the spec into move programs
    #[structopt(name = "instrument")]
    Instrument {
        /// The output directory
        #[structopt(short = "o", default_value = "build")]
        out_dir: String,

        /// If set, dump the program AST before and after spec instrumentation
        #[structopt(long = "dump-ast")]
        dump_ast: bool,
    },
}

pub fn main() -> Result<()> {
    let Options {
        source_files,
        dependencies,
        strict,
        command,
    } = Options::from_args();

    let env = match command {
        Command::Build => run_model_builder(&source_files, &dependencies)?,
        Command::Instrument { out_dir, dump_ast } => {
            run_spec_instrumenter(&source_files, &dependencies, &out_dir, dump_ast)?
        }
    };

    let mut error_writer = StandardStream::stderr(ColorChoice::Auto);
    if env.has_errors() {
        env.report_errors(&mut error_writer);
        bail!("Operation failed with errors");
    }
    if strict && env.has_warnings() {
        env.report_warnings(&mut error_writer);
        bail!("Operation failed with warnings");
    }
    Ok(())
}
