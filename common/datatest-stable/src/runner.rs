// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{utils, Result};
use std::{
    io::{self, Write},
    num::NonZeroUsize,
    panic::{catch_unwind, AssertUnwindSafe},
    path::Path,
    process,
    sync::mpsc::{channel, Sender},
    thread,
};
use structopt::StructOpt;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, StructOpt)]
#[structopt(about = "Datatest-harness for running data-driven tests")]
struct TestOpts {
    /// The FILTER string is tested against the name of all tests, and only those tests whose names
    /// contain the filter are run.
    filter: Option<String>,
    #[structopt(long = "exact")]
    /// Exactly match filters rather than by substring
    filter_exact: bool,
    #[structopt(long, default_value = "32", env = "RUST_TEST_THREADS")]
    /// Number of threads used for running tests in parallel
    test_threads: NonZeroUsize,
    #[structopt(short = "q", long)]
    /// Output minimal information
    quiet: bool,
    #[structopt(long)]
    /// We already can't capture anything but we don't want arg parsing to fail so this is a noop
    nocapture: bool,
}

pub fn runner(reqs: &[Requirements]) {
    let options = TestOpts::from_args();

    let tests = reqs.iter().flat_map(|req| req.expand()).collect();

    match run_tests(options, tests) {
        Ok(true) => {}
        Ok(false) => process::exit(101),
        Err(e) => {
            eprintln!("error: io error when running tests: {:?}", e);
            process::exit(101);
        }
    }
}

struct Test {
    testfn: Box<dyn Fn() -> Result<()> + Send>,
    name: String,
}

enum TestResult {
    Ok,
    Failed,
    FailedWithMsg(String),
}

fn run_tests(options: TestOpts, tests: Vec<Test>) -> io::Result<bool> {
    let total = tests.len();

    // Filter out tests
    let mut remaining = match &options.filter {
        None => tests,
        Some(filter) => tests
            .into_iter()
            .filter(|test| {
                if options.filter_exact {
                    test.name == filter[..]
                } else {
                    test.name.contains(&filter[..])
                }
            })
            .rev()
            .collect(),
    };

    let filtered_out = total - remaining.len();
    let mut summary = TestSummary::new(total, filtered_out);

    if !options.quiet {
        summary.write_starting_msg()?;
    }

    let (tx, rx) = channel();

    let mut pending = 0;
    while pending > 0 || !remaining.is_empty() {
        while pending < options.test_threads.get() && !remaining.is_empty() {
            let test = remaining.pop().unwrap();
            run_test(test, tx.clone());
            pending += 1;
        }

        let (name, result) = rx.recv().unwrap();
        summary.handle_result(name, result)?;

        pending -= 1;
    }

    // Write Test Summary
    if !options.quiet {
        summary.write_summary()?;
    }

    Ok(summary.success())
}

fn run_test(test: Test, channel: Sender<(String, TestResult)>) {
    let Test { name, testfn } = test;

    let cfg = thread::Builder::new().name(name.clone());
    cfg.spawn(move || {
        let result = match catch_unwind(AssertUnwindSafe(|| testfn())) {
            Ok(Ok(())) => TestResult::Ok,
            Ok(Err(e)) => TestResult::FailedWithMsg(format!("{:?}", e)),
            Err(_) => TestResult::Failed,
        };

        channel.send((name, result)).unwrap();
    })
    .unwrap();
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

pub struct Requirements {
    test: fn(&Path) -> Result<()>,
    test_name: String,
    root: String,
    pattern: String,
}

impl Requirements {
    pub fn new(
        test: fn(&Path) -> Result<()>,
        test_name: String,
        root: String,
        pattern: String,
    ) -> Self {
        Self {
            test,
            test_name,
            root,
            pattern,
        }
    }

    /// Generate standard test descriptors ([`test::TestDescAndFn`]) from the descriptor of
    /// `#[datatest::files(..)]`.
    ///
    /// Scans all files in a given directory, finds matching ones and generates a test descriptor
    /// for each of them.
    fn expand(&self) -> Vec<Test> {
        let root = Path::new(&self.root).to_path_buf();

        let re = regex::Regex::new(&self.pattern)
            .unwrap_or_else(|_| panic!("invalid regular expression: '{}'", self.pattern));

        let tests: Vec<_> = utils::iterate_directory(&root)
            .filter_map(|path| {
                let input_path = path.to_string_lossy();
                if re.is_match(&input_path) {
                    let testfn = self.test;
                    let name = utils::derive_test_name(&root, &path, &self.test_name);
                    let testfn = Box::new(move || (testfn)(&path));

                    Some(Test { testfn, name })
                } else {
                    None
                }
            })
            .collect();

        // We want to avoid silent fails due to typos in regexp!
        if tests.is_empty() {
            panic!(
                "no test cases found for test '{}'. Scanned directory: '{}' with pattern '{}'",
                self.test_name, self.root, self.pattern,
            );
        }

        tests
    }
}
