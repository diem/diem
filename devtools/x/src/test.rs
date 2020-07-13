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
    /// Skip running expensive libra testsuite integration tests
    unit: bool,
    #[structopt(long)]
    /// Test only this package's library unit tests, skipping doctests
    lib: bool,
    #[structopt(long)]
    /// Do not fast fail the run if tests (or test executables) fail
    no_fail_fast: bool,
    #[structopt(long)]
    /// Do not run tests, only compile the test executables
    no_run: bool,
    #[structopt(long, short)]
    /// Number of parallel jobs, defaults to # of CPUs
    jobs: Option<u16>,
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
                "-Zprofile -Ccodegen-units=1 -Coverflow-checks=off",
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
    if args.lib {
        direct_args.push(OsString::from("--lib"));
    };

    if let Some(jobs) = args.jobs {
        direct_args.push(OsString::from("--jobs"));
        direct_args.push(OsString::from(jobs.to_string()));
    };

    let cmd = CargoCommand::Test {
        cargo_config: xctx.config().cargo_config(),
        direct_args: direct_args.as_slice(),
        args: &args.args,
        env: &env_vars,
    };

    if args.unit {
        cmd.run_with_exclusions(
            config.system_tests().iter().map(|(p, _)| p),
            &CargoArgs::default(),
        )?;
    } else if !args.package.is_empty() {
        cmd.run_on_packages(args.package.iter(), &CargoArgs::default())?;
    } else if utils::project_is_root(&xctx)? {
        // TODO Maybe only run a subest of tests if we're not inside
        // a package but not at the project root (e.g. language)
        cmd.run_on_all_packages(&CargoArgs::default())?;
    } else {
        cmd.run_on_local_package(&CargoArgs::default())?;
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
