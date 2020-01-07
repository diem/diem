// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

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
use crate::config::{Args, EXECUTE_UNVERIFIED_MODULE, RUN_ON_VM};
use bytecode_generator::BytecodeGenerator;
use bytecode_verifier::VerifiedModule;
use language_e2e_tests::executor::FakeExecutor;
use libra_state_view::StateView;
use libra_types::{account_address::AccountAddress, byte_array::ByteArray};
use std::{fs, io::Write, panic};
use utils::module_generation::{generate_module, ModuleGeneratorOptions};
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{CompiledModule, FunctionDefinitionIndex, SignatureToken},
    gas_schedule::MAXIMUM_NUMBER_OF_GAS_UNITS,
    transaction_metadata::TransactionMetadata,
};
use vm_cache_map::Arena;
use vm_runtime::chain_state::TransactionExecutionContext;
use vm_runtime::{
    data_cache::BlockDataCache, runtime::VMRuntime, txn_executor::TransactionExecutor,
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
fn run_vm(module: VerifiedModule) -> Result<(), String> {
    // By convention the 0'th index function definition is the entrypoint to the module (i.e. that
    // will contain only simply-typed arguments).
    let entry_idx = FunctionDefinitionIndex::new(0);
    let function_signature = {
        let handle = module.function_def_at(entry_idx).function;
        let sig_idx = module.function_handle_at(handle).signature;
        module.function_signature_at(sig_idx).clone()
    };
    let main_args: Vec<Value> = function_signature
        .arg_types
        .iter()
        .map(|sig_tok| match sig_tok {
            SignatureToken::Address => Value::address(AccountAddress::new([0; 32])),
            SignatureToken::U64 => Value::u64(0),
            SignatureToken::Bool => Value::bool(true),
            SignatureToken::ByteArray => Value::byte_array(ByteArray::new(vec![])),
            _ => unimplemented!("Unsupported argument type: {:#?}", sig_tok),
        })
        .collect();

    let executor = FakeExecutor::from_genesis_file();
    execute_function_in_module(executor.get_state_view(), module, entry_idx, main_args)
        .map_err(|err| format!("Runtime error: {:?}", err))?;
    Ok(())
}

/// Execute the first function in a module
fn execute_function_in_module(
    state_view: &dyn StateView,
    module: VerifiedModule,
    idx: FunctionDefinitionIndex,
    args: Vec<Value>,
) -> VMResult<()> {
    let module_id = module.as_inner().self_id();
    let entry_name = {
        let entry_func_idx = module.function_def_at(idx).function;
        let entry_name_idx = module.function_handle_at(entry_func_idx).name;
        module.identifier_at(entry_name_idx)
    };
    {
        let arena = Arena::new();
        let mut runtime = VMRuntime::new(&arena);
        runtime.cache_module(module.clone());

        let data_cache = BlockDataCache::new(state_view);
        let mut context =
            TransactionExecutionContext::new(*MAXIMUM_NUMBER_OF_GAS_UNITS, &data_cache);
        let gas_schedule = runtime.load_gas_schedule(&mut context, &data_cache)?;
        let mut txn_executor =
            TransactionExecutor::new(&gas_schedule, &data_cache, TransactionMetadata::default());
        txn_executor.execute_function(&runtime, &module_id, &entry_name, args)
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

/// Run generate_bytecode for 'iterations' iterations and test each generated module
/// on the bytecode verifier.
pub fn run_generation(args: Args) {
    env_logger::init();
    let iterations = args.num_iterations;
    let mut verified_programs: u64 = 0;
    let mut executed_programs: u64 = 0;
    let mut generation_options = ModuleGeneratorOptions::default();
    generation_options.min_table_size = 10;
    // No type parameters for now
    generation_options.max_ty_params = 1;
    generation_options.max_functions = 10;
    generation_options.max_structs = 10;
    // Test generation cannot currently handle non-simple types (nested structs, and references)
    generation_options.simple_types_only = true;
    // Test generation cannot currently cope with resources
    generation_options.add_resources = false;
    for i in 0..iterations {
        let mut module = generate_module(generation_options.clone()).into_inner();
        BytecodeGenerator::new(None).generate_module(&mut module);
        debug!("Running on verifier...");
        let module = module.freeze().expect("generated module failed to freeze.");
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
