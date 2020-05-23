// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Functionality related to the command line interface of the Move prover.

use anyhow::anyhow;
use clap::{App, Arg};
use docgen::docgen::DocgenOptions;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simplelog::{
    CombinedLogger, Config, ConfigBuilder, LevelPadding, SimpleLogger, TermLogger, TerminalMode,
};
use std::sync::atomic::{AtomicBool, Ordering};

/// Represents the virtual path to the boogie prelude which is inlined into the binary.
pub const INLINE_PRELUDE: &str = "<inline-prelude>";

/// Default flags passed to boogie. Additional flags will be added to this via the -B option.

const DEFAULT_BOOGIE_FLAGS: &[&str] = &[
    "-doModSetAnalysis",
    "-noinfer",
    "-printVerifiedProceduresCount:0",
    "-printModel:4",
    // Right now, we let boogie only produce one error per procedure. The boogie wrapper isn't
    // capable to sort out multiple errors and associate them with models otherwise.
    "-errorLimit:1",
];

/// Atomic used to prevent re-initialization of logging.
static LOGGER_CONFIGURED: AtomicBool = AtomicBool::new(false);

/// Atomic used to detect whether we are running in test mode.
static TEST_MODE: AtomicBool = AtomicBool::new(false);

/// Default for what functions to verify.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum VerificationScope {
    /// Verify only public functions.
    Public,
    /// Verify all functions.
    All,
    /// Verify no functions
    None,
}

impl Default for VerificationScope {
    fn default() -> Self {
        Self::Public
    }
}

/// Represents options provided to the tool. Most of those options are configured via a toml
/// source; some over the command line flags.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    /// Path to the boogie prelude. The special string `INLINE_PRELUDE` is used to refer to
    /// a prelude build into this binary.
    pub prelude_path: String,
    /// The path to the boogie output which represents the verification problem.
    pub output_path: String,
    /// Verbosity level for logging.
    pub verbosity_level: LevelFilter,
    /// Whether to run the documentation generator instead of the prover.
    pub run_docgen: bool,
    /// An account address to use if none is specified in the source.
    pub account_address: String,
    /// The paths to the Move sources.
    pub move_sources: Vec<String>,
    /// The paths to any dependencies for the Move sources. Those will not be verified but
    /// can be used by `move_sources`.
    pub move_deps: Vec<String>,
    /// Options for the prover.
    pub prover: ProverOptions,
    /// Options for the prover backend.
    pub backend: BackendOptions,
    /// Options for the documentation generator.
    pub docgen: DocgenOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            prelude_path: INLINE_PRELUDE.to_string(),
            output_path: "output.bpl".to_string(),
            run_docgen: false,
            account_address: "0x234567".to_string(),
            verbosity_level: LevelFilter::Info,
            move_sources: vec![],
            move_deps: vec![],
            docgen: DocgenOptions::default(),
            prover: ProverOptions::default(),
            backend: BackendOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProverOptions {
    /// Whether to only generate backend code.
    pub generate_only: bool,
    /// Whether to generate stubs for native functions.
    pub native_stubs: bool,
    /// Whether to minimize execution traces in errors.
    pub minimize_execution_trace: bool,
    /// Whether to omit debug information in generated model.
    pub omit_model_debug: bool,
    /// Whether output for e.g. diagnosis shall be stable/redacted so it can be used in test
    /// output.
    pub stable_test_output: bool,
    /// Scope of what functions to verify.
    pub verify_scope: VerificationScope,
    /// Whether to emit global axiom that resources are well-formed.
    pub resource_wellformed_axiom: bool,
    /// Whether to automatically debug trace values of specification expression leafs.
    pub debug_trace: bool,
}

impl Default for ProverOptions {
    fn default() -> Self {
        Self {
            generate_only: false,
            native_stubs: false,
            minimize_execution_trace: true,
            omit_model_debug: false,
            stable_test_output: false,
            verify_scope: VerificationScope::Public,
            resource_wellformed_axiom: true,
            debug_trace: false,
        }
    }
}

