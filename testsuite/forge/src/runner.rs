// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use rand::{Rng, SeedableRng};
use std::{
    io::{self, Write},
    num::NonZeroUsize,
    process,
};
use structopt::{clap::arg_enum, StructOpt};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
// TODO going to remove random seed once cluster deployment supports re-run genesis
use rand::rngs::OsRng;

#[derive(Debug, StructOpt)]
#[structopt(about = "Forged in Fire")]
pub struct Options {
    /// The FILTER string is tested against the name of all tests, and only those tests whose names
    /// contain the filter are run.
    filter: Option<String>,
    #[structopt(long = "exact")]
    /// Exactly match filters rather than by substring
    filter_exact: bool,
    #[structopt(long, default_value = "1", env = "RUST_TEST_THREADS")]
    /// NO-OP: unsupported option, exists for compatibility with the default test harness
    /// Number of threads used for running tests in parallel
    test_threads: NonZeroUsize,
    #[structopt(short = "q", long)]
    /// Output minimal information
    quiet: bool,
    #[structopt(long)]
    /// NO-OP: unsupported option, exists for compatibility with the default test harness
    nocapture: bool,
    #[structopt(long)]
    /// List all tests
    list: bool,
    #[structopt(long)]
    /// List or run ignored tests
    ignored: bool,
    #[structopt(long)]
    /// Include ignored tests when listing or running tests
    include_ignored: bool,
    /// Configure formatting of output:
    ///   pretty = Print verbose output;
    ///   terse = Display one character per test;
    ///   (json is unsupported, exists for compatibility with the default test harness)
    #[structopt(long, possible_values = &Format::variants(), default_value, case_insensitive = true)]
    format: Format,
}

impl Options {
    pub fn from_args() -> Self {
        StructOpt::from_args()
    }
}

arg_enum! {
    #[derive(Debug, Eq, PartialEq)]
    pub enum Format {
        Pretty,
        Terse,
        Json,
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::Pretty
    }
}

pub fn forge_main<F: Factory>(tests: ForgeConfig<'_>, factory: F, options: &Options) -> Result<()> {
    let forge = Forge::new(options, tests, factory);

    if options.list {
        forge.list()?;

        return Ok(());
    }

    match forge.run() {
        Ok(()) => Ok(()),
        Err(e) => {
            eprintln!("Failed to run tests:\n{}", e);
            process::exit(101); // Exit with a non-zero exit code if tests failed
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitialVersion {
    Oldest,
    Newest,
}

pub struct ForgeConfig<'cfg> {
    public_usage_tests: &'cfg [&'cfg dyn PublicUsageTest],
    admin_tests: &'cfg [&'cfg dyn AdminTest],
    network_tests: &'cfg [&'cfg dyn NetworkTest],

    /// The initial number of validators to spawn when the test harness creates a swarm
    initial_validator_count: NonZeroUsize,

    /// The initial version to use when the test harness creates a swarm
    initial_version: InitialVersion,
}

impl<'cfg> ForgeConfig<'cfg> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_public_usage_tests(
        mut self,
        public_usage_tests: &'cfg [&'cfg dyn PublicUsageTest],
    ) -> Self {
        self.public_usage_tests = public_usage_tests;
        self
    }

    pub fn with_admin_tests(mut self, admin_tests: &'cfg [&'cfg dyn AdminTest]) -> Self {
        self.admin_tests = admin_tests;
        self
    }

    pub fn with_network_tests(mut self, network_tests: &'cfg [&'cfg dyn NetworkTest]) -> Self {
        self.network_tests = network_tests;
        self
    }

    pub fn with_initial_validator_count(mut self, initial_validator_count: NonZeroUsize) -> Self {
        self.initial_validator_count = initial_validator_count;
        self
    }

    pub fn with_initial_version(mut self, initial_version: InitialVersion) -> Self {
        self.initial_version = initial_version;
        self
    }

    pub fn number_of_tests(&self) -> usize {
        self.public_usage_tests.len() + self.admin_tests.len() + self.network_tests.len()
    }

    pub fn all_tests(&self) -> impl Iterator<Item = &'cfg dyn Test> + 'cfg {
        self.public_usage_tests
            .iter()
            .map(|t| t as &dyn Test)
            .chain(self.admin_tests.iter().map(|t| t as &dyn Test))
            .chain(self.network_tests.iter().map(|t| t as &dyn Test))
    }
}

impl<'cfg> Default for ForgeConfig<'cfg> {
    fn default() -> Self {
        Self {
            public_usage_tests: &[],
            admin_tests: &[],
            network_tests: &[],
            initial_validator_count: NonZeroUsize::new(1).unwrap(),
            initial_version: InitialVersion::Newest,
        }
    }
}

pub struct Forge<'cfg, F> {
    options: &'cfg Options,
    tests: ForgeConfig<'cfg>,
    factory: F,
}

impl<'cfg, F: Factory> Forge<'cfg, F> {
    pub fn new(options: &'cfg Options, tests: ForgeConfig<'cfg>, factory: F) -> Self {
        Self {
            options,
            tests,
            factory,
        }
    }

    pub fn list(&self) -> Result<()> {
        for test in self.filter_tests(self.tests.all_tests()) {
            println!("{}: test", test.name());
        }

        if self.options.format == Format::Pretty {
            println!();
            println!(
                "{} tests",
                self.filter_tests(self.tests.all_tests()).count()
            );
        }

        Ok(())
    }

    pub fn initial_version(&self) -> Version {
        let versions = self.factory.versions();
        match self.tests.initial_version {
            InitialVersion::Oldest => versions.min(),
            InitialVersion::Newest => versions.max(),
        }
        .expect("There has to be at least 1 version")
    }

