// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand},
    config::Config,
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

pub fn run(mut args: Args, config: Config) -> Result<()> {
    args.args.extend(args.benchname.clone());

    let cmd = CargoCommand::Bench(&args.args);
    let base_args = CargoArgs {
        all_features: true,
        ..CargoArgs::default()
    };

    if !args.package.is_empty() {
        let run_together = args.package.iter().filter(|p| !config.is_exception(p));
        let run_separate = args.package.iter().filter_map(|p| {
            config.package_exceptions().get(p).map(|e| {
                (
                    p,
                    CargoArgs {
                        all_features: e.all_features,
                        ..base_args
                    },
                )
            })
        });
        cmd.run_on_packages_together(run_together, &base_args)?;
        cmd.run_on_packages_separate(run_separate)?;
    } else if utils::project_is_root()? {
        cmd.run_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &base_args,
        )?;
        cmd.run_on_packages_separate(config.package_exceptions().iter().map(|(name, pkg)| {
            (
                name,
                CargoArgs {
                    all_features: pkg.all_features,
                    ..base_args
                },
            )
        }))?;
    } else {
        let package = utils::get_local_package()?;
        let all_features = config
            .package_exceptions()
            .get(&package)
            .map(|pkg| pkg.all_features)
            .unwrap_or(true);

        cmd.run_on_local_package(&CargoArgs {
            all_features,
            ..base_args
        })?;
    }
    Ok(())
}