/// Backend options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BackendOptions {
    /// Path to the boogie executable.
    pub boogie_exe: String,
    /// Path to the z3 executable.
    pub z3_exe: String,
    /// Whether to use cvc4.
    pub use_cvc4: bool,
    /// Path to the cvc4 executable.
    pub cvc4_exe: String,
    /// List of flags to pass on to boogie.
    pub boogie_flags: Vec<String>,
    /// Whether to use native array theory.
    pub use_array_theory: bool,
    /// Whether to produce an SMT file for each verification problem.
    pub generate_smt: bool,
    /// Whether native instead of stratified equality should be used.
    pub native_equality: bool,
    /// A string determining the type of requires used for parameter type checks. Can be
    /// `"requires"` or `"free requires`".
    pub type_requires: String,
    /// The depth until which stratified functions are expanded.
    pub stratification_depth: usize,
    /// A string to be used to inline a function of medium size. Can be empty or `{:inline}`.
    pub aggressive_func_inline: String,
    /// A string to be used to inline a function of small size. Can be empty or `{:inline}`.
    pub func_inline: String,
    /// A bound to apply to the length of serialization results.
    pub serialize_bound: usize,
    /// How many times to call the prover backend for the verification problem. This is used for
    /// benchmarking.
    pub bench_repeat: usize,
}

impl Default for BackendOptions {
    fn default() -> Self {
        let get_env = |s| std::env::var(s).unwrap_or_else(|_| String::new());
        Self {
            bench_repeat: 1,
            boogie_exe: get_env("BOOGIE_EXE"),
            z3_exe: get_env("Z3_EXE"),
            use_cvc4: false,
            cvc4_exe: get_env("CVC4_EXE"),
            boogie_flags: vec![],
            use_array_theory: false,
            generate_smt: false,
            native_equality: false,
            type_requires: "free requires".to_owned(),
            stratification_depth: 4,
            aggressive_func_inline: "".to_owned(),
            func_inline: "{:inline}".to_owned(),
            serialize_bound: 4,
        }
    }
}

impl Options {
    /// Creates options from toml configuration source.
    pub fn create_from_toml(toml_source: &str) -> anyhow::Result<Options> {
        Ok(toml::from_str(toml_source)?)
    }

    /// Creates options from toml configuration file.
    pub fn create_from_toml_file(toml_file: &str) -> anyhow::Result<Options> {
        Self::create_from_toml(&std::fs::read_to_string(toml_file)?)
    }

    // Creates options from command line arguments. This parses the arguments and terminates
    // the program on errors, printing usage information. The first argument is expected to be
    // the program name.
    pub fn create_from_args(args: &[String]) -> anyhow::Result<Options> {
        // Clap definition of the command line interface.
        let is_number = |s: String| {
            s.parse::<usize>()
                .map(|_| ())
                .map_err(|_| "expected number".to_string())
        };
        let cli = App::new("mvp")
            .version("0.1.0")
            .about("The Move Prover")
            .author("The Libra Core Contributors")
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .takes_value(true)
                    .value_name("TOML_FILE")
                    .env("MOVE_PROVER_CONFIG")
                    .help("path to a configuration file. \
                     Values in this file will be overridden by command line flags"),
            )
            .arg(
                Arg::with_name("config-str")
                    .conflicts_with("config")
                    .short("C")
                    .long("config-str")
                    .takes_value(true)
                    .multiple(true)
                    .number_of_values(1)
                    .value_name("TOML_STRING")
                    .help("inline configuration string in toml syntax. Can be repeated. \
                     Use as in `-C=prover.opt=value -C=backend.opt=value`"),
            )
            .arg(
                Arg::with_name("print-config")
                    .long("print-config")
                    .help("prints the effective toml configuration, then exits")
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .takes_value(true)
                    .value_name("BOOGIE_FILE")
                    .help("path to the boogie output which represents the verification problem"),
            )
            .arg(
                Arg::with_name("verbosity")
                    .short("v")
                    .long("verbose")
                    .takes_value(true)
                    .possible_values(&["error", "warn", "info", "debug"])
                    .help("verbosity level."),
            )
            .arg(
                Arg::with_name("generate-only")
                    .short("g")
                    .long("generate-only")
                    .help("only generate boogie file but do not call boogie"),
            )
            .arg(
                Arg::with_name("trace")
                    .long("trace")
                    .short("t")
                    .help("enables automatic tracing of expressions in prover errors")
            )
            .arg(
                Arg::with_name("docgen")
                    .long("docgen")
                    .help("run the documentation generator instead of the prover. \
                    Generated docs will be written into the directory at `--output=<path>`"),
            )
            .arg(
                Arg::with_name("verify")
                    .long("verify")
                    .takes_value(true)
                    .possible_values(&["public", "all", "none"])
                    .value_name("SCOPE")
                    .help("default scope of verification \
                    (can be overridden by `pragma verify=true|false`)"),
            )
            .arg(
                Arg::with_name("bench-repeat")
                    .long("bench-repeat")
                    .takes_value(true)
                    .value_name("COUNT")
                    .validator(is_number)
                    .help(
                        "for benchmarking: how many times to call the backend on the verification problem",
                    ),
            )
            .arg(
                Arg::with_name("dependencies")
                    .long("dependency")
                    .short("d")
                    .multiple(true)
                    .number_of_values(1)
                    .takes_value(true)
                    .value_name("PATH_TO_DEPENDENCY")
                    .help("path to a Move file, or a directory which will be searched for \
                    Move files, containing dependencies which will not be verified")
            )
            .arg(
                Arg::with_name("sources")
                    .multiple(true)
                    .value_name("PATH_TO_SOURCE_FILE")
                    .min_values(1)
                    .help("the source files to verify"),
            )
            .after_help("More options available via `--config file` or `--config-str str`. \
            Use `--print-config` to see format and current values. \
            See `move-prover/src/cli.rs::Option` for documentation.");

