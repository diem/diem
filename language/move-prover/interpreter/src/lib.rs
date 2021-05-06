// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use structopt::StructOpt;

use bytecode::{
    function_target_pipeline::{FunctionTargetPipeline, FunctionTargetsHolder},
    options::ProverOptions,
    pipeline_factory::default_pipeline_with_options,
};
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

    /// Print verbose debug information
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,
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

#[derive(Debug, Eq, PartialEq)]
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
    interpret_with_default_pipeline(
        env,
        &options.entrypoint.0,
        &options.entrypoint.1,
        &options.ty_args,
        &args,
        options.verbose,
    )
}

pub fn interpret_with_default_pipeline(
    env: &GlobalEnv,
    module_id: &ModuleId,
    func_name: &IdentStr,
    ty_args: &[TypeTag],
    args: &[MoveValue],
    verbose: bool,
) -> VMResult<Vec<Vec<u8>>> {
    let options = ProverOptions {
        for_interpretation: true,
        ..Default::default()
    };
    let pipeline = default_pipeline_with_options(&options);
    env.set_extension(options);
    interpret(env, module_id, func_name, ty_args, args, pipeline, verbose)
}

pub fn interpret(
    env: &GlobalEnv,
    module_id: &ModuleId,
    func_name: &IdentStr,
    ty_args: &[TypeTag],
    args: &[MoveValue],
    pipeline: FunctionTargetPipeline,
    verbose: bool,
) -> VMResult<Vec<Vec<u8>>> {
    // find the entrypoint
    let entrypoint_env = env
        .find_module_by_language_storage_id(module_id)
        .and_then(|module_env| module_env.find_function(env.symbol_pool().make(func_name.as_str())))
        .ok_or_else(|| {
            PartialVMError::new(StatusCode::LOOKUP_FAILED).finish(Location::Undefined)
        })?;
    if verbose {
        println!(
            "[compiled module]\n{:?}\n",
            entrypoint_env.module_env.get_verified_module()
        );
    }

    // collect and transform function targets
    let mut targets = FunctionTargetsHolder::default();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            targets.add_target(&func_env)
        }
    }
    if verbose {
        pipeline.run_with_hook(
            env,
            &mut targets,
            |holder| verbose_stepwise_processing(env, 0, "stackless", holder),
            |step, processor, holders| {
                verbose_stepwise_processing(env, step, &processor.name(), holders)
            },
        )
    } else {
        pipeline.run(env, &mut targets);
    }

    // execute and convert results
    let (vm_result, _) = interpret_internal(env, &targets, &entrypoint_env, ty_args, args, verbose);
    vm_result.map(|rets| {
        rets.into_iter()
            .map(|v| {
                let (ty, val, _) = v.decompose();
                let move_val = convert_typed_value_to_move_value(ty.get_base_type(), val);
                move_val.simple_serialize().unwrap()
            })
            .collect::<Vec<_>>()
    })
}

fn interpret_internal(
    env: &GlobalEnv,
    targets: &FunctionTargetsHolder,
    fun_env: &FunctionEnv,
    ty_args: &[TypeTag],
    args: &[MoveValue],
    verbose: bool,
) -> (VMResult<Vec<TypedValue>>, GlobalState) {
    // dump the bytecode if requested
    if verbose {
        let mut text = String::new();
        for module_env in env.get_modules() {
            for func_env in module_env.get_functions() {
                for (variant, target) in targets.get_targets(&func_env) {
                    target.register_annotation_formatters_for_test();
                    text += &format!("[variant {}]\n{}\n", variant, target);
                }
            }
        }
        println!("{}", text);
    }

    // invoke the runtime
    let vm = Runtime::new(env, targets);
    let global_state = GlobalState::default();
    vm.execute(fun_env, ty_args, args, global_state)
}

fn verbose_stepwise_processing(
    env: &GlobalEnv,
    step: usize,
    name: &str,
    targets: &FunctionTargetsHolder,
) {
    // short-circuit the execution if prior phases run into errors
    if env.has_errors() {
        return;
    }
    let mut text = String::new();
    for module_env in env.get_modules() {
        for func_env in module_env.get_functions() {
            for (_, target) in targets.get_targets(&func_env) {
                if !target.data.code.is_empty() {
                    target.register_annotation_formatters_for_test();
                    text += &format!("[{}-{}]\n{}\n", step, name, target);
                }
            }
        }
    }
    println!("{}", text);
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
