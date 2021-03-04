// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Default flags passed to boogie. Additional flags will be added to this via the -B option.
const DEFAULT_BOOGIE_FLAGS: &[&str] = &[
    "-doModSetAnalysis",
    "-printVerifiedProceduresCount:0",
    "-printModel:1",
    "-enhancedErrorMessages:1",
];

/// Boogie options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BoogieOptions {
    /// Path to the boogie executable.
    pub boogie_exe: String,
    /// Path to the z3 executable.
    pub z3_exe: String,
    /// Whether to use cvc4.
    pub use_cvc4: bool,
    /// Path to the cvc4 executable.
    pub cvc4_exe: String,
    /// Whether to generate debug trace code.
    pub debug_trace: bool,
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
    pub stable_test_output: bool,
    /// Number of Boogie instances to be run concurrently.
    pub num_instances: usize,
    /// Whether to run Boogie instances sequentially.
    pub sequential_task: bool,
}

impl Default for BoogieOptions {
    fn default() -> Self {
        let get_env = |s| std::env::var(s).unwrap_or_else(|_| String::new());
        Self {
            bench_repeat: 1,
            boogie_exe: get_env("BOOGIE_EXE"),
            z3_exe: get_env("Z3_EXE"),
            use_cvc4: false,
            cvc4_exe: get_env("CVC4_EXE"),
            boogie_flags: vec![],
            debug_trace: false,
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
            proc_cores: 4,
            vc_timeout: 40,
            keep_artifacts: false,
            eager_threshold: 100,
            lazy_threshold: 100,
            stable_test_output: false,
            num_instances: 1,
            sequential_task: false,
        }
    }
}

impl BoogieOptions {
    /// Returns command line to call boogie.
    pub fn get_boogie_command(&self, boogie_file: &str) -> Vec<String> {
        let mut result = vec![self.boogie_exe.clone()];
        let mut add = |sl: &[&str]| result.extend(sl.iter().map(|s| (*s).to_string()));
        add(DEFAULT_BOOGIE_FLAGS);
        if self.use_cvc4 {
            add(&[
                "-proverOpt:SOLVER=cvc4",
                &format!("-proverOpt:PROVER_PATH={}", &self.cvc4_exe),
            ]);
        } else {
            add(&[&format!("-proverOpt:PROVER_PATH={}", &self.z3_exe)]);
        }
        if self.use_array_theory {
            add(&[
                "-useArrayTheory",
                "/proverOpt:O:smt.array.extensional=false",
            ]);
        } else {
            add(&[&format!(
                "-proverOpt:O:smt.QI.EAGER_THRESHOLD={}",
                self.eager_threshold
            )]);
            add(&[&format!(
                "-proverOpt:O:smt.QI.LAZY_THRESHOLD={}",
                self.lazy_threshold
            )]);
        }
        add(&[&format!(
            "-vcsCores:{}",
            if self.stable_test_output {
                // Do not use multiple cores if stable test output is requested.
                // Error messages may appear in non-deterministic order otherwise.
                1
            } else {
                self.proc_cores
            }
        )]);
        // TODO: see what we can make out of these flags.
        //add(&["-proverOpt:O:smt.QI.PROFILE=true"]);
        //add(&["-proverOpt:O:trace=true"]);
        //add(&["-proverOpt:VERBOSITY=3"]);
        //add(&["-proverOpt:C:-st"]);
        if self.generate_smt {
            add(&["-proverLog:@PROC@.smt"]);
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

    /// Adjust a timeout value, given in seconds, for the runtime environment.
    pub fn adjust_timeout(&self, time: usize) -> usize {
        // If running on a Linux flavor as in Ci, add 100% to the timeout for added
        // robustness against flakiness.
        match std::env::consts::OS {
            "linux" | "freebsd" | "openbsd" => usize::saturating_add(time, time),
            _ => time,
        }
    }
}
