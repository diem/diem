// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cargo::CargoCommand, check, config::Config, Result};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    check_args: check::Args,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(args: Args, config: Config) -> Result<()> {
    let mut pass_through_args = vec!["-D".into(), "warnings".into()];
    for lint in config.allowed_clippy_lints() {
        pass_through_args.push("-A".into());
        pass_through_args.push(lint.into());
    }
    pass_through_args.extend(args.args);

    let cmd = CargoCommand::Clippy(&pass_through_args);
    check::run_with(cmd, args.check_args, config)
}
