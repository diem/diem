// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand},
    context::XContext,
    xcontext::{execution_location, project_metadata::Config},
    Result,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run check on the provided packages
    package: Vec<String>,
    #[structopt(long)]
    /// Run check on all packages in the workspace
    workspace: bool,
    #[structopt(long)]
    /// Run check on all targets of a package (lib, bin, test, example)
    all_targets: bool,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let cmd = CargoCommand::Check;
    run_with(cmd, args, xctx.config())
}

pub fn run_with(cmd: CargoCommand<'_>, args: Args, config: &Config) -> Result<()> {
    // If we've been asked to build all targets then we need to enable all_features so that
    // building the testing targets works
    let base_args = CargoArgs {
        all_features: args.all_targets,
        all_targets: args.all_targets,
    };

    if !args.package.is_empty() {
        let run_together = args.package.iter().filter(|p| !config.is_exception(p));
        let run_separate = args.package.iter().filter_map(|p| {
            config.package_exceptions().get(p).map(|e| {
                (
                    p,
                    CargoArgs {
                        all_features: if args.all_targets {
                            e.all_features
                        } else {
                            false
                        },
                        ..base_args
                    },
                )
            })
        });
        cmd.run_on_packages_together(run_together, &base_args)?;
        cmd.run_on_packages_separate(run_separate)?;
    } else if execution_location::project_is_root()? || args.workspace {
        cmd.run_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &base_args,
        )?;
        cmd.run_on_packages_separate(config.package_exceptions().iter().map(|(name, pkg)| {
            (
                name,
                CargoArgs {
                    all_features: if args.all_targets {
                        pkg.all_features
                    } else {
                        false
                    },
                    ..base_args
                },
            )
        }))?;
    } else {
        let package = execution_location::get_local_package()?;
        let cargo_args = if args.all_targets {
            let all_features = config
                .package_exceptions()
                .get(&package)
                .map(|pkg| pkg.all_features)
                .unwrap_or(true);
            CargoArgs {
                all_features,
                ..base_args
            }
        } else {
            base_args
        };

        cmd.run_on_local_package(&cargo_args)?;
    }
    Ok(())
}
