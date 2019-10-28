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

    pub fn all_features(&mut self) -> &mut Self {
        self.inner.arg("--all-features");
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

pub enum CargoCommand {
    Test,
}

impl CargoCommand {
    pub fn run_on_local_package(
        &self,
        all_features: bool,
        pass_through_args: &[OsString],
    ) -> Result<()> {
        let mut cargo = Cargo::new(self.as_str());
        if all_features {
            cargo.all_features();
        }
        cargo.pass_through(pass_through_args).run()
    }

    pub fn run_on_packages_separate<I, S>(
        &self,
        packages: I,
        pass_through_args: &[OsString],
    ) -> Result<()>
    where
        I: IntoIterator<Item = (S, bool)>,
        S: AsRef<OsStr>,
    {
        for (name, all_features) in packages {
            let mut cargo = Cargo::new(self.as_str());
            if all_features {
                cargo.all_features();
            }
            cargo
                .packages(&[name])
                .current_dir(project_root())
                .pass_through(pass_through_args)
                .run()?;
        }
        Ok(())
    }

    pub fn run_on_packages_together<I, S>(
        &self,
        packages: I,
        pass_through_args: &[OsString],
    ) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Cargo::new(self.as_str())
            .current_dir(project_root())
            .all_features()
            .packages(packages)
            .pass_through(pass_through_args)
            .run()
    }

    pub fn run_with_exclusions<I, S>(
        &self,
        exclusions: I,
        pass_through_args: &[OsString],
    ) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Cargo::new(self.as_str())
            .current_dir(project_root())
            .all()
            .all_features()
            .exclusions(exclusions)
            .pass_through(pass_through_args)
            .run()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CargoCommand::Test => "test",
        }
    }
}
