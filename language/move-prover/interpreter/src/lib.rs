// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use structopt::StructOpt;

use bytecode::function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder};
use move_binary_format::errors::VMResult;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::MoveValue,
};
use move_model::model::{FunctionEnv, GlobalEnv};

mod concrete;
mod shared;

use crate::concrete::{
    runtime::Runtime,
    value::{GlobalState, TypedValue},
};

/// Options passed into the interpreter generator.
#[derive(StructOpt)]
pub struct InterpreterOptions {
    /// The function to be executed, specified in the format of `addr::module_name::function_name`
    #[structopt(long = "entry", parse(try_from_str = parse_entrypoint))]
    entrypoint: (ModuleId, Identifier),

    /// Possibly-empty list of signers for the execution
    #[structopt(long = "signers", parse(try_from_str = AccountAddress::from_hex_literal))]
    signers: Vec<AccountAddress>,
    /// Possibly-empty list of arguments passed to the transaction
    #[structopt(long = "args", parse(try_from_str = parse_transaction_argument))]
    args: Vec<TransactionArgument>,
    /// Possibly-empty list of type arguments passed to the transaction (e.g., `T` in
    /// `main<T>()`). Must match the type arguments kinds expected by `script_file`.
    #[structopt(long = "type-args", parse(try_from_str = parse_type_tag))]
    type_args: Vec<TypeTag>,

    /// Run the interpreter at every step of the transformation pipeline
    #[structopt(long = "stepwise")]
    stepwise: bool,
    /// Output directory that holds the debug information of produced during interpretation
    #[structopt(short = "o", long = "output")]
    output: Option<String>,
}

fn parse_entrypoint(input: &str) -> Result<(ModuleId, Identifier)> {
    let tokens: Vec<_> = input.split("::").collect();
    if tokens.len() != 3 {
        bail!("Invalid entrypoint: {}", input);
    }
    let module_id = ModuleId::new(
        AccountAddress::from_hex_literal(tokens[0])?,
        Identifier::new(tokens[1])?,
    );
    let func_name = Identifier::new(tokens[2])?;
    Ok((module_id, func_name))
}

//**************************************************************************************************
// Entry
//**************************************************************************************************
pub fn interpret(
    options: &InterpreterOptions,
    env: &GlobalEnv,
    pipeline: FunctionTargetPipeline,
) -> Result<()> {
    // find the entrypoint
    let entrypoint_env = env
        .find_module_by_language_storage_id(&options.entrypoint.0)
        .and_then(|module_env| {
            module_env.find_function(env.symbol_pool().make(options.entrypoint.1.as_str()))
        })
        .ok_or_else(|| {
            anyhow!(
                "Unable to find entrypoint: {}::{}",
                options.entrypoint.0,
                options.entrypoint.1
            )
        })?;

    // consolidate arguments
    let args: Vec<_> = options
        .signers
        .iter()
        .map(|addr| MoveValue::Signer(*addr))
        .chain(options.args.iter().map(|arg| match arg {
            TransactionArgument::Bool(v) => MoveValue::Bool(*v),
            TransactionArgument::U8(v) => MoveValue::U8(*v),
            TransactionArgument::U64(v) => MoveValue::U64(*v),
            TransactionArgument::U128(v) => MoveValue::U128(*v),
            TransactionArgument::Address(v) => MoveValue::Address(*v),
            TransactionArgument::U8Vector(v) => {
                MoveValue::Vector(v.iter().map(|e| MoveValue::U8(*e)).collect())
            }
        }))
        .collect();

    // prepare workdir
    let workdir_opt = match &options.output {
        None => None,
        Some(workdir) => {
            fs::create_dir_all(workdir)?;
            Some(workdir)
        }
    };

    // collect function targets
    let mut targets = FunctionTargetsHolder::default();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // run through the pipeline
    if options.stepwise {
        pipeline.run_with_hook(
            env,
            &mut targets,
            |holder| {
                stepwise_processing(
                    env,
                    workdir_opt,
                    0,
                    "stackless",
                    holder,
                    &entrypoint_env,
                    &options.type_args,
                    &args,
                )
            },
            |step, processor, holders| {
                stepwise_processing(
                    env,
                    workdir_opt,
                    step,
                    &processor.name(),
                    holders,
                    &entrypoint_env,
                    &options.type_args,
                    &args,
                )
            },
        );
    } else {
        pipeline.run(env, &mut targets);
        stepwise_processing(
            env,
            workdir_opt,
            0,
            "final",
            &targets,
            &entrypoint_env,
            &options.type_args,
            &args,
        );
    }
    Ok(())
}

fn stepwise_processing<P: AsRef<Path>>(
    env: &GlobalEnv,
    workdir_opt: Option<P>,
    step: usize,
    name: &str,
    targets: &FunctionTargetsHolder,
    fun_env: &FunctionEnv,
    ty_args: &[TypeTag],
    args: &[MoveValue],
) {
    match stepwise_processing_internal(
        env,
        workdir_opt,
        step,
        name,
        targets,
        fun_env,
        ty_args,
        args,
    ) {
        Ok(_) => (),
        Err(e) => panic!("Unexpected error during step {} - {}: {}", step, name, e),
    }
}

fn stepwise_processing_internal<P: AsRef<Path>>(
    env: &GlobalEnv,
    workdir_opt: Option<P>,
    step: usize,
    name: &str,
    targets: &FunctionTargetsHolder,
    fun_env: &FunctionEnv,
    ty_args: &[TypeTag],
    args: &[MoveValue],
) -> Result<()> {
    // short-circuit the execution if prior phases run into errors
    if env.has_errors() {
        return Ok(());
    }
    let filebase_opt =
        workdir_opt.map(|workdir| workdir.as_ref().join(format!("{}_{}", step, name)));

    // dump the bytecode if requested
    if let Some(filebase) = &filebase_opt {
        let mut text = String::new();
        for module_env in env.get_modules() {
            for func_env in module_env.get_functions() {
                for (variant, target) in targets.get_targets(&func_env) {
                    if !target.data.code.is_empty() {
                        target.register_annotation_formatters_for_test();
                        text += &format!("\n[variant {}]\n{}\n", variant, target);
                    }
                }
            }
        }
        let mut file = File::create(filebase.with_extension("bytecode"))?;
        file.write_all(text.as_bytes())?;
    }

    // invoke the runtime
    let vm = Runtime::new(env, targets);
    let global_state = GlobalState::default();
    let (result, updated_global_state) = vm.execute(fun_env, ty_args, args, global_state);
    consume_execution_result(step, name, result, updated_global_state);
    Ok(())
}

fn consume_execution_result(
    step: usize,
    name: &str,
    result: VMResult<Vec<TypedValue>>,
    _global_state: GlobalState,
) {
    println!("Execution results for step {}: {}", step, name);
    match result {
        Ok(_) => println!("Success"),
        Err(e) => println!("Aborted with error: {}", e),
    }
}
