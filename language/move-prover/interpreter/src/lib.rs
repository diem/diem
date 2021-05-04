// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
use structopt::StructOpt;

use bytecode::function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder};
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::{MoveStruct, MoveValue},
    vm_status::StatusCode,
};
use move_model::model::{FunctionEnv, GlobalEnv};

mod concrete;
mod shared;

use crate::concrete::{
    runtime::Runtime,
    ty::{BaseType, IntType, PrimitiveType},
    value::{BaseValue, GlobalState, TypedValue},
};

/// Options passed into the interpreter generator.
#[derive(StructOpt)]
pub struct InterpreterOptions {
    /// The function to be executed, specified in the format of `addr::module_name::function_name`
    #[structopt(long = "entry", parse(try_from_str = parse_entrypoint))]
    pub entrypoint: (ModuleId, Identifier),

    /// Possibly-empty list of signers for the execution
    #[structopt(long = "signers", parse(try_from_str = AccountAddress::from_hex_literal))]
    pub signers: Vec<AccountAddress>,
    /// Possibly-empty list of arguments passed to the transaction
    #[structopt(long = "args", parse(try_from_str = parse_transaction_argument))]
    pub args: Vec<TransactionArgument>,
    /// Possibly-empty list of type arguments passed to the transaction (e.g., `T` in
    /// `main<T>()`). Must match the type arguments kinds expected by `script_file`.
    #[structopt(long = "ty-args", parse(try_from_str = parse_type_tag))]
    pub ty_args: Vec<TypeTag>,

    /// Run the interpreter at every step of the transformation pipeline
    #[structopt(long = "stepwise")]
    pub stepwise: bool,
    /// Output directory that holds the debug information of produced during interpretation
    #[structopt(short = "o", long = "output")]
    pub output: Option<PathBuf>,
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

#[derive(Eq, PartialEq)]
struct ExecutionResult {
    vm_result: VMResult<Vec<TypedValue>>,
    global_state: GlobalState,
}

//**************************************************************************************************
// Entry
//**************************************************************************************************
pub fn interpret_with_options(
    options: InterpreterOptions,
    env: &GlobalEnv,
    pipeline: FunctionTargetPipeline,
) -> VMResult<Vec<Vec<u8>>> {
    // consolidate arguments
    let args: Vec<_> = options
        .signers
        .into_iter()
        .map(MoveValue::Signer)
        .chain(options.args.into_iter().map(|arg| match arg {
            TransactionArgument::Bool(v) => MoveValue::Bool(v),
            TransactionArgument::U8(v) => MoveValue::U8(v),
            TransactionArgument::U64(v) => MoveValue::U64(v),
            TransactionArgument::U128(v) => MoveValue::U128(v),
            TransactionArgument::Address(v) => MoveValue::Address(v),
            TransactionArgument::U8Vector(v) => {
                MoveValue::Vector(v.into_iter().map(MoveValue::U8).collect())
            }
        }))
        .collect();

    // run the actual interpreter
    interpret(
        env,
        &options.entrypoint.0,
        &options.entrypoint.1,
        &options.ty_args,
        &args,
        pipeline,
        options.output,
        options.stepwise,
    )
}

pub fn interpret(
    env: &GlobalEnv,
    module_id: &ModuleId,
    func_name: &IdentStr,
    ty_args: &[TypeTag],
    args: &[MoveValue],
    pipeline: FunctionTargetPipeline,
    output_opt: Option<PathBuf>,
    stepwise: bool,
) -> VMResult<Vec<Vec<u8>>> {
    // find the entrypoint
    let entrypoint_env = env
        .find_module_by_language_storage_id(module_id)
        .and_then(|module_env| module_env.find_function(env.symbol_pool().make(func_name.as_str())))
        .ok_or_else(|| {
            PartialVMError::new(StatusCode::LOOKUP_FAILED).finish(Location::Undefined)
        })?;

    // collect function targets
    let mut targets = FunctionTargetsHolder::default();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }

    // prepare workdir
    let workdir_opt = output_opt.map(|workdir| {
        fs::create_dir_all(&workdir).unwrap();
        workdir
    });

    // run through the pipeline
    if stepwise {
        pipeline.run_with_hook(
            env,
            &mut targets,
            |holder| {
                stepwise_processing(
                    env,
                    workdir_opt.as_ref(),
                    0,
                    "stackless",
                    holder,
                    &entrypoint_env,
                    ty_args,
                    args,
                )
            },
            |step, processor, holders| {
                stepwise_processing(
                    env,
                    workdir_opt.as_ref(),
                    step,
                    &processor.name(),
                    holders,
                    &entrypoint_env,
                    ty_args,
                    args,
                )
            },
        );
    } else {
        pipeline.run(env, &mut targets);
        stepwise_processing(
            env,
            workdir_opt.as_ref(),
            0,
            "final",
            &targets,
            &entrypoint_env,
            ty_args,
            args,
        );
    }

    // collect and convert results
    let result = env.get_extension::<ExecutionResult>().unwrap();
    result.vm_result.clone().map(|rets| {
        rets.into_iter()
            .map(|v| {
                let (ty, val, _) = v.decompose();
                let move_val = convert_typed_value_to_move_value(ty.get_base_type(), val);
                move_val.simple_serialize().unwrap()
            })
            .collect::<Vec<_>>()
    })
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
    let (vm_result, updated_global_state) = vm.execute(fun_env, ty_args, args, global_state);
    let actual_result = ExecutionResult {
        vm_result,
        global_state: updated_global_state,
    };

    // handle the results
    if env.has_extension::<ExecutionResult>() {
        let expect_result = env.get_extension::<ExecutionResult>().unwrap();
        if expect_result.as_ref() != &actual_result {
            bail!(
                "Execution result in step {}: {} does not match with prior results",
                step,
                name
            );
        }
    } else {
        env.set_extension(actual_result);
    }
    Ok(())
}

fn convert_typed_value_to_move_value(ty: &BaseType, val: BaseValue) -> MoveValue {
    match ty {
        BaseType::Primitive(PrimitiveType::Bool) => MoveValue::Bool(val.into_bool()),
        BaseType::Primitive(PrimitiveType::Int(IntType::U8)) => MoveValue::U8(val.into_u8()),
        BaseType::Primitive(PrimitiveType::Int(IntType::U64)) => MoveValue::U64(val.into_u64()),
        BaseType::Primitive(PrimitiveType::Int(IntType::U128)) => MoveValue::U128(val.into_u128()),
        BaseType::Primitive(PrimitiveType::Int(IntType::Num)) => unreachable!(),
        BaseType::Primitive(PrimitiveType::Address) => MoveValue::Address(val.into_address()),
        BaseType::Primitive(PrimitiveType::Signer) => MoveValue::Signer(val.into_signer()),
        BaseType::Vector(elem) => MoveValue::Vector(
            val.into_vector()
                .into_iter()
                .map(|e| convert_typed_value_to_move_value(elem, e))
                .collect(),
        ),
        BaseType::Struct(inst) => MoveValue::Struct(MoveStruct::new(
            val.into_struct()
                .into_iter()
                .zip(inst.fields.iter())
                .map(|(field_val, field_info)| {
                    convert_typed_value_to_move_value(&field_info.ty, field_val)
                })
                .collect(),
        )),
    }
}