        // Parse the arguments. This will abort the program on parsing errors and print help.
        // It will also accept options like --help.
        let matches = cli.get_matches_from(args);

        // Initialize options.
        let get_vec = |s: &str| -> Vec<String> {
            match matches.values_of(s) {
                Some(vs) => vs.map(|v| v.to_string()).collect(),
                _ => vec![],
            }
        };

        let mut options = if matches.is_present("config") {
            Self::create_from_toml_file(matches.value_of("config").unwrap())?
        } else if matches.is_present("config-str") {
            let config_lines = get_vec("config-str").join("\n");
            Self::create_from_toml(&config_lines)?
        } else {
            Options::default()
        };

        // Analyze arguments.
        if matches.is_present("output") {
            options.output_path = matches.value_of("output").unwrap().to_string();
        }
        if matches.is_present("verbosity") {
            options.verbosity_level = match matches.value_of("verbosity").unwrap() {
                "error" => LevelFilter::Error,
                "warn" => LevelFilter::Warn,
                "info" => LevelFilter::Info,
                "debug" => LevelFilter::Debug,
                _ => unreachable!("should not happen"),
            }
        }
        options.move_sources = get_vec("sources");
        options.move_deps = get_vec("dependencies");
        if matches.is_present("verify") {
            options.prover.verify_scope = match matches.value_of("verify").unwrap() {
                "public" => VerificationScope::Public,
                "all" => VerificationScope::All,
                "none" => VerificationScope::None,
                _ => unreachable!("should not happen"),
            }
        }
        if matches.is_present("bench-repeat") {
            options.backend.bench_repeat =
                matches.value_of("bench-repeat").unwrap().parse::<usize>()?;
        }
        if matches.is_present("docgen") {
            options.run_docgen = true;
        }
        if matches.is_present("trace") {
            options.prover.debug_trace = true;
        }
        if matches.is_present("print-config") {
            println!("{}", toml::to_string(&options).unwrap());
            Err(anyhow!("exiting"))
        } else {
            Ok(options)
        }
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
        )])
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
        let mut result = vec![self.backend.boogie_exe.clone()];
        let mut add = |sl: &[&str]| result.extend(sl.iter().map(|s| (*s).to_string()));
        add(DEFAULT_BOOGIE_FLAGS);
        if self.backend.use_cvc4 {
            add(&[
                "-proverOpt:SOLVER=cvc4",
                &format!("-proverOpt:PROVER_PATH={}", &self.backend.cvc4_exe),
            ]);
        } else {
            add(&[&format!("-proverOpt:PROVER_PATH={}", &self.backend.z3_exe)]);
        }
        if self.backend.use_array_theory {
            add(&["-useArrayTheory"]);
        }
        add(&["-proverOpt:O:smt.QI.EAGER_THRESHOLD=100"]);
        add(&["-proverOpt:O:smt.QI.LAZY_THRESHOLD=100"]);
        // TODO: see what we can make out of these flags.
        //add(&["-proverOpt:O:smt.QI.PROFILE=true"]);
        //add(&["-proverOpt:O:trace=true"]);
        //add(&["-proverOpt:VERBOSITY=3"]);
        //add(&["-proverOpt:C:-st"]);
        if self.backend.generate_smt {
            add(&["-proverLog:@PROC@.smt"]);
        }
        for f in &self.backend.boogie_flags {
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
