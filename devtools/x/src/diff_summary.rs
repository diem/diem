// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::context::XContext;
use guppy::graph::summaries::{diff::SummaryDiff, Summary};
use std::{fs, path::PathBuf};
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    pub enum OutputFormat {
        Toml,
        Json,
        Text,
    }
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(name = "BASE_SUMMARY")]
    /// Path to the base summary
    base_summary: PathBuf,
    #[structopt(name = "COMPARE_SUMMARY")]
    /// Path to the comparison summary
    compare_summary: PathBuf,
    #[structopt(name = "OUTPUT_FORMAT", default_value = "Text")]
    /// optionally, output can be formated as json or toml
    output_format: OutputFormat,
}

pub fn run(args: Args, _xctx: XContext) -> crate::Result<()> {
    let base_summary_text = fs::read_to_string(&args.base_summary)?;
    let base_summary = Summary::parse(&base_summary_text)?;
    let compare_summary_text = fs::read_to_string(&args.compare_summary)?;
    let compare_summary = Summary::parse(&compare_summary_text)?;

    let summary_diff = SummaryDiff::new(&base_summary, &compare_summary);

    match args.output_format {
        OutputFormat::Json => println!("{}", serde_json::to_string(&summary_diff)?),
        OutputFormat::Toml => println!("{}", toml::to_string(&summary_diff)?),
        OutputFormat::Text => println!("{}", summary_diff.report()),
    };

    Ok(())
}
