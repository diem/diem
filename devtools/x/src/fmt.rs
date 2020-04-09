// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::Cargo, context::XContext, Result};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long)]
    /// Run in 'check' mode. Exits with 0 if input is
    /// formatted correctly. Exits with 1 and prints a diff if
    /// formatting is required.
    check: bool,

    #[structopt(long)]
    /// Run check on all packages in the workspace
    workspace: bool,

    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    /// Pass through args to rustfmt
    args: Vec<OsString>,
}

pub fn run(args: Args, _xctx: XContext) -> Result<()> {
    // Hardcode that we want imports merged
    let mut pass_through_args = vec!["--config".into(), "merge_imports=true".into()];

    if args.check {
        pass_through_args.push("--check".into());
    }

    pass_through_args.extend(args.args);

    let mut cmd = Cargo::new("fmt");

    if args.workspace {
        // cargo fmt doesn't have a --workspace flag, instead it uses the
        // old --all flag
        cmd.all();
    }

    cmd.pass_through(&pass_through_args).run()
}
