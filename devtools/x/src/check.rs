// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand},
    context::XContext,
    utils, Result,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run check on the provided packages
    pub(crate) package: Vec<String>,
    #[structopt(long)]
    /// Run check on all packages in the workspace
    pub(crate) workspace: bool,
    #[structopt(long)]
    /// Run check on all targets of a package (lib, bin, test, example)
    pub(crate) all_targets: bool,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let cmd = CargoCommand::Check(xctx.config().cargo_config());
    run_with(cmd, args, &xctx)
}

pub fn run_with(cmd: CargoCommand<'_>, args: Args, xctx: &XContext) -> Result<()> {
    let base_args = CargoArgs {
        all_targets: args.all_targets,
    };

    if !args.package.is_empty() {
        cmd.run_on_packages(args.package.iter(), &base_args)?;
    } else if utils::project_is_root(&xctx.config().cargo_config())? || args.workspace {
        cmd.run_on_all_packages(&base_args)?;
    } else {
        cmd.run_on_local_package(&base_args)?;
    }
    Ok(())
}
