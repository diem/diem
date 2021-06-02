// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cargo::{
        build_args::{BuildArgs, Coloring},
        selected_package::SelectedPackageArgs,
        CargoCommand,
    },
    context::XContext,
    Result,
};
use anyhow::bail;
use cargo_metadata::Message;
use guppy::PackageId;
use nextest_runner::{
    reporter::{Color, ReporterOpts, TestReporter},
    runner::TestRunnerOpts,
    test_filter::{RunIgnored, TestFilter},
    test_list::{TestBinary, TestList},
};
use std::ffi::OsString;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(flatten)]
    pub(crate) package_args: SelectedPackageArgs,
    #[structopt(long, short)]
    /// Skip running expensive diem testsuite integration tests
    unit: bool,
    #[structopt(flatten)]
    pub(crate) build_args: BuildArgs,
    #[structopt(flatten)]
    pub(crate) runner_opts: TestRunnerOpts,
    #[structopt(long)]
    /// Do not run tests, only compile the test executables
    no_run: bool,
    /// Run ignored tests
    #[structopt(long, possible_values = &RunIgnored::variants(), default_value, case_insensitive = true)]
    run_ignored: RunIgnored,
    #[structopt(name = "FILTERS", last = true)]
    filters: Vec<String>,
    #[structopt(flatten)]
    reporter_opts: ReporterOpts,
}

pub fn run(args: Args, xctx: XContext) -> Result<()> {
    let config = xctx.config();

    let mut packages = args.package_args.to_selected_packages(&xctx)?;
    if args.unit {
        packages.add_excludes(config.system_tests().iter().map(|(p, _)| p.as_str()));
    }

    let mut direct_args = Vec::new();
    args.build_args.add_args(&mut direct_args);

    // Always pass in --no-run as the test runner is responsible for running these tests.
    direct_args.push(OsString::from("--no-run"));

    // TODO: no-fail-fast (needs support in nextest)

    // Step 1: build all the test binaries with --no-run.
    let cmd = CargoCommand::Test {
        cargo_config: xctx.config().cargo_config(),
        direct_args: direct_args.as_slice(),
        // Don't pass in the args (test name) -- they're for use by the test runner.
        args: &[],
        env: &[],
        skip_sccache: false,
    };

    let messages = cmd.run_capture_messages(&packages)?;

    if args.no_run {
        // Don't proceed further.
        return Ok(());
    }

    let package_graph = xctx.core().package_graph()?;
    let workspace = package_graph.workspace();

    let mut executables = vec![];
    for message in messages {
        let message = message?;
        match message {
            Message::CompilerArtifact(artifact) if artifact.profile.test => {
                if let Some(binary) = artifact.executable {
                    // Look up the executable by package ID.
                    let package_id = PackageId::new(artifact.package_id.repr);

                    let package = package_graph.metadata(&package_id)?;
                    let cwd = Some(
                        workspace.root().join(
                            package
                                .source()
                                .workspace_path()
                                .expect("tests should never be built for non-workspace artifacts"),
                        ),
                    );

                    // Construct the binary ID from the package and build target.
                    let mut binary_id = package.name().to_owned();
                    if artifact.target.name != package.name() {
                        binary_id.push_str("::");
                        binary_id.push_str(&artifact.target.name);
                    }

                    executables.push(TestBinary {
                        binary,
                        binary_id,
                        cwd,
                    });
                }
            }
            _ => {
                // Ignore all other messages.
            }
        }
    }

    let test_filter = TestFilter::new(args.run_ignored, &args.filters);
    let test_list = TestList::new(executables, &test_filter)?;

    let runner = args.runner_opts.build(&test_list);

    let color = match args.build_args.color {
        Coloring::Auto => Color::Auto,
        Coloring::Always => Color::Always,
        Coloring::Never => Color::Never,
    };
    let mut reporter = TestReporter::new(&test_list, color, args.reporter_opts);

    let run_stats = runner.try_execute(|event| reporter.report_event(event))?;
    if !run_stats.is_success() {
        bail!("test run failed");
    }

    Ok(())
}
