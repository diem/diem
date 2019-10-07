// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod abstract_state;
pub mod borrow_graph;
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
use crate::config::{Args, EXECUTE_UNVERIFIED_MODULE, GAS_METERING, RUN_ON_VM};
use bytecode_generator::BytecodeGenerator;
use bytecode_verifier::VerifiedModule;
use cost_synthesis::module_generator::ModuleBuilder;
use language_e2e_tests::data_store::FakeDataStore;
use libra_types::{account_address::AccountAddress, byte_array::ByteArray};
use std::{fs, io::Write, panic};
use vm::{
    file_format::{
        Bytecode, CompiledModuleMut, FunctionDefinitionIndex, FunctionSignature, SignatureToken,
        StructDefinitionIndex,
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
        VerifiedModule::new(module.clone())
            .map_err(|(_, errs)| format!("Module verification failed: {:#?}", errs))
    });
    verifier_panic.unwrap_or_else(|err| Err(format!("Verifier panic: {:#?}", err)))
}

/// This function runs a verified module in the VM runtime
/// This code is based on `cost-synthesis/src/vm_runner.rs`
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

/// Serialize a module to `path` if `output_path` is `Some(path)`. If `output_path` is `None`
/// print the module out as debug output.
fn output_error_case(module: CompiledModule, output_path: Option<String>, iteration: u64) {
    match output_path {
        Some(path) => {
            let mut out = vec![];
            module
                .serialize(&mut out)
                .expect("Unable to serialize module");
            let output_file = format!("{}/case{}.module", path, iteration);
            let mut f = fs::File::create(&output_file)
                .unwrap_or_else(|err| panic!("Unable to open output file {}: {}", &path, err));
            f.write_all(&out)
                .unwrap_or_else(|err| panic!("Unable to write to output file {}: {}", &path, err));
        }
        None => {
            debug!("{:#?}", module);
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
    acquires_global_resources: &[StructDefinitionIndex],
    module: CompiledModuleMut,
) -> Vec<Bytecode> {
    let mut bytecode_generator = BytecodeGenerator::new(None);
    bytecode_generator.generate(arguments, signature, acquires_global_resources, module)
}

/// Run generate_bytecode for 'iterations' iterations and test each generated module
/// on the bytecode verifier.
pub fn run_generation(args: Args) {
    env_logger::init();
    let iterations = args.num_iterations;
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
                output_error_case(module.clone(), args.output_path.clone(), i);
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
                let execution_result = panic::catch_unwind(|| run_vm(verified_module));
                match execution_result {
                    Ok(execution_result) => {
                        match execution_result {
                            Ok(_) => {
                                // We cannot execute more than u64::max_value() iterations.
                                verify!(executed_programs < u64::max_value());
                                executed_programs += 1
                            }
                            Err(e) => {
                                // TODO: Uncomment this to allow saving of modules that fail
                                // the VM runtime.
                                // output_error_case(module.clone(), args.output_path.clone(), i);
                                error!("{}", e)
                            }
                        }
                    }
                    Err(_) => {
                        // Save modules that cause the VM runtime to panic
                        output_error_case(module.clone(), args.output_path.clone(), i);
                    }
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
