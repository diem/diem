// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::VerifiedModule;
use move_core_types::fs::AFS;
use rand::{rngs::StdRng, SeedableRng};
use utils::module_generation::{generate_module, ModuleGeneratorOptions};

#[test]
fn module_generation() {
    let mut rng = StdRng::from_entropy();
    let _fs = AFS::new();

    for _ in 0..50 {
        let module = generate_module(&mut rng, ModuleGeneratorOptions::default());
        VerifiedModule::new(module).unwrap();
    }
}
