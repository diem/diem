// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod stdlib;
pub mod transaction_scripts;

use bytecode_source_map::source_map::{ModuleSourceMap, SourceMap};
use bytecode_verifier::{verify_module_dependencies, VerifiedModule};
use ir_to_bytecode::compiler::compile_module;
use libra_types::{account_address::AccountAddress, account_config};
use move_ir_types::ast::Loc;
use once_cell::sync::Lazy;

static ANNOTATED_STDLIB: Lazy<(Vec<VerifiedModule>, SourceMap<Loc>)> =
    Lazy::new(|| build_stdlib(account_config::CORE_CODE_ADDRESS));

/// Returns a reference to the standard library, compiled with the
/// [default address](account_config::CORE_CODE_ADDRESS).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
pub fn stdlib_modules() -> &'static [VerifiedModule] {
    &*ANNOTATED_STDLIB.0
}

/// Returns a reference to the source maps for the standard library.
///
/// The order of the modules returned in this follows the same order as the modules returned in the
/// stdlib_modules.
pub fn stdlib_source_map() -> &'static [ModuleSourceMap<Loc>] {
    &*ANNOTATED_STDLIB.1
}

/// Builds and returns a copy of the standard library with this address as the self address.
///
/// A copy of the stdlib built with the [default address](account_config::CORE_CODE_ADDRESS) is
/// available through [`stdlib_modules`].
pub fn build_stdlib(address: AccountAddress) -> (Vec<VerifiedModule>, SourceMap<Loc>) {
    let mut stdlib_modules = vec![];
    let mut stdlib_source_maps = vec![];

    for module_def in stdlib::module_defs() {
        let (compiled_module, source_map) =
            compile_module(address, (*module_def).clone(), &stdlib_modules)
                .expect("stdlib module failed to compile");
        let verified_module =
            VerifiedModule::new(compiled_module).expect("stdlib module failed to verify");

        let verification_errors = verify_module_dependencies(&verified_module, &stdlib_modules);
        // Fail if the module doesn't verify
        for e in &verification_errors {
            println!("{:?}", e);
        }
        assert!(verification_errors.is_empty());

        stdlib_modules.push(verified_module);
        stdlib_source_maps.push(source_map)
    }

    (stdlib_modules, stdlib_source_maps)
}