    pub fn run(&self) -> Result<()> {
        let test_count = self.filter_tests(self.tests.all_tests()).count();
        let filtered_out = test_count.saturating_sub(self.tests.all_tests().count());

        let mut summary = TestSummary::new(test_count, filtered_out);
        summary.write_starting_msg()?;

        if test_count > 0 {
            let initial_version = self.initial_version();
            let mut rng = ::rand::rngs::StdRng::from_seed(OsRng.gen());
            let mut swarm = self.factory.launch_swarm(
                &mut rng,
                self.tests.initial_validator_count,
                &initial_version,
            )?;

            // Run PublicUsageTests
            for test in self.filter_tests(self.tests.public_usage_tests.iter()) {
                let mut public_ctx = PublicUsageContext::new(
                    CoreContext::from_rng(&mut rng),
                    swarm.chain_info().into_public_info(),
                );
                let result = run_test(|| test.run(&mut public_ctx));
                summary.handle_result(test.name().to_owned(), result)?;
            }

            // Run AdminTests
            for test in self.filter_tests(self.tests.admin_tests.iter()) {
                let mut admin_ctx =
                    AdminContext::new(CoreContext::from_rng(&mut rng), swarm.chain_info());
                let result = run_test(|| test.run(&mut admin_ctx));
                summary.handle_result(test.name().to_owned(), result)?;
            }

            for test in self.filter_tests(self.tests.network_tests.iter()) {
                let report = TestReport::new();
                let mut network_ctx =
                    NetworkContext::new(CoreContext::from_rng(&mut rng), &mut *swarm, report);
                let result = run_test(|| test.run(&mut network_ctx));
                summary.handle_result(test.name().to_owned(), result)?;
            }

            io::stdout().flush()?;
            io::stderr().flush()?;

            if !summary.success() {
                println!();
                println!("Swarm logs can be found here: {}", swarm.logs_location());
            }
        }

        summary.write_summary()?;

        if summary.success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tests Failed"))
        }
    }

    fn filter_tests<'a, T: Test, I: Iterator<Item = T> + 'a>(
        &'a self,
        tests: I,
    ) -> impl Iterator<Item = T> + 'a {
        tests
            // Filter by ignored
            .filter(
                move |test| match (self.options.include_ignored, self.options.ignored) {
                    (true, _) => true, // Don't filter anything
                    (false, true) => test.ignored(),
                    (false, false) => !test.ignored(),
                },
            )
            // Filter by test name
            .filter(move |test| {
                if let Some(filter) = &self.options.filter {
                    if self.options.filter_exact {
                        test.name() == &filter[..]
                    } else {
                        test.name().contains(&filter[..])
                    }
                } else {
                    true
                }
            })
    }
}

enum TestResult {
    Ok,
    Failed,
    FailedWithMsg(String),
}

fn run_test<F: FnOnce() -> Result<()>>(f: F) -> TestResult {
    match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(f)) {
        Ok(Ok(())) => TestResult::Ok,
        Ok(Err(e)) => TestResult::FailedWithMsg(format!("{:?}", e)),
        Err(_) => TestResult::Failed,
    }
}

struct TestSummary {
    stdout: StandardStream,
    total: usize,
    filtered_out: usize,
    passed: usize,
    failed: Vec<String>,
}

impl TestSummary {
    fn new(total: usize, filtered_out: usize) -> Self {
        Self {
            stdout: StandardStream::stdout(ColorChoice::Auto),
            total,
            filtered_out,
            passed: 0,
            failed: Vec::new(),
        }
    }

    fn handle_result(&mut self, name: String, result: TestResult) -> io::Result<()> {
        write!(self.stdout, "test {} ... ", name)?;
        match result {
            TestResult::Ok => {
                self.passed += 1;
                self.write_ok()?;
            }
            TestResult::Failed => {
                self.failed.push(name);
                self.write_failed()?;
            }
            TestResult::FailedWithMsg(msg) => {
                self.failed.push(name);
                self.write_failed()?;
                writeln!(self.stdout)?;

                write!(self.stdout, "Error: {}", msg)?;
            }
        }
        writeln!(self.stdout)?;
        Ok(())
    }

    fn write_ok(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(self.stdout, "ok")?;
        self.stdout.reset()?;
        Ok(())
    }

    fn write_failed(&mut self) -> io::Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        write!(self.stdout, "FAILED")?;
        self.stdout.reset()?;
        Ok(())
    }

    fn write_starting_msg(&mut self) -> io::Result<()> {
        writeln!(self.stdout)?;
        writeln!(
            self.stdout,
            "running {} tests",
            self.total - self.filtered_out
        )?;
        Ok(())
    }

    fn write_summary(&mut self) -> io::Result<()> {
        // Print out the failing tests
        if !self.failed.is_empty() {
            writeln!(self.stdout)?;
            writeln!(self.stdout, "failures:")?;
            for name in &self.failed {
                writeln!(self.stdout, "    {}", name)?;
            }
        }

        writeln!(self.stdout)?;
        write!(self.stdout, "test result: ")?;
        if self.failed.is_empty() {
            self.write_ok()?;
        } else {
            self.write_failed()?;
        }
        writeln!(
            self.stdout,
            ". {} passed; {} failed; {} filtered out",
            self.passed,
            self.failed.len(),
            self.filtered_out
        )?;
        writeln!(self.stdout)?;
        Ok(())
    }

    fn success(&self) -> bool {
        self.failed.is_empty()
    }
}
