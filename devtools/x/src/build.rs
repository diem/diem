// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    cargo::{build_args::BuildArgs, selected_package::SelectedPackageArgs, CargoCommand},
    context::XContext,
    Result,
};
use log::info;
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    package_args: SelectedPackageArgs,
    #[structopt(flatten)]
    build_args: BuildArgs,
    #[structopt(long, parse(from_os_str))]
    /// Copy final artifacts to this directory (unstable)
    out_dir: Option<OsString>,
    #[structopt(long)]
    /// Output the build plan in JSON (unstable)
    build_plan: bool,
}

pub fn convert_args(args: &Args) -> Vec<OsString> {
    let mut direct_args = Vec::new();
    args.build_args.add_args(&mut direct_args);
    if let Some(out_dir) = &args.out_dir {
        direct_args.push(OsString::from("--out-dir"));
        direct_args.push(OsString::from(out_dir));
    };
    if args.build_plan {
        direct_args.push(OsString::from("--build-plan"));
    };

    direct_args
}

pub fn run(args: Box<Args>, xctx: XContext) -> Result<()> {
    info!("Build plan: {}", args.build_plan);

    let direct_args = convert_args(&args);

    let cmd = CargoCommand::Build {
        cargo_config: xctx.config().cargo_config(),
        direct_args: direct_args.as_slice(),
        args: &[],
        env: &[],
        skip_sccache: false,
    };

    let packages = args.package_args.to_selected_packages(&xctx)?;
    cmd.run_on_packages(&packages)
}
