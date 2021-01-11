// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

/// Backend options.
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
    pub use_boogie_debug_attrib: bool,
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
            proc_cores: 1,
            vc_timeout: 40,
            keep_artifacts: false,
            eager_threshold: 100,
            lazy_threshold: 100,
            use_boogie_debug_attrib: true,
        }
    }
}
