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

use crate::config::{Args, EXECUTE_UNVERIFIED_MODULE, RUN_ON_VM};
use bytecode_generator::BytecodeGenerator;
use bytecode_verifier::VerifiedModule;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use getrandom::getrandom;
use language_e2e_tests::executor::FakeExecutor;
use libra_logger::{debug, error, info};
use libra_state_view::StateView;
use libra_types::{account_address::AccountAddress, vm_error::StatusCode};
use libra_vm::LibraVM;
use move_vm_types::values::Value;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{fs, io::Write, panic, thread};
use utils::module_generation::generate_module;
use vm::{
    access::ModuleAccess,
    errors::VMResult,
    file_format::{CompiledModule, CompiledModuleMut, FunctionDefinitionIndex, SignatureToken},
    transaction_metadata::TransactionMetadata,
};

/// This function calls the Bytecode verifier to test it
fn run_verifier(module: CompiledModule) -> Result<VerifiedModule, String> {
    let verifier_panic = panic::catch_unwind(|| {
        VerifiedModule::new(module.clone())
            .map_err(|(_, errs)| format!("Module verification failed: {:#?}", errs))
    });
    verifier_panic.unwrap_or_else(|err| Err(format!("Verifier panic: {:#?}", err)))
}

/// This function runs a verified module in the VM runtime
fn run_vm(module: VerifiedModule) -> VMResult<()> {
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
            SignatureToken::Address => Value::address(AccountAddress::DEFAULT),
            SignatureToken::U64 => Value::u64(0),
            SignatureToken::Bool => Value::bool(true),
            SignatureToken::Vector(inner_tok) if **inner_tok == SignatureToken::U8 => {
                Value::vector_u8(vec![])
            }
            _ => unimplemented!("Unsupported argument type: {:#?}", sig_tok),
        })
        .collect();

    let executor = FakeExecutor::from_genesis_file();
    execute_function_in_module(executor.get_state_view(), module, entry_idx, main_args)
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
        let mut libra_vm = LibraVM::new();
        libra_vm.load_configs(state_view);

        let internals = libra_vm.internals();
        let move_vm = internals.move_vm();
        move_vm.cache_module(module.clone());

        let gas_schedule = internals.gas_schedule()?;
        let txn_data = TransactionMetadata::default();
        internals.with_txn_context(&txn_data, state_view, |mut txn_context| {
            move_vm.execute_function(
                &module_id,
                &entry_name,
                gas_schedule,
                &mut txn_context,
                &txn_data,
                args,
            )
        })
    }
}

