// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Functionality related to the command line interface of the Move prover.

use abigen::AbigenOptions;
use anyhow::anyhow;
use clap::{App, Arg};
use docgen::DocgenOptions;
use errmapgen::ErrmapOptions;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simplelog::{
    CombinedLogger, Config, ConfigBuilder, LevelPadding, SimpleLogger, TermLogger, TerminalMode,
};
use spec_lang::env::VerificationScope;
use std::sync::atomic::{AtomicBool, Ordering};

/// Represents the virtual path to the boogie prelude which is inlined into the binary.
pub const INLINE_PRELUDE: &str = "<inline-prelude>";

/// Default flags passed to boogie. Additional flags will be added to this via the -B option.

const DEFAULT_BOOGIE_FLAGS: &[&str] = &[
    "-doModSetAnalysis",
    "-printVerifiedProceduresCount:0",
    "-printModel:1",
    "-enhancedErrorMessages:1",
];

/// Atomic used to prevent re-initialization of logging.
static LOGGER_CONFIGURED: AtomicBool = AtomicBool::new(false);

/// Atomic used to detect whether we are running in test mode.
static TEST_MODE: AtomicBool = AtomicBool::new(false);

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
    /// Whether to run the ABI generator instead of the prover.
    pub run_abigen: bool,
    /// Whether to run the error map generator instead of the prover.
    pub run_errmapgen: bool,
    /// Whether to run a static analysis that computes the set of types that may be packed by the
    /// Move code under analysis instead of the prover.
    pub run_packed_types_gen: bool,
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
    /// Options for the ABI generator.
    pub abigen: AbigenOptions,
    /// Options for the error map generator.
    /// TODO: this currently create errors during deserialization, so skip them for this.
    #[serde(skip_serializing)]
    pub errmapgen: ErrmapOptions,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            prelude_path: INLINE_PRELUDE.to_string(),
            output_path: "output.bpl".to_string(),
            run_docgen: false,
            run_abigen: false,
            run_errmapgen: false,
            run_packed_types_gen: false,
            account_address: "0x234567".to_string(),
            verbosity_level: LevelFilter::Info,
            move_sources: vec![],
            move_deps: vec![],
            prover: ProverOptions::default(),
            backend: BackendOptions::default(),
            docgen: DocgenOptions::default(),
            abigen: AbigenOptions::default(),
            errmapgen: ErrmapOptions::default(),
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
    /// [deprecated] Whether to emit global axiom that resources are well-formed.
    pub resource_wellformed_axiom: bool,
    /// Whether to assume wellformedness when elements are read from memory, instead of on
    /// function entry.
    pub assume_wellformed_on_access: bool,
    /// Whether to assume a global invariant when the related memory
    /// is accessed, instead of on function entry. This is currently known to be slower
    /// if one than off, so off by default.
    pub assume_invariant_on_access: bool,
    /// Whether pack/unpack should recurse over the structure.
    pub deep_pack_unpack: bool,
    /// Whether to automatically debug trace values of specification expression leafs.
    pub debug_trace: bool,
    /// Report warnings. This is not on by default. We may turn it on if the warnings
    /// are better filtered, e.g. do not contain unused schemas intended for other modules.
    pub report_warnings: bool,
    /// Whether to dump the transformed stackless bytecode to a file
    pub dump_bytecode: bool,
    /// Number of Boogie instances to be run concurrently.
    pub num_instances: usize,
    /// Whether to run Boogie instances sequentially.
    pub sequential_task: bool,
    /// Run negative verification checks.
    pub negative_checks: bool,
}

