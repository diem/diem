// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_model::model::VerificationScope;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

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
        }
    }
}

pub static PROVER_DEFAULT_OPTIONS: Lazy<ProverOptions> = Lazy::new(ProverOptions::default);
