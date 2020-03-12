// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils::project_root, Result};
use anyhow::anyhow;
use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::{Command, Output, Stdio},
};

pub struct Cargo {
    inner: Command,
    pass_through_args: Vec<OsString>,
}

impl Cargo {
    pub fn new<S: AsRef<OsStr>>(command: S) -> Self {
        let mut inner = Command::new("cargo");
        inner.arg(command);
        Self {
            inner,
            pass_through_args: Vec::new(),
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

    pub fn run(&mut self) -> Result<()> {
        self.inner.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        self.do_run().map(|_| ())
    }

    pub fn run_with_output(&mut self) -> Result<Vec<u8>> {
        self.inner.stderr(Stdio::inherit());
        self.do_run().map(|o| o.stdout)
    }

    fn do_run(&mut self) -> Result<Output> {
        if !self.pass_through_args.is_empty() {
            self.inner.arg("--").args(&self.pass_through_args);
        }
        let output = self.inner.output()?;
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
    Test(&'a [OsString]),
}

impl<'a> CargoCommand<'a> {
    pub fn run_on_local_package(&self, args: &CargoArgs) -> Result<()> {
        let mut cargo = Cargo::new(self.as_str());
        Self::apply_args(&mut cargo, args);
        cargo.pass_through(self.pass_through_args()).run()
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
                .packages(&[name])
                .current_dir(project_root())
                .pass_through(self.pass_through_args())
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
            .packages(packages)
            .pass_through(self.pass_through_args())
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
            .exclusions(exclusions)
            .pass_through(self.pass_through_args())
            .run()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CargoCommand::Bench(_) => "bench",
            CargoCommand::Check => "check",
            CargoCommand::Clippy(_) => "clippy",
            CargoCommand::Test(_) => "test",
        }
    }

    fn pass_through_args(&self) -> &[OsString] {
        match self {
            CargoCommand::Bench(args) => args,
            CargoCommand::Check => &[],
            CargoCommand::Clippy(args) => args,
            CargoCommand::Test(args) => args,
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