/// Serialize a module to `path` if `output_path` is `Some(path)`. If `output_path` is `None`
/// print the module out as debug output.
fn output_error_case(module: CompiledModule, output_path: Option<String>, case_id: u64, tid: u64) {
    match output_path {
        Some(path) => {
            let mut out = vec![];
            module
                .serialize(&mut out)
                .expect("Unable to serialize module");
            let output_file = format!("{}/case{}_{}.module", path, tid, case_id);
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

fn seed(seed: Option<String>) -> [u8; 32] {
    let mut array = [0u8; 32];
    match seed {
        Some(string) => {
            let vec = hex::decode(string).unwrap();
            if vec.len() != 32 {
                panic!("Invalid seed supplied, the length must be 32.");
            }
            for (i, byte) in vec.into_iter().enumerate() {
                array[i] = byte;
            }
        }
        None => {
            getrandom(&mut array).unwrap();
        }
    };
    array
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    VerificationFailure,
    ExecutionFailure,
    Valid,
}

fn bytecode_module(rng: &mut StdRng, module: CompiledModuleMut) -> CompiledModuleMut {
    let mut generated_module = BytecodeGenerator::new(rng).generate_module(module.clone());
    // Module generation can retry under certain circumstances
    while generated_module.is_none() {
        generated_module = BytecodeGenerator::new(rng).generate_module(module.clone());
    }
    generated_module.unwrap()
}

pub fn module_frame_generation(
    num_iters: Option<u64>,
    seed: [u8; 32],
    sender: Sender<CompiledModuleMut>,
    stats: Receiver<Status>,
) {
    let mut verification_failures: u128 = 0;
    let mut execution_failures: u128 = 0;
    let mut generated: u128 = 1;

    let generation_options = config::module_generation_settings();
    let mut rng = StdRng::from_seed(seed);
    let mut module = generate_module(&mut rng, generation_options.clone()).into_inner();
    // Either get the number of iterations provided by the user, or iterate "infinitely"--up to
    // u128::MAX number of times.
    let iters = num_iters
        .map(|x| x as u128)
        .unwrap_or_else(|| std::u128::MAX);

    while generated < iters && sender.send(module).is_ok() {
        module = generate_module(&mut rng, generation_options.clone()).into_inner();
        generated += 1;
        while let Ok(stat) = stats.try_recv() {
            match stat {
                Status::VerificationFailure => verification_failures += 1,
                Status::ExecutionFailure => execution_failures += 1,
                _ => (),
            };
        }

        if generated > 0 && generated % 100 == 0 {
            info!(
                "Generated: {} Verified: {} Executed: {}",
                generated,
                (generated - verification_failures),
                (generated - execution_failures)
            );
        }
    }

    // Drop the sender channel to signal to the consumers that they should expect no more modules,
    // and should finish up.
    drop(sender);

    // Gather final stats from the consumers.
    while let Ok(stat) = stats.recv() {
        match stat {
            Status::VerificationFailure => verification_failures += 1,
            Status::ExecutionFailure => execution_failures += 1,
            _ => (),
        };
    }
    info!(
        "Final stats: Generated: {} Verified: {} Executed: {}",
        generated,
        (generated - verification_failures),
        (generated - execution_failures)
    );
}

pub fn bytecode_generation(
    output_path: Option<String>,
    tid: u64,
    mut rng: StdRng,
    receiver: Receiver<CompiledModuleMut>,
    stats: Sender<Status>,
) {
    while let Ok(module) = receiver.recv() {
        let mut status = Status::VerificationFailure;
        debug!("Generating module");
        let module = bytecode_module(&mut rng, module);

        debug!("Done...Running module on verifier...");
        let module = module.freeze().expect("generated module failed to freeze.");
        let verified_module = match run_verifier(module.clone()) {
            Ok(verified_module) => {
                status = Status::ExecutionFailure;
                Some(verified_module)
            }
            Err(e) => {
                error!("{}", e);
                let uid = rng.gen::<u64>();
                output_error_case(module.clone(), output_path.clone(), uid, tid);
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
                debug!("Done...Running module on VM...");
                let execution_result = panic::catch_unwind(|| run_vm(verified_module));
                match execution_result {
                    Ok(execution_result) => match execution_result {
                        Ok(_) => {
                            status = Status::Valid;
                        }
                        Err(e) => match e.major_status {
                            StatusCode::ARITHMETIC_ERROR | StatusCode::OUT_OF_GAS => {
                                status = Status::Valid;
                            }
                            _ => {
                                error!("{}", e);
                                let uid = rng.gen::<u64>();
                                output_error_case(module.clone(), output_path.clone(), uid, tid);
                            }
                        },
                    },
                    Err(_) => {
                        // Save modules that cause the VM runtime to panic
                        let uid = rng.gen::<u64>();
                        output_error_case(module.clone(), output_path.clone(), uid, tid);
                    }
                }
            } else {
                status = Status::Valid;
            }
        };
        stats.send(status).unwrap();
    }

    drop(stats);
}

/// Run generate_bytecode for the range passed in and test each generated module
/// on the bytecode verifier.
pub fn run_generation(args: Args) {
    let num_threads = if let Some(num_threads) = args.num_threads {
        num_threads as usize
    } else {
        num_cpus::get()
    };
    assert!(
        num_threads > 0,
        "Number of worker threads must be greater than 0"
    );

    let (sender, receiver) = bounded(num_threads);
    let (stats_sender, stats_reciever) = unbounded();
    let seed = seed(args.seed);

    let mut threads = Vec::new();
    for tid in 0..num_threads {
        let receiver = receiver.clone();
        let stats_sender = stats_sender.clone();
        let rng = StdRng::from_seed(seed);
        let output_path = args.output_path.clone();
        threads.push(thread::spawn(move || {
            bytecode_generation(output_path, tid as u64, rng, receiver, stats_sender)
        }));
    }

    // Need to drop this channel otherwise we'll get infinite blocking since the other channels are
    // cloned; this one will remain open unless we close it and other threads are going to block
    // waiting for more stats.
    drop(stats_sender);

    let num_iters = args.num_iterations;
    threads.push(thread::spawn(move || {
        module_frame_generation(num_iters, seed, sender, stats_reciever)
    }));

    for thread in threads {
        thread.join().unwrap();
    }
}
