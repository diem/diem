// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::ffi::OsString;
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    #[derive(Debug)]
    pub enum Coloring {
        Auto,
        Always,
        Never,
    }
}

/// Arguments for controlling cargo build and other similar commands (like check).
#[derive(Debug, StructOpt)]
pub struct BuildArgs {
    #[structopt(long, short)]
    /// No output printed to stdout
    pub(crate) quiet: bool,
    #[structopt(long, short)]
    /// Number of parallel build jobs, defaults to # of CPUs
    pub(crate) jobs: Option<u16>,
    #[structopt(long)]
    /// Only this package's library
    pub(crate) lib: bool,
    #[structopt(long, number_of_values = 1)]
    /// Only the specified binary
    pub(crate) bin: Vec<String>,
    #[structopt(long)]
    /// All binaries
    pub(crate) bins: bool,
    #[structopt(long, number_of_values = 1)]
    /// Only the specified example
    pub(crate) example: Vec<String>,
    #[structopt(long)]
    /// All examples
    pub(crate) examples: bool,
    #[structopt(long, number_of_values = 1)]
    /// Only the specified test target
    pub(crate) test: Vec<String>,
    #[structopt(long)]
    /// All tests
    pub(crate) tests: bool,
    #[structopt(long, number_of_values = 1)]
    /// Only the specified bench target
    pub(crate) bench: Vec<String>,
    #[structopt(long)]
    /// All benches
    pub(crate) benches: bool,
    #[structopt(long)]
    /// All targets
    pub(crate) all_targets: bool,
    #[structopt(long)]
    /// Artifacts in release mode, with optimizations
    pub(crate) release: bool,
    #[structopt(long)]
    /// Artifacts with the specified profile
    pub(crate) profile: Option<String>,
    #[structopt(long, number_of_values = 1)]
    /// Space-separated list of features to activate
    pub(crate) features: Vec<String>,
    #[structopt(long)]
    /// Activate all available features
    pub(crate) all_features: bool,
    #[structopt(long)]
    /// Do not activate the `default` feature
    pub(crate) no_default_features: bool,
    #[structopt(long)]
    /// TRIPLE
    pub(crate) target: Option<String>,
    #[structopt(long, parse(from_os_str))]
    /// Directory for all generated artifacts
    pub(crate) target_dir: Option<OsString>,
    #[structopt(long, parse(from_os_str))]
    /// Path to Cargo.toml
    pub(crate) manifest_path: Option<OsString>,
    #[structopt(long)]
    /// Error format
    pub(crate) message_format: Option<String>,
    #[structopt(long, short, parse(from_occurrences))]
    /// Use verbose output (-vv very verbose/build.rs output)
    pub(crate) verbose: usize,
    #[structopt(long, possible_values = &Coloring::variants(), default_value="Auto")]
    /// Coloring: auto, always, never
    pub(crate) color: Coloring,
    #[structopt(long)]
    /// Require Cargo.lock and cache are up to date
    pub(crate) frozen: bool,
    #[structopt(long)]
    /// Require Cargo.lock is up to date
    pub(crate) locked: bool,
    #[structopt(long)]
    /// Run without accessing the network
    pub(crate) offline: bool,
}

impl BuildArgs {
    pub fn add_args(&self, direct_args: &mut Vec<OsString>) {
        if self.quiet {
            direct_args.push(OsString::from("--quiet"));
        }
        if let Some(jobs) = self.jobs {
            direct_args.push(OsString::from("--jobs"));
            direct_args.push(OsString::from(jobs.to_string()));
        };
        if self.lib {
            direct_args.push(OsString::from("--lib"));
        };
        if !self.bin.is_empty() {
            direct_args.push(OsString::from("--bin"));
            for bin in &self.bin {
                direct_args.push(OsString::from(bin));
            }
        }
        if self.bins {
            direct_args.push(OsString::from("--bins"));
        };
        if !self.example.is_empty() {
            direct_args.push(OsString::from("--example"));
            for example in &self.example {
                direct_args.push(OsString::from(example));
            }
        }
        if self.examples {
            direct_args.push(OsString::from("--examples"));
        };

        if !self.test.is_empty() {
            direct_args.push(OsString::from("--test"));
            for test in &self.test {
                direct_args.push(OsString::from(test));
            }
        }
        if self.tests {
            direct_args.push(OsString::from("--tests"));
        };

        if !self.bench.is_empty() {
            direct_args.push(OsString::from("--bench"));
            for bench in &self.bench {
                direct_args.push(OsString::from(bench));
            }
        }
        if self.benches {
            direct_args.push(OsString::from("--benches"));
        };

        if self.all_targets {
            direct_args.push(OsString::from("--all-targets"));
        };
        if self.release {
            direct_args.push(OsString::from("--release"));
        };

        if let Some(profile) = &self.profile {
            direct_args.push(OsString::from("--profile"));
            direct_args.push(OsString::from(profile.to_string()));
        };

        if !self.features.is_empty() {
            direct_args.push(OsString::from("--features"));
            for features in &self.features {
                direct_args.push(OsString::from(features));
            }
        }
        if self.all_features {
            direct_args.push(OsString::from("--all-features"));
        };
        if self.no_default_features {
            direct_args.push(OsString::from("--no-default-features"));
        };

        if let Some(target) = &self.target {
            direct_args.push(OsString::from("--target"));
            direct_args.push(OsString::from(target.to_string()));
        };
        if let Some(target_dir) = &self.target_dir {
            direct_args.push(OsString::from("--target-dir"));
            direct_args.push(OsString::from(target_dir));
        };
        if let Some(manifest_path) = &self.manifest_path {
            direct_args.push(OsString::from("--manifest-path"));
            direct_args.push(manifest_path.to_owned());
        };
        if let Some(message_format) = &self.message_format {
            direct_args.push(OsString::from("--message-format"));
            direct_args.push(OsString::from(message_format.to_string()));
        };
        if self.verbose > 0 {
            direct_args.push(OsString::from(format!("-{}", "v".repeat(self.verbose))));
        };
        if self.color.to_string() != Coloring::Auto.to_string() {
            direct_args.push(OsString::from("--color"));
            direct_args.push(OsString::from(self.color.to_string()));
        };
        if self.frozen {
            direct_args.push(OsString::from("--frozen"));
        };
        if self.locked {
            direct_args.push(OsString::from("--locked"));
        };
        if self.offline {
            direct_args.push(OsString::from("--offline"));
        };
    }
}
