// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{xcontext::execution_location::project_root, Result};
use anyhow::anyhow;
use log::{info, warn};
use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::{Command, Output, Stdio},
    time::Instant,
};

pub struct Cargo {
    inner: Command,
    pass_through_args: Vec<OsString>,
    env_additions: Vec<(OsString, OsString)>,
}

impl Cargo {
    pub fn new<S: AsRef<OsStr>>(command: S) -> Self {
        let mut inner = Command::new("cargo");
        inner.arg(command);
        Self {
            inner,
            pass_through_args: Vec::new(),
            env_additions: Vec::new(),
        }
    }

    pub fn all(&mut self) -> &mut Self {
        self.inner.arg("--all");
        self
    }

    pub fn workspace(&mut self) -> &mut Self {
        self.inner.arg("--workspace");
        self
    }

    pub fn all_features(&mut self) -> &mut Self {
        self.inner.arg("--all-features");
        self
    }

    pub fn all_targets(&mut self) -> &mut Self {
        self.inner.arg("--all-targets");
        self
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    pub fn packages<I, S>(&mut self, packages: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for p in packages {
            self.inner.arg("--package");
            self.inner.arg(p);
        }
        self
    }

    pub fn exclusions<I, S>(&mut self, exclusions: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for e in exclusions {
            self.inner.arg("--exclude");
            self.inner.arg(e);
        }
        self
    }

    /// Adds a series of arguments to x's target command.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Adds an argument to x's target command.
    #[allow(dead_code)]
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// Adds "Pass Through" arguments to x's target command.
    /// Pass through arguments appear after a double dash " -- " and may
    /// not be handled/checked by x's target command itself, but an underlying executable.
    pub fn pass_through<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.pass_through_args.push(a.as_ref().to_owned());
        }
        self
    }

    /// Passes extra environment variables to x's target command.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (key, val) in vars {
            self.env(key, val);
        }
        self
    }

    /// Passes an extra environment variable to x's target command.
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env_additions
            .push((key.as_ref().to_os_string(), val.as_ref().to_os_string()));
        self
    }

    pub fn run(&mut self) -> Result<()> {
        self.inner.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        self.do_run(true).map(|_| ())
    }

    /// Runs this command, capturing the standard output into a `Vec<u8>`.
    /// No logging/timing will be displayed as the result of this call from x.
    pub fn run_with_output(&mut self) -> Result<Vec<u8>> {
        self.inner.stderr(Stdio::inherit());
        // Since system out hijacked don't log for this command
        self.do_run(false).map(|o| o.stdout)
    }

    /// Internal run command, where the magic happens.
    /// If log is true, any environment variable overrides will be logged, the full command will be logged,
    /// and after the command's output reaches stdout, the command will be printed again along with the time took
    /// to process the command (wallclock) in ms.
    fn do_run(&mut self, log: bool) -> Result<Output> {
        // these arguments are passed through cargo/x to underlying executable (test, clippy, etc)
        if !self.pass_through_args.is_empty() {
            self.inner.arg("--").args(&self.pass_through_args);
        }

        // once all the arguments are added to the command we can log it.
        if log {
            self.env_additions
                .iter()
                .for_each(|t| info!("Env {:?}: {:?}", t.0, t.1));
            info!("Executing: {:?}", &self.inner);
        }
        self.inner
            .envs(self.env_additions.iter().map(|(k, v)| (k, v)));
        let now = Instant::now();
        let output = self.inner.output()?;
        // once the command has been executed we log it's success or failure.
        if log {
            if output.status.success() {
                info!(
                    "Completed in {}ms: {:?}",
                    now.elapsed().as_millis(),
                    &self.inner
                );
            } else {
                warn!(
                    "Failed in {}ms: {:?}",
                    now.elapsed().as_millis(),
                    &self.inner
                );
            }
        }
        if !output.status.success() {
            return Err(anyhow!("failed to run cargo command"));
        }
        Ok(output)
    }
}

