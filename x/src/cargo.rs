// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::project_root, Result};
use anyhow::anyhow;
use log::{info, warn};
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::Path,
    process::{Command, Output, Stdio},
    time::Instant,
};

pub struct Cargo {
    inner: Command,
    direct_args: Vec<OsString>,
    pass_through_args: Vec<OsString>,
    env_additions: HashMap<OsString, OsString>,
}

impl Cargo {
    pub fn new<S: AsRef<OsStr>>(command: S) -> Self {
        let mut inner = Command::new("cargo");
        inner.arg(command);
        Self {
            inner,
            direct_args: Vec::new(),
            pass_through_args: Vec::new(),
            env_additions: HashMap::new(),
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

    /// Not all arguments are shared between commands, and x doesn't care about all argument types.
    /// Various commands can add extra args to the base cargo command via this function.
    pub fn direct_arguments<I, S>(&mut self, direct_args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for direct_arg in direct_args {
            self.inner.arg(direct_arg);
        }
        self
    }

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
    pub fn env_additions<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.env_additions
                .insert(a.0.as_ref().to_owned(), a.1.as_ref().to_owned());
        }
        self
    }

    pub fn run(&mut self) -> Result<()> {
        self.inner.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        self.do_run(true).map(|_| ())
    }

    pub fn run_with_output(&mut self) -> Result<Vec<u8>> {
        self.inner.stderr(Stdio::inherit());
        // Since system out hijacked don't log for this command
        self.do_run(false).map(|o| o.stdout)
    }

    fn do_run(&mut self, log: bool) -> Result<Output> {
        //these are arguments on the cargo command
        if !self.direct_args.is_empty() {
            self.inner.args(&self.direct_args);
        }

        //this are arguments are passed through cargo to underlaying
        //executable
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
        self.inner.envs(&self.env_additions);
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
            .direct_arguments(self.direct_args())
            .pass_through(self.pass_through_args())
            .env_additions(self.get_extra_env().as_ref().to_owned())
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
                .direct_arguments(self.direct_args())
                .packages(&[name])
                .current_dir(project_root())
                .pass_through(self.pass_through_args())
                .env_additions(self.get_extra_env().as_ref().to_owned())
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
            .direct_arguments(self.direct_args())
            .packages(packages)
            .pass_through(self.pass_through_args())
            .env_additions(self.get_extra_env().as_ref().to_owned())
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
            .direct_arguments(self.direct_args())
            .exclusions(exclusions)
            .pass_through(self.pass_through_args())
            .env_additions(self.get_extra_env().as_ref().to_owned())
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
