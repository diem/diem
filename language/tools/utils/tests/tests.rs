// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::VerifiedModule;
use utils::module_generation::{generate_module, ModuleGeneratorOptions};

#[test]
fn module_generation() {
    for _ in 0..500 {
        let module = generate_module(None, ModuleGeneratorOptions::default());
        VerifiedModule::new(module).unwrap();
    }
}
