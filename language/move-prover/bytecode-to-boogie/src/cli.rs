// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Functionality related to the command line interface of the move prover.

use clap::{App, Arg};
use log::{error, LevelFilter};
use simplelog::{
    CombinedLogger, Config, ConfigBuilder, LevelPadding, SimpleLogger, TermLogger, TerminalMode,
};
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};

/// Represents the virtual path to the boogie prelude which is inlined into the binary.
pub const INLINE_PRELUDE: &str = "<inline-prelude>";

/// Default flags passed to boogie. Additional flags will be added to this via the -B option.

const DEFAULT_BOOGIE_FLAGS: &[&str] = &[
    "-doModSetAnalysis",
    "-noinfer",
    "-printVerifiedProceduresCount:0",
];

/// Atomic used to prevent re-initialization of logging.
static LOGGER_CONFIGURED: AtomicBool = AtomicBool::new(false);

/// Atomic used to detect whether we are running in test mode.
static TEST_MODE: AtomicBool = AtomicBool::new(false);

/// Represents options provided to the tool.
#[derive(Debug)]
pub struct Options {
    /// Path to the boogie prelude. The special string `INLINE_PRELUDE` is used to refer to
    /// a prelude build into this binary.
    pub prelude_path: String,
    /// The path to the boogie output which represents the verification problem.
    pub output_path: String,
    /// Verbosity level for logging
    pub verbosity_level: LevelFilter,
    /// The paths to the mvir sources.
    pub mvir_sources: Vec<String>,
    /// Path to the boogie executable.
    pub boogie_exe: String,
    /// Path to the z3 executable.
    pub z3_exe: String,
    /// Whether to use cvc4
    pub use_cvc4: bool,
    /// Path to the cvc4 executable.
    pub cvc4_exe: String,
    /// List of flags to pass on to boogie.
    pub boogie_flags: Vec<String>,
    /// Whether to only generate boogie
    pub generate_only: bool,
    /// Whether to generate stubs for native functions.
    pub native_stubs: bool,
    /// Whether to minimize execution traces in errors.
    pub minimize_execution_trace: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prelude_path: INLINE_PRELUDE.to_string(),
            output_path: "output.bpl".to_string(),
            verbosity_level: LevelFilter::Info,
            mvir_sources: vec![],
            boogie_exe: "".to_string(),
            z3_exe: "".to_string(),
            use_cvc4: false,
            cvc4_exe: "".to_string(),
            boogie_flags: vec![],
            generate_only: false,
            native_stubs: false,
            minimize_execution_trace: true,
        }
    }
}

