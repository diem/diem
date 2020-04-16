// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{CargoArgs, CargoCommand},
    context::XContext,
    utils,
    utils::project_root,
    Result,
};
use log::info;
use std::{
    ffi::OsString,
    process::{Command, Stdio},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short, number_of_values = 1)]
    /// Run test on the provided packages
    package: Vec<String>,
    #[structopt(long, short)]
    /// Only run unit tests
    unit: bool,
    #[structopt(long)]
    /// Do not fast fail the run if tests (or test executables) fail
    no_fail_fast: bool,
    #[structopt(long)]
    /// Do not run tests, only compile the test executables
    no_run: bool,
    #[structopt(long, parse(from_os_str))]
    /// Directory to output HTML coverage report to (using grcov)
    html_cov_dir: Option<OsString>,
    #[structopt(name = "TESTNAME", parse(from_os_str))]
    testname: Option<OsString>,
    #[structopt(name = "ARGS", parse(from_os_str), last = true)]
    args: Vec<OsString>,
}

pub fn run(mut args: Args, xctx: XContext) -> Result<()> {
    args.args.extend(args.testname.clone());
    let config = xctx.config();

    let env_vars: &[(&str, &str)] = if args.html_cov_dir.is_some() {
        info!("Running \"cargo clean\" before collecting coverage");
        let mut clean_cmd = Command::new("cargo");
        clean_cmd.arg("clean");
        clean_cmd.output()?;
        &[
            // A way to use -Z (unstable) flags with the stable compiler. See below.
            ("RUSTC_BOOTSTRAP", "1"),
            // Recommend setting for grcov, avoids using the cargo cache.
            ("CARGO_INCREMENTAL", "0"),
            // Recommend flags for use with grcov, with these flags removed: -Copt-level=0, -Clink-dead-code.
            // for more info see:  https://github.com/mozilla/grcov#example-how-to-generate-gcda-fiels-for-a-rust-project
            (
                "RUSTFLAGS",
                "-Zprofile -Ccodegen-units=1 -Coverflow-checks=off -Zno-landing-pads",
            ),
        ]
    } else {
        &[]
    };

    let mut direct_args = Vec::new();
    if args.no_run {
        direct_args.push(OsString::from("--no-run"));
    };
    if args.no_fail_fast {
        direct_args.push(OsString::from("--no-fail-fast"));
    };

    let cmd = CargoCommand::Test {
        direct_args: direct_args.as_slice(),
        args: &args.args,
        env: &env_vars,
    };

    // When testing, by deafult we want to turn on all features to ensure that the fuzzing features
    // are flipped on
    let base_args = CargoArgs {
        all_features: true,
        ..CargoArgs::default()
    };

    if args.unit {
        cmd.run_with_exclusions(
            config.package_exceptions().iter().map(|(p, _)| p),
            &base_args,
        )?;
        cmd.run_on_packages_separate(
            config
                .package_exceptions()
                .iter()
                .filter(|(_, pkg)| !pkg.system)
                .map(|(name, pkg)| {
                    (
                        name,
                        CargoArgs {
                            all_features: pkg.all_features,
                            ..base_args
                        },
                    )
                }),
        )?;
    } else if !args.package.is_empty() {
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
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
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

    if args.html_cov_dir.is_some() {
        let debug_dir = project_root().join("target/debug/");
        let mut grcov_html = Command::new("grcov");
        grcov_html
            //output file from coverage: gcda files
            .arg(debug_dir.as_os_str())
            //source code location
            .arg("-s")
            .arg(project_root().as_os_str())
            //html output
            .arg("-t")
            .arg("html")
            .arg("--llvm")
            .arg("--branch")
            .arg("--ignore")
            .arg("/*")
            .arg("--ignore")
            .arg("x/*")
            .arg("--ignore")
            .arg("testsuite/*")
            .arg("--ignore-not-existing")
            .arg("-o")
            .arg(args.html_cov_dir.unwrap());
        info!("Build grcov Html Coverage Report");
        info!("{:?}", grcov_html);
        grcov_html.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        grcov_html.output()?;
    }

    Ok(())
}