#[derive(Debug, Default)]
pub struct CargoArgs {
    pub all_features: bool,
    pub all_targets: bool,
}

/// Represents an invocations of cargo that will call multiple other invocations of
/// cargo based on groupings implied by the contents of <workspace-root>/x.toml.
pub enum CargoCommand<'a> {
    Bench(&'a [OsString]),
    Check,
    Clippy(&'a [OsString]),
    Test {
        direct_args: &'a [OsString],
        args: &'a [OsString],
        env: &'a [(&'a str, &'a str)],
    },
}

impl<'a> CargoCommand<'a> {
    pub fn run_on_local_package(&self, args: &CargoArgs) -> Result<()> {
        let mut cargo = Cargo::new(self.as_str());
        Self::apply_args(&mut cargo, args);
        cargo
            .args(self.direct_args())
            .pass_through(self.pass_through_args())
            .envs(self.get_extra_env().to_owned())
            .run()
    }

    pub fn run_on_packages_separate<I, S>(&self, packages: I) -> Result<()>
    where
        I: IntoIterator<Item = (S, CargoArgs)>,
        S: AsRef<OsStr>,
    {
        for (name, args) in packages {
            let mut cargo = Cargo::new(self.as_str());
            Self::apply_args(&mut cargo, &args);
            cargo
                .args(self.direct_args())
                .packages(&[name])
                .current_dir(project_root())
                .pass_through(self.pass_through_args())
                .envs(self.get_extra_env().to_owned())
                .run()?;
        }
        Ok(())
    }

    pub fn run_on_packages_together<I, S>(&self, packages: I, args: &CargoArgs) -> Result<()>
    where
        I: IntoIterator<Item = S> + Clone,
        S: AsRef<OsStr>,
    {
        // Early return if we have no packages to run
        if packages.clone().into_iter().count() == 0 {
            return Ok(());
        }

        let mut cargo = Cargo::new(self.as_str());
        Self::apply_args(&mut cargo, args);
        cargo
            .current_dir(project_root())
            .args(self.direct_args())
            .packages(packages)
            .pass_through(self.pass_through_args())
            .envs(self.get_extra_env().to_owned())
            .run()
    }

    pub fn run_with_exclusions<I, S>(&self, exclusions: I, args: &CargoArgs) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cargo = Cargo::new(self.as_str());
        Self::apply_args(&mut cargo, args);
        cargo
            .current_dir(project_root())
            .workspace()
            .args(self.direct_args())
            .exclusions(exclusions)
            .pass_through(self.pass_through_args())
            .envs(self.get_extra_env().to_owned())
            .run()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CargoCommand::Bench(_) => "bench",
            CargoCommand::Check => "check",
            CargoCommand::Clippy(_) => "clippy",
            CargoCommand::Test { .. } => "test",
        }
    }

    fn pass_through_args(&self) -> &[OsString] {
        match self {
            CargoCommand::Bench(args) => args,
            CargoCommand::Check => &[],
            CargoCommand::Clippy(args) => args,
            CargoCommand::Test { args, .. } => args,
        }
    }

    fn direct_args(&self) -> &[OsString] {
        match self {
            CargoCommand::Bench(_) => &[],
            CargoCommand::Check => &[],
            CargoCommand::Clippy(_) => &[],
            CargoCommand::Test { direct_args, .. } => direct_args,
        }
    }

    pub fn get_extra_env(&self) -> &[(&str, &str)] {
        match self {
            CargoCommand::Bench(_) => &[],
            CargoCommand::Check => &[],
            CargoCommand::Clippy(_) => &[],
            CargoCommand::Test { env, .. } => &env,
        }
    }

    fn apply_args(cargo: &mut Cargo, args: &CargoArgs) {
        if args.all_features {
            cargo.all_features();
        }
        if args.all_targets {
            cargo.all_targets();
        }
    }
}
