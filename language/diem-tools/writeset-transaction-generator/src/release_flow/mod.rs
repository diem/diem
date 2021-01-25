// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod unit_tests;

pub mod artifacts;
mod create;
mod verify;

pub use artifacts::{hash_for_modules, load_artifact, save_release_artifact, ReleaseArtifact};
pub use create::create_release;
pub use verify::verify_release;

pub mod test_utils {
    use compiled_stdlib::{stdlib_modules, StdLibOptions};
    use diem_types::account_config::CORE_CODE_ADDRESS;
    use vm::{file_format::empty_module, CompiledModule};

    pub fn release_modules() -> Vec<CompiledModule> {
        let mut modules = stdlib_modules(StdLibOptions::Compiled).to_vec();
        // Publish a new dummy module
        let mut module = empty_module();
        module.address_identifiers[0] = CORE_CODE_ADDRESS;

        modules.push(module.freeze().unwrap());
        // TODO: See if there's a module that we can remove and if we can modify an existing module
        modules
    }
}
