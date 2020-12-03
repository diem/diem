// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::CargoCommand, check, context::XContext, Result};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    check_args: check::Args,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let mut pass_through_args = vec![];
    pass_through_args.extend(args.args);

    let with_all_targets = check::Args {
        all_targets: true,
        ..args.check_args
    };

    let cmd = CargoCommand::Fix(xctx.config().cargo_config(), &pass_through_args);
    check::run_with(cmd, with_all_targets, &xctx)
}