impl Default for ProverOptions {
    fn default() -> Self {
        Self {
            generate_only: false,
            native_stubs: false,
            minimize_execution_trace: true,
            omit_model_debug: false,
            stable_test_output: false,
            verify_scope: VerificationScope::All,
            resource_wellformed_axiom: false,
            assume_wellformed_on_access: false,
            deep_pack_unpack: false,
            debug_trace: false,
            report_warnings: false,
            assume_invariant_on_access: false,
            dump_bytecode: false,
            num_instances: 1,
            sequential_task: false,
            negative_checks: false,
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
    /// Whether to use the sequence theory as the internal representation for $Vector type.
    pub vector_using_sequences: bool,
    /// A seed for the prover.
    pub random_seed: usize,
    /// The number of cores to use for parallel processing of verification conditions.
    pub proc_cores: usize,
    /// A (soft) timeout for the solver, per verification condition, in seconds.
    pub vc_timeout: usize,
    /// Whether Boogie output and log should be saved.
    pub keep_artifacts: bool,
    /// Eager threshold for quantifier instantiation.
    pub eager_threshold: usize,
    /// Lazy threshold for quantifier instantiation.
    pub lazy_threshold: usize,
    /// Whether to use the new Boogie `{:debug ..}` attribute for tracking debug values.
    pub use_boogie_debug_attrib: bool,
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
            serialize_bound: 0,
            vector_using_sequences: false,
            random_seed: 1,
            proc_cores: 1,
            vc_timeout: 40,
            keep_artifacts: false,
            eager_threshold: 100,
            lazy_threshold: 100,
            use_boogie_debug_attrib: true,
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
            .author("The Diem Core Contributors")
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
                    .long("generate-only")
                    .short("g")
                    .help("only generate boogie file but do not call boogie"),
            )
            .arg(
                Arg::with_name("warn")
                    .long("warn")
                    .short("w")
                    .help("produces warnings")
            )
            .arg(
                Arg::with_name("trace")
                    .long("trace")
                    .short("t")
                    .help("enables automatic tracing of expressions in prover errors")
            )
            .arg(
                Arg::with_name("keep")
                    .long("keep")
                    .short("k")
                    .help("keep intermediate artifacts of the backend around")
            )
            .arg(
                Arg::with_name("negative")
                    .long("negative")
                    .help("run negative verification checks")
            ).arg(
                Arg::with_name("seed")
                    .long("seed")
                    .short("s")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets a random seed for the prover (default 0)")
            )
            .arg(
                Arg::with_name("cores")
                    .long("cores")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets the number of cores to use. \
                     NOTE: multiple cores may currently lead to scrambled model \
                     output from boogie (default 1)")
            )
            .arg(
                Arg::with_name("timeout")
                    .long("timeout")
                    .short("T")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets a timeout (in seconds) for each \
                             individual verification condition (default 40)")
            )
            .arg(
                Arg::with_name("docgen")
                    .long("docgen")
                    .help("run the documentation generator instead of the prover. \
                    Generated docs will be written into the directory `./doc` unless configured otherwise via toml"),
            )
            .arg(
                Arg::with_name("docgen-template")
                    .long("docgen-template")
                    .takes_value(true)
                    .value_name("FILE")
                    .help("a template for documentation generation."),
            )
            .arg(
                Arg::with_name("abigen")
                    .long("abigen")
                    .help("run the ABI generator instead of the prover. \
                    Generated ABIs will be written into the directory `./abi` unless configured otherwise via toml"),
            )
            .arg(
                Arg::with_name("errmapgen")
                    .long("errmapgen")
                    .help("run the error map generator instead of the prover. \
                    The generated error map will be written to `errmap` unless configured otherwise"),
            )
            .arg(
                Arg::with_name("packedtypesgen")
                    .long("packedtypesgen")
                    .help("run the packed types generator instead of the prover.")
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
            .arg(
                Arg::with_name("eager-threshold")
                    .long("eager-threshold")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets the eager threshold for quantifier instantiation (default 100)")
            )
            .arg(
                Arg::with_name("lazy-threshold")
                    .long("lazy-threshold")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets the lazy threshold for quantifier instantiation (default 100)")
            )
            .arg(
                Arg::with_name("dump-bytecode")
                    .long("dump-bytecode")
                    .help("whether to dump the transformed bytecode to a file")
            )
            .arg(
                Arg::with_name("num-instances")
                    .long("num-instances")
                    .takes_value(true)
                    .value_name("NUMBER")
                    .validator(is_number)
                    .help("sets the number of Boogie instances to run concurrently (default 1)")
            )
            .arg(
                Arg::with_name("sequential")
                    .long("sequential")
                    .help("whether to run the Boogie instances sequentially")
            )
            .arg(
                Arg::with_name("use-cvc4")
                    .long("use-cvc4")
                    .help("use cvc4 solver instead of z3")
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
            if matches.is_present("config-str") {
                return Err(anyhow!(
                    "currently, if `--config` (including via $MOVE_PROVER_CONFIG) is given \
                       `--config-str` cannot be used. Consider editing your \
                       configuration file instead."
                ));
            }
            Self::create_from_toml_file(matches.value_of("config").unwrap())?
        } else if matches.is_present("config-str") {
            Self::create_from_toml(matches.value_of("config-str").unwrap())?
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
        if matches.is_present("generate-only") {
            options.prover.generate_only = true;
        }
        if matches.occurrences_of("sources") > 0 {
            options.move_sources = get_vec("sources");
        }
        if matches.occurrences_of("dependencies") > 0 {
            options.move_deps = get_vec("dependencies");
        }
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
        if matches.is_present("docgen-template") {
            options.run_docgen = true;
            options.docgen.root_doc_templates = vec![matches
                .value_of("docgen-template")
                .map(|s| s.to_string())
                .unwrap()]
        }
        if matches.is_present("abigen") {
            options.run_abigen = true;
        }
        if matches.is_present("errmapgen") {
            options.run_errmapgen = true;
        }
        if matches.is_present("packedtypesgen") {
            options.run_packed_types_gen = true;
        }
        if matches.is_present("warn") {
            options.prover.report_warnings = true;
        }
        if matches.is_present("trace") {
            options.prover.debug_trace = true;
        }
        if matches.is_present("dump-bytecode") {
            options.prover.dump_bytecode = true;
        }
        if matches.is_present("num-instances") {
            let num_instances = matches
                .value_of("num-instances")
                .unwrap()
                .parse::<usize>()?;
            options.prover.num_instances = std::cmp::max(num_instances, 1); // at least one instance
        }
        if matches.is_present("sequential") {
            options.prover.sequential_task = true;
        }
        if matches.is_present("keep") {
            options.backend.keep_artifacts = true;
        }
        if matches.is_present("negative") {
            options.prover.negative_checks = true;
        }
        if matches.is_present("seed") {
            options.backend.random_seed = matches.value_of("seed").unwrap().parse::<usize>()?;
        }
        if matches.is_present("timeout") {
            options.backend.vc_timeout = matches.value_of("timeout").unwrap().parse::<usize>()?;
        }
        if matches.is_present("cores") {
            options.backend.proc_cores = matches.value_of("cores").unwrap().parse::<usize>()?;
        }
        if matches.is_present("eager-threshold") {
            options.backend.eager_threshold = matches
                .value_of("eager-threshold")
                .unwrap()
                .parse::<usize>()?;
        }
        if matches.is_present("lazy-threshold") {
            options.backend.lazy_threshold = matches
                .value_of("lazy-threshold")
                .unwrap()
                .parse::<usize>()?;
        }
        if matches.is_present("use-cvc4") {
            options.backend.use_cvc4 = true;
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
        if !self.prover.negative_checks {
            // Right now, we let boogie only produce one error per procedure. The boogie wrapper isn't
            // capable to sort out multiple errors and associate them with models otherwise.
            add(&["-errorLimit:1"]);
        }
        if self.backend.use_cvc4 {
            add(&[
                "-proverOpt:SOLVER=cvc4",
                &format!("-proverOpt:PROVER_PATH={}", &self.backend.cvc4_exe),
            ]);
        } else {
            add(&[&format!("-proverOpt:PROVER_PATH={}", &self.backend.z3_exe)]);
        }
        if self.backend.use_array_theory {
            add(&[
                "-useArrayTheory",
                "/proverOpt:O:smt.array.extensional=false",
            ]);
        } else {
            add(&[&format!(
                "-proverOpt:O:smt.QI.EAGER_THRESHOLD={}",
                self.backend.eager_threshold
            )]);
            add(&[&format!(
                "-proverOpt:O:smt.QI.LAZY_THRESHOLD={}",
                self.backend.lazy_threshold
            )]);
        }
        add(&[&format!(
            "-vcsCores:{}",
            if self.prover.stable_test_output {
                // Do not use multiple cores if stable test output is requested.
                // Error messages may appear in non-deterministic order otherwise.
                1
            } else {
                self.backend.proc_cores
            }
        )]);
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

    /// Adjust a timeout value, given in seconds, for the runtime environment.
    pub fn adjust_timeout(&self, time: usize) -> usize {
        // If running on a Linux flavor as in Ci, add 100% to the timeout for added
        // robustness against flakiness.
        match std::env::consts::OS {
            "linux" | "freebsd" | "openbsd" => time + time,
            _ => time,
        }
    }

    /// Convenience function to enable debugging (like high verbosity) on this instance.
    pub fn enable_debug(&mut self) {
        self.verbosity_level = LevelFilter::Debug;
    }
}
