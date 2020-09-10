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
    /// Do not run the benchmarks, but compile them
    #[structopt(long)]
    no_run: bool,
    #[structopt(name = "BENCHNAME", parse(from_os_str))]
    benchname: Option<OsString>,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(mut args: Args, xctx: XContext) -> Result<()> {
    args.args.extend(args.benchname.clone());
    let config = xctx.config();

    let mut direct_args = Vec::new();
    if args.no_run {
        direct_args.push(OsString::from("--no-run"));
    };

    let cmd = CargoCommand::Bench(config.cargo_config(), direct_args.as_slice(), &args.args);
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
