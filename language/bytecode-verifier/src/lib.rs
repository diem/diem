// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! Verifies bytecode sanity.

// Bounds checks are implemented in the `vm` crate.
pub mod absint;
pub mod acquires_list_verifier;
pub mod check_duplication;
pub mod code_unit_verifier;
pub mod constants;
pub mod control_flow;
pub mod control_flow_graph;
pub mod instantiation_loops;
pub mod instruction_consistency;
pub mod locals_safety;
pub mod reference_safety;
pub mod resolver;
pub mod resources;
pub mod signature;
pub mod stack_usage_verifier;
pub mod struct_defs;
pub mod type_safety;
pub mod unused_entries;

pub mod verifier;

pub use check_duplication::DuplicationChecker;
pub use code_unit_verifier::CodeUnitVerifier;
pub use instruction_consistency::InstructionConsistency;
pub use resources::ResourceTransitiveChecker;
pub use signature::SignatureChecker;
pub use stack_usage_verifier::StackUsageVerifier;
pub use struct_defs::RecursiveStructDefChecker;
pub use verifier::{
    batch_verify_modules, verify_main_signature, verify_module_dependencies,
    verify_script_dependencies, VerifiedModule, VerifiedScript,
};
use vm::{
    access::ModuleAccess,
    file_format::{Kind, SignatureToken},
    CompiledModule,
};

//
// Helpers functions for signatures
//

pub(crate) fn kind(module: &CompiledModule, ty: &SignatureToken, constraints: &[Kind]) -> Kind {
    use SignatureToken::*;

    match ty {
        // The primitive types & references have kind unrestricted.
        Bool | U8 | U64 | U128 | Address | Reference(_) | MutableReference(_) => Kind::Copyable,
        Signer => Kind::Resource,
        TypeParameter(idx) => constraints[*idx as usize],
        Vector(ty) => kind(module, ty, constraints),
        Struct(idx) => {
            let sh = module.struct_handle_at(*idx);
            if sh.is_nominal_resource {
                Kind::Resource
            } else {
                Kind::Copyable
            }
        }
        StructInstantiation(idx, type_args) => {
            let sh = module.struct_handle_at(*idx);
            if sh.is_nominal_resource {
                return Kind::Resource;
            }
            // Gather the kinds of the type actuals.
            let kinds = type_args
                .iter()
                .map(|ty| kind(module, ty, constraints))
                .collect::<Vec<_>>();
            // Derive the kind of the struct.
            //   - If any of the type actuals is `all`, then the struct is `all`.
            //     - `all` means some part of the type can be either `resource` or
            //       `unrestricted`.
            //     - Therefore it is also impossible to determine the kind of the type as a
            //       whole, and thus `all`.
            //   - If none of the type actuals is `all`, then the struct is a resource if
            //     and only if one of the type actuals is `resource`.
            kinds.iter().cloned().fold(Kind::Copyable, Kind::join)
        }
    }
}
