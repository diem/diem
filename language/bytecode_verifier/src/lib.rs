// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Verifies bytecode sanity.

#![feature(exhaustive_patterns)]
#![feature(never_type)]

// Bounds checks are implemented in the `vm` crate.
pub mod abstract_interpreter;
pub mod abstract_state;
pub mod check_duplication;
pub mod code_unit_verifier;
pub mod control_flow_graph;
pub mod nonce;
pub mod partition;
pub mod resources;
pub mod signature;
pub mod stack_usage_verifier;
pub mod struct_defs;
#[cfg(test)]
mod unit_tests;
pub mod verifier;

pub use check_duplication::DuplicationChecker;
pub use code_unit_verifier::CodeUnitVerifier;
pub use resources::ResourceTransitiveChecker;
pub use signature::SignatureChecker;
pub use stack_usage_verifier::StackUsageVerifier;
pub use struct_defs::RecursiveStructDefChecker;
pub use verifier::{
    verify_main_signature, verify_module, verify_module_dependencies, verify_script,
    verify_script_dependencies, VerifiedModule, VerifiedScript,
};