impl Options {
    // Creates options from command line arguments. This parses the arguments and terminates
    // the program on errors, printing usage information. The first argument is expected to be
    // the program name.
    pub fn initialize_from_args(&mut self, args: &[String]) {
        // Clap definition of the command line interface.
        let cli = App::new("mvp")
            .version("0.1.0")
            .about("The Move Prover")
            .author("The Libra Core Contributors")
            .arg(
                Arg::with_name("prelude")
                    .short("p")
                    .long("prelude")
                    .value_name("BOOGIE_FILE")
                    .default_value(&INLINE_PRELUDE)
                    .help("path to an alternative boogie prelude"),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .value_name("BOOGIE_FILE")
                    .default_value("output.bpl")
                    .help("path to the boogie output which represents the verification problem"),
            )
            .arg(
                Arg::with_name("verbosity")
                    .short("v")
                    .long("verbose")
                    .possible_values(&["error", "warn", "info", "debug"])
                    .default_value("info")
                    .help("verbosity level"),
            )
            .arg(
                Arg::with_name("generate-only")
                    .short("g")
                    .long("generate-only")
                    .help("only generate boogie file but do not call boogie"),
            )
            .arg(
                Arg::with_name("native-stubs")
                    .long("native-stubs")
                    .help("whether to generate stubs for native functions"),
            )
            .arg(
                Arg::with_name("boogie-exe")
                    .long("boogie-exe")
                    .default_value("boogie")
                    .env("BOOGIE_EXE")
                    .value_name("PATH")
                    .help("path to the boogie executable"),
            )
            .arg(
                Arg::with_name("z3-exe")
                    .long("z3-exe")
                    .default_value("z3")
                    .env("Z3_EXE")
                    .value_name("PATH")
                    .help("path to the z3 executable"),
            )
            .arg(
                Arg::with_name("use-cvc4")
                    .long("use-cvc4")
                    .help("whether to use cvc4 instead of z3 as a backend"),
            )
            .arg(
                Arg::with_name("cvc4-exe")
                    .long("cvc4-exe")
                    .takes_value(true)
                    .default_value("cvc4")
                    .env("CVC4_EXE")
                    .value_name("PATH")
                    .help("path to the cvc4 executable"),
            )
            .arg(
                Arg::with_name("boogie-flags")
                    .short("B")
                    .long("boogie")
                    .multiple(true)
                    // See documentation of multiple() why the next option is needed.
                    // This effectively still allows us to have `-B opt1 -B opt2 ...`,
                    // but not `-B opt1 opt2` because the latter messes with positional
                    // arguments.
                    .number_of_values(1)
                    .takes_value(true)
                    .value_name("BOOGIE_FLAG")
                    .help("specifies a flag to be passed on to boogie"),
            )
            .arg(
                Arg::with_name("sources")
                    .multiple(true)
                    .value_name("MVIR_FILE")
                    .min_values(1)
                    .help("path to a mvir file (with embedded spec)"),
            );

        // Parse the arguments. This will abort the program on parsing errors and print help.
        // It will also accept options like --help.
        let matches = cli.get_matches_from(args);
        let get_with_default = |s: &str| matches.value_of(s).expect("Expected default").to_string();
        let get_vec = |s: &str| -> Vec<String> {
            match matches.values_of(s) {
                Some(vs) => vs.map(|v| v.to_string()).collect(),
                _ => vec![],
            }
        };

        self.prelude_path = get_with_default("prelude");
        self.output_path = get_with_default("output");
        self.verbosity_level = match get_with_default("verbosity").as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            _ => unreachable!("should not happen"),
        };
        self.generate_only = matches.is_present("generate-only");
        self.native_stubs = matches.is_present("native-stubs");
        self.use_cvc4 = matches.is_present("use-cvc4");
        self.boogie_exe = get_with_default("boogie-exe");
        self.z3_exe = get_with_default("z3-exe");
        self.cvc4_exe = get_with_default("cvc4-exe");
        self.boogie_flags = get_vec("boogie-flags");
        self.mvir_sources = get_vec("sources");
    }

    /// Sets up logging based on provided options. This should be called as early as possible
    /// and before any use of info!, warn! etc.
    pub fn setup_logging(&self) {
        CombinedLogger::init(vec![TermLogger::new(
            self.verbosity_level,
            ConfigBuilder::new()
                .set_time_level(LevelFilter::Debug)
                .set_level_padding(LevelPadding::Off)
                .build(),
            TerminalMode::Mixed,
        )
        .expect("Unexpected TermLogger init failure")])
        .expect("Unexpected CombinedLogger init failure");
    }

    pub fn setup_logging_for_test(&self) {
        // Loggers are global static, so we have to protect against reinitializing.
        if LOGGER_CONFIGURED.compare_and_swap(false, true, Ordering::Relaxed) {
            return;
        }
        TEST_MODE.store(true, Ordering::Relaxed);
        SimpleLogger::init(self.verbosity_level, Config::default())
            .expect("UnexpectedSimpleLogger failure");
    }

    /// Returns command line to call boogie.
    pub fn get_boogie_command(&self, boogie_file: &str) -> Vec<String> {
        let mut result = vec![self.boogie_exe.clone()];
        let mut add = |sl: &[&str]| result.extend(sl.iter().map(|s| (*s).to_string()));
        add(DEFAULT_BOOGIE_FLAGS);
        if self.use_cvc4 {
            add(&[
                "-proverOpt:SOLVER=cvc4",
                &format!("-cvc4exe:{}", &self.cvc4_exe),
            ]);
        } else {
            add(&[&format!("-z3exe:{}", &self.z3_exe)]);
        }
        for f in &self.boogie_flags {
            add(&[f.as_str()]);
        }
        add(&[boogie_file]);
        result
    }

    /// Returns name of file where to log boogie output.
    pub fn get_boogie_log_file(&self, boogie_file: &str) -> String {
        format!("{}.log", boogie_file)
    }
}

/// If result is not OK, log error and abort program. In difference to result.expect(), this
/// is dealing with a user input, not a program error, and thus does not print a stacktrace.
pub fn abort_on_error<T, E: Display>(r: Result<T, E>, msg: &str) -> T {
    match r {
        Ok(x) => x,
        Err(e) => abort_with_error(&format!("{}: {}", msg, e)),
    }
}

/// Abort program after printing error message. This is dealing with a user input error,
/// so panic! with a stack trace is not used.
pub fn abort_with_error<T>(msg: &str) -> T {
    error!("{}", msg);
    if TEST_MODE.load(Ordering::Relaxed) {
        // We do not want to exit the process during tests, because we do not see buffered terminal
        // output if so.
        panic!("test aborted");
    } else {
        std::process::exit(1);
    }
}
