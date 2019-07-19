// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod stdlib;
pub mod transaction_scripts;

use bytecode_verifier::{verify_module_dependencies, VerifiedModule};
use ir_to_bytecode::compiler::compile_module;
use lazy_static::lazy_static;
use types::{account_address::AccountAddress, account_config};

lazy_static! {
    static ref STDLIB_MODULES: Vec<VerifiedModule> =
        { build_stdlib(&account_config::core_code_address()) };
}

/// Returns a reference to the standard library, compiled with the
/// [default address](account_config::core_code_address).
///
/// The order the modules are presented in is important: later modules depend on earlier ones.
pub fn stdlib_modules() -> &'static [VerifiedModule] {
    &*STDLIB_MODULES
}

/// Builds and returns a copy of the standard library with this address as the self address.
///
/// A copy of the stdlib built with the [default address](account_config::core_code_address) is
/// available through [`stdlib_modules`].
pub fn build_stdlib(address: &AccountAddress) -> Vec<VerifiedModule> {
    let mut stdlib_modules = vec![];

    for module_def in stdlib::module_defs() {
        let compiled_module = compile_module(address, *module_def, &stdlib_modules)
            .expect("stdlib module failed to compile");
        let verified_module =
            VerifiedModule::new(compiled_module).expect("stdlib module failed to verify");

        let (verified_module, verification_errors) =
            verify_module_dependencies(verified_module, &stdlib_modules);
        // Fail if the module doesn't verify
        for e in &verification_errors {
            println!("{:?}", e);
        }
        assert!(verification_errors.is_empty());

        stdlib_modules.push(verified_module);
    }

    stdlib_modules
}
