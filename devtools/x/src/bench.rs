// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand},
    context::XContext,
    utils, Result,
};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run test on the provided packages
    package: Vec<String>,
    #[structopt(name = "BENCHNAME", parse(from_os_str))]
    benchname: Option<OsString>,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(mut args: Args, xctx: XContext) -> Result<()> {
    args.args.extend(args.benchname.clone());
    let config = xctx.config();

    let cmd = CargoCommand::Bench(config.cargo_config(), &args.args);
    let base_args = CargoArgs::default();

    if !args.package.is_empty() {
        cmd.run_on_packages(args.package.iter(), &base_args)?;
    } else if utils::project_is_root(&xctx)? {
        cmd.run_on_all_packages(&base_args)?;
    } else {
        cmd.run_on_local_package(&base_args)?;
    };
    Ok(())
}
