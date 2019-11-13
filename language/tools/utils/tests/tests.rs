// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bytecode_verifier::VerifiedModule;
use ir_to_bytecode::compiler::compile_module;
use libra_types::account_address::AccountAddress;
use utils::module_generator::*;
use vm::file_format::CompiledModule;

#[test]
fn module_generation() {
    for _ in 0..500 {
        let m = ModuleGenerator::create(&Set::new());
        let deps: Vec<CompiledModule> = Vec::new();
        let (compiled, _) = compile_module(AccountAddress::default(), m, &deps).unwrap();
        VerifiedModule::new(compiled).unwrap();
    }
}
