// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::context::XContext;
use guppy::graph::summaries::{diff::SummaryDiff, Summary};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(name = "BASE_SUMMARY")]
    /// Path to the base summary
    base_summary: PathBuf,
    #[structopt(name = "COMPARE_SUMMARY")]
    /// Path to the comparison summary
    compare_summary: PathBuf,
    #[structopt(name = "OUTPUT_FORMAT")]
    /// optionally, output can be formated as json or toml
    output_format: Option<String>,
}

pub fn run(args: Args, _xctx: XContext) -> crate::Result<()> {
    let base_summary_text = fs::read_to_string(&args.base_summary)?;
    let base_summary = Summary::parse(&base_summary_text)?;
    let compare_summary_text = fs::read_to_string(&args.compare_summary)?;
    let compare_summary = Summary::parse(&compare_summary_text)?;

    let summary_diff = SummaryDiff::new(&base_summary, &compare_summary);

    match args.output_format.as_deref() {
        Some("json") => println!("{}", serde_json::to_string(&summary_diff)?),
        Some("toml") => println!("{}", toml::to_string(&summary_diff)?),
        _ => println!("{}", summary_diff.report()),
    };

    Ok(())
}
