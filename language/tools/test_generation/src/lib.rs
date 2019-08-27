// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod abstract_state;
pub mod bytecode_generator;
pub mod common;
pub mod control_flow_graph;
pub mod summaries;
pub mod transitions;

#[macro_use]
extern crate mirai_annotations;

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
fn run_verifier(module: CompiledModule) -> u64 {
    match VerifiedModule::new(module) {
        Ok(_) => 1,
        Err((_, errs)) => {
            // error!("Module: {:#?}", module.clone());
            error!("Module verification failed: {:#?}", errs);
            0
        }
    }
}

/// Generate a sequence of bytecode instructions such that
/// - The arguments 'arguments' are used
/// - The return type 'signature' is reached
/// - The number of instructions generated is between 'target_min' and 'target_max'
pub fn generate_bytecode(
    arguments: &[SignatureToken],
    signature: &FunctionSignature,
    target_min: usize,
    target_max: usize,
) -> Vec<Bytecode> {
    let mut bytecode_generator = BytecodeGenerator::new(None);
    bytecode_generator.generate(arguments, signature, target_min, target_max)
}

/// Run generate_bytecode for 'iterations' iterations and test each generated module
/// on the bytecode verifier.
pub fn run_generation(iterations: usize) {
    env_logger::init();

    let mut valid_programs: u64 = 0;
    for _ in 0..iterations {
        let module =
            ModuleBuilder::new(1, Some(Box::new(generate_bytecode))).materialize_unverified();
        valid_programs += run_verifier(module);
    }

    info!(
        "Total valid: {}, Total programs: {}, Percent valid: {:.2}",
        valid_programs,
        iterations,
        (valid_programs as f64) / (iterations as f64) * 100.0
    );
}
