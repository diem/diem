// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_command_line_common::files::MOVE_ERROR_DESC_EXTENSION;
use move_explain::parse_and_explain;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Move Explain",
    about = "Explain Move abort codes. Errors are defined as a global category + module-specific reason for the error."
)]
struct Args {
    /// The location (module id) returned with a `MoveAbort` error
    #[structopt(long = "location", short = "l")]
    location: String,
    /// The abort code returned with a `MoveAbort` error
    #[structopt(long = "abort-code", short = "a")]
    abort_code: u64,
    /// Path to the error code mapping file
    #[structopt(long = MOVE_ERROR_DESC_EXTENSION, short = "e", parse(from_os_str))]
    errmap_path: PathBuf,
}

fn main() {
    let args = Args::from_args();

    parse_and_explain(
        &mut std::io::stdout(),
        &args.location,
        args.abort_code,
        &args.errmap_path,
    );
}
