// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod abstract_state;
pub mod bytecode_generator;
pub mod config;
pub mod control_flow_graph;
pub mod error;
pub mod summaries;
pub mod transitions;

#[macro_use]
extern crate mirai_annotations;

#[macro_use]
extern crate log;
extern crate env_logger;
use crate::config::{EXECUTE_UNVERIFIED_MODULE, GAS_METERING, RUN_ON_VM};
use bytecode_generator::BytecodeGenerator;
use bytecode_verifier::VerifiedModule;
use cost_synthesis::module_generator::ModuleBuilder;
use language_e2e_tests::data_store::FakeDataStore;
use std::panic;
use types::{account_address::AccountAddress, byte_array::ByteArray};
use vm::{
    file_format::{
        Bytecode, CompiledModuleMut, FunctionDefinitionIndex, FunctionSignature, SignatureToken,
    },
    transaction_metadata::TransactionMetadata,
    CompiledModule,
};
use vm_cache_map::Arena;
use vm_runtime::{
    code_cache::module_cache::{ModuleCache, VMModuleCache},
    loaded_data::function::{FunctionRef, FunctionReference},
    txn_executor::TransactionExecutor,
};
use vm_runtime_types::value::Value;

/// This function calls the Bytecode verifier to test it
fn run_verifier(module: CompiledModule) -> Result<VerifiedModule, String> {
    let verifier_panic = panic::catch_unwind(|| {
        match VerifiedModule::new(module.clone()) {
            Ok(verified_module) => Ok(verified_module),
            Err((_, errs)) => {
                // info!("Generated module: {:#?}", module);
                Err(format!("Module verification failed: {:#?}", errs))
            }
        }
    });
    verifier_panic.unwrap_or_else(|err| Err(format!("Verifier panic: {:#?}", err)))
}

/// This function runs a verified module in the VM runtime
/// This code is based on `cost_synthesis/src/vm_runner.rs`
fn run_vm(module: VerifiedModule) -> Result<(), String> {
    use vm::access::ModuleAccess;
    let mut modules = ::stdlib::stdlib_modules().to_vec();
    // The standard library modules are bounded
    assume!(modules.len() < usize::max_value());
    modules.push(module);
    let root_module = modules
        .last()
        .expect("[VM Setup] Unable to get root module");
    let allocator = Arena::new();
    let module_id = root_module.self_id();
    let module_cache = VMModuleCache::new(&allocator);
    let entry_idx = FunctionDefinitionIndex::new(0);
    let data_cache = FakeDataStore::default();
    module_cache.cache_module(root_module.clone());
    let loaded_module = module_cache
        .get_loaded_module(&module_id)
        .expect("[Module Lookup] Invariant violation while looking up module")
        .expect("[Module Lookup] Runtime error while looking up module");
    for m in modules.clone() {
        module_cache.cache_module(m);
    }
    let mut vm =
        TransactionExecutor::new(&module_cache, &data_cache, TransactionMetadata::default());
    let entry_func = FunctionRef::new(&loaded_module, entry_idx);
    let mut function_args: Vec<Value> = Vec::new();
    for arg_type in entry_func.signature().arg_types.clone() {
        function_args.push(match arg_type {
            SignatureToken::Address => Value::address(AccountAddress::new([0; 32])),
            SignatureToken::U64 => Value::u64(0),
            SignatureToken::Bool => Value::bool(true),
            SignatureToken::String => Value::string("".into()),
            SignatureToken::ByteArray => Value::byte_array(ByteArray::new(vec![])),
            _ => unimplemented!("Unsupported argument type: {:#?}", arg_type),
        });
    }
    if !GAS_METERING {
        vm.turn_off_gas_metering();
    }
    match vm.execute_function(&module_id, &entry_func.name(), function_args) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Runtime error: {:?}", e)),
    }
}

/// Generate a sequence of bytecode instructions such that
/// - The arguments 'arguments' are used
/// - The return type 'signature' is reached
/// - The number of instructions generated is between 'target_min' and 'target_max'
pub fn generate_bytecode(
    arguments: &[SignatureToken],
    signature: &FunctionSignature,
    module: CompiledModuleMut,
) -> Vec<Bytecode> {
    let mut bytecode_generator = BytecodeGenerator::new(None);
    bytecode_generator.generate(arguments, signature, module)
}

/// Run generate_bytecode for 'iterations' iterations and test each generated module
/// on the bytecode verifier.
pub fn run_generation(iterations: u64) {
    env_logger::init();

    let mut verified_programs: u64 = 0;
    let mut executed_programs: u64 = 0;
    for i in 0..iterations {
        let module =
            ModuleBuilder::new(1, Some(Box::new(generate_bytecode))).materialize_unverified();
        debug!("Running on verifier...");
        let verified_module = match run_verifier(module.clone()) {
            Ok(verified_module) => {
                // We cannot execute more than u64::max_value() iterations.
                verify!(verified_programs < u64::max_value());
                verified_programs += 1;
                Some(verified_module)
            }
            Err(e) => {
                error!("{}", e);
                if EXECUTE_UNVERIFIED_MODULE {
                    Some(VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(
                        module.clone(),
                    ))
                } else {
                    None
                }
            }
        };
        if let Some(verified_module) = verified_module {
            if RUN_ON_VM {
                debug!("Running on VM...");
                match run_vm(verified_module) {
                    Ok(_) => {
                        // We cannot execute more than u64::max_value() iterations.
                        verify!(executed_programs < u64::max_value());
                        executed_programs += 1
                    }
                    Err(e) => error!("{}", e),
                }
            }
        };
        if iterations > 10 && i % (iterations / 10) == 0 {
            info!("Iteration: {} / {}", i, iterations);
        }
    }

    info!(
        "Total programs: {}, Percent valid: {:.2}, Percent executed {:.2}",
        iterations,
        (verified_programs as f64) / (iterations as f64) * 100.0,
        (executed_programs as f64) / (iterations as f64) * 100.0,
    );
}
