// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand, SelectedPackageArgs},
    context::XContext,
    Result,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
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
    let packages = args.package_args.to_selected_packages(xctx)?;
    cmd.run_on_packages(&packages, &base_args)
}
