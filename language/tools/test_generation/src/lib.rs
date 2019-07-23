// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod abstract_state;
pub mod bytecode_generator;
pub mod summaries;
pub mod transitions;

#[macro_use]
extern crate log;
extern crate env_logger;
use bytecode_generator::BytecodeGenerator;
use bytecode_verifier::VerifiedModule;
use cost_synthesis::module_generator::ModuleBuilder;
use vm::{
    file_format::{Bytecode, FunctionSignature, SignatureToken},
    CompiledModule,
};

/// This function calls the Bytecode verifier to test it
fn run_verifier(module: CompiledModule) {
    match VerifiedModule::new(module) {
        Ok(_) => {}
        Err((_, errs)) => {
            error!("Module verification failed: {:#?}", errs);
        }
    }
}

/// Generate a sequence of bytecode instructions such that
/// - The arguments 'arguments' are used
/// - The return type '_signature' is reached
/// - The number of instructions generated is between 'target_min' and 'target_max'
pub fn generate_bytecode(
    arguments: &[SignatureToken],
    _signature: &FunctionSignature,
    target_min: usize,
    target_max: usize,
) -> Vec<Bytecode> {
    let seed: [u8; 32] = [0; 32];
    let mut bytecode_generator = BytecodeGenerator::new(seed);
    bytecode_generator.generate(arguments, _signature, target_min, target_max)
}

/// Run generate_bytecode for 'iterations' iterations and test each generated module
/// on the bytecode verifier.
pub fn run_generation(iterations: usize) {
    env_logger::init();

    for _ in 0..iterations {
        let module =
            ModuleBuilder::new(1, Some(Box::new(generate_bytecode))).materialize_unverified();
        println!("Module: {:#?}", module);
        run_verifier(module);
    }
}
