// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Verifies bytecode sanity.

// Bounds checks are implemented in the `vm` crate.
pub mod absint;
pub mod abstract_state;
pub mod acquires_list_verifier;
pub mod check_duplication;
pub mod code_unit_verifier;
pub mod control_flow_graph;
pub mod instantiation_loops;
pub mod resolver;
pub mod resources;
pub mod signature;
pub mod stack_usage_verifier;
pub mod struct_defs;
pub mod type_memory_safety;
pub mod unused_entries;

pub mod verifier;

pub use check_duplication::DuplicationChecker;
pub use code_unit_verifier::CodeUnitVerifier;
pub use resources::ResourceTransitiveChecker;
pub use signature::SignatureChecker;
pub use stack_usage_verifier::StackUsageVerifier;
pub use struct_defs::RecursiveStructDefChecker;
pub use verifier::{
    batch_verify_modules, verify_main_signature, verify_module_dependencies,
    verify_script_dependencies, VerifiedModule, VerifiedScript,
};
