// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::{
    cargo::{CargoArgs, CargoCommand, SelectedPackageArgs},
    context::XContext,
    Result,
};
use log::info;
use std::ffi::OsString;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(Debug)]
    enum Coloring {
        Auto,
        Always,
        Never,
    }
}

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(long, short)]
    /// No output printed to stdout
    quiet: bool,
    #[structopt(flatten)]
    package_args: SelectedPackageArgs,
    #[structopt(long, number_of_values = 1)]
    /// Package to exclude (see `cargo help pkgid`)
    exclude: Vec<String>,
    #[structopt(long, short)]
    /// Number of parallel jobs, defaults to # of CPUs
    jobs: Option<u16>,
    #[structopt(long)]
    /// Build only this package's library
    lib: bool,
    #[structopt(long, number_of_values = 1)]
    /// Build only the specified binary
    bin: Vec<String>,
    #[structopt(long)]
    /// Build all binaries
    bins: bool,
    #[structopt(long, number_of_values = 1)]
    /// Build only the specified example
    example: Vec<String>,
    #[structopt(long)]
    /// Build all examples
    examples: bool,
    #[structopt(long, number_of_values = 1)]
    /// Build only the specified test target
    test: Vec<String>,
    #[structopt(long)]
    /// Build all tests
    tests: bool,
    #[structopt(long, number_of_values = 1)]
    /// Build only the specified bench target
    bench: Vec<String>,
    #[structopt(long)]
    ///  Build all benches
    benchs: bool,
    #[structopt(long)]
    /// Build all targets
    all_targets: bool,
    #[structopt(long)]
    /// Build artifacts in release mode, with optimizations
    release: bool,
    #[structopt(long)]
    /// Build artifacts with the specified profile
    profile: Option<String>,
    #[structopt(long, number_of_values = 1)]
    /// Space-separated list of features to activate
    features: Vec<String>,
    #[structopt(long)]
    /// Activate all available features
    all_features: bool,
    #[structopt(long)]
    /// Do not activate the `default` feature
    no_default_features: bool,
    #[structopt(long)]
    /// TRIPLE
    target: Option<String>,
    #[structopt(long, parse(from_os_str))]
    /// Directory for all generated artifacts
    target_dir: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    /// Copy final artifacts to this directory (unstable)
    out_dir: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    /// Path to Cargo.toml
    manifest_path: Option<OsString>,
    #[structopt(long)]
    /// Error format
    message_format: Option<String>,
    #[structopt(long)]
    /// Output the build plan in JSON (unstable)
    build_plan: bool,
    //TODO: support -vv?
    #[structopt(long, short, parse(from_occurrences))]
    ///  Use verbose output (-vv very verbose/build.rs output)
    verbose: usize,
    #[structopt(long, possible_values = &Coloring::variants(), default_value="Auto")]
    ///  Coloring: auto, always, never
    coloring: Coloring,
    #[structopt(long)]
    ///  Require Cargo.lock and cache are up to date
    frozen: bool,
    #[structopt(long)]
    ///  Require Cargo.lock is up to date
    locked: bool,
    #[structopt(long)]
    ///  Run without accessing the network
    offline: bool,
}

pub fn convert_args(args: &Args) -> Vec<OsString> {
    let mut direct_args = Vec::new();
    if args.quiet {
        direct_args.push(OsString::from("--quiet"));
    }
    if let Some(jobs) = args.jobs {
        direct_args.push(OsString::from("--jobs"));
        direct_args.push(OsString::from(jobs.to_string()));
    };
    if args.lib {
        direct_args.push(OsString::from("--lib"));
    };
    if !args.bin.is_empty() {
        direct_args.push(OsString::from("--bin"));
        for bin in &args.bin {
            direct_args.push(OsString::from(bin));
        }
    }
    if args.bins {
        direct_args.push(OsString::from("--bins"));
    };
    if !args.example.is_empty() {
        direct_args.push(OsString::from("--example"));
        for example in &args.example {
            direct_args.push(OsString::from(example));
        }
    }
    if args.examples {
        direct_args.push(OsString::from("--examples"));
    };

    if !args.test.is_empty() {
        direct_args.push(OsString::from("--test"));
        for test in &args.test {
            direct_args.push(OsString::from(test));
        }
    }
    if args.tests {
        direct_args.push(OsString::from("--tests"));
    };

    if !args.bench.is_empty() {
        direct_args.push(OsString::from("--bench"));
        for bench in &args.bench {
            direct_args.push(OsString::from(bench));
        }
    }
    if args.benchs {
        direct_args.push(OsString::from("--benchs"));
    };

    if args.all_targets {
        direct_args.push(OsString::from("--all-targets"));
    };
    if args.release {
        direct_args.push(OsString::from("--release"));
    };

    if let Some(profile) = &args.profile {
        direct_args.push(OsString::from("--profile"));
        direct_args.push(OsString::from(profile.to_string()));
    };

    if !args.features.is_empty() {
        direct_args.push(OsString::from("--features"));
        for features in &args.features {
            direct_args.push(OsString::from(features));
        }
    }
    if args.all_features {
        direct_args.push(OsString::from("--all-features"));
    };
    if args.no_default_features {
        direct_args.push(OsString::from("--no-default-features"));
    };

    if let Some(target) = &args.target {
        direct_args.push(OsString::from("--target"));
        direct_args.push(OsString::from(target.to_string()));
    };
    if let Some(target_dir) = &args.target_dir {
        direct_args.push(OsString::from("--target-dir"));
        direct_args.push(OsString::from(target_dir));
    };
    if let Some(out_dir) = &args.out_dir {
        direct_args.push(OsString::from("--out-dir"));
        direct_args.push(OsString::from(out_dir));
    };
    if let Some(manifest_path) = &args.manifest_path {
        direct_args.push(OsString::from("--manifest-path"));
        direct_args.push(manifest_path.to_owned());
    };
    if let Some(message_format) = &args.message_format {
        direct_args.push(OsString::from("--message-format"));
        direct_args.push(OsString::from(message_format.to_string()));
    };
    if args.build_plan {
        direct_args.push(OsString::from("--build-plan"));
    };
    if args.verbose > 0 {
        direct_args.push(OsString::from(format!("-{}", "v".repeat(args.verbose))));
    };
    if args.coloring.to_string() != Coloring::Auto.to_string() {
        direct_args.push(OsString::from("--coloring"));
        direct_args.push(OsString::from(args.coloring.to_string()));
    };
    if args.frozen {
        direct_args.push(OsString::from("--frozen"));
    };
    if args.locked {
        direct_args.push(OsString::from("--locked"));
    };
    if args.offline {
        direct_args.push(OsString::from("--offline"));
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
    };

    let packages = args.package_args.to_selected_packages(&xctx)?;
    cmd.run_on_packages(&packages, &CargoArgs::default())
}
