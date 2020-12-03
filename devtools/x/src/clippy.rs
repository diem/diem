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
    let mut pass_through_args = vec!["-D".into(), "warnings".into()];
    for lint in xctx.config().allowed_clippy_lints() {
        pass_through_args.push("-A".into());
        pass_through_args.push(lint.into());
    }
    for lint in xctx.config().warn_clippy_lints() {
        pass_through_args.push("-W".into());
        pass_through_args.push(lint.into());
    }
    pass_through_args.extend(args.args);

    let cmd = CargoCommand::Clippy(xctx.config().cargo_config(), &pass_through_args);
    check::run_with(cmd, args.check_args, &xctx)
}
