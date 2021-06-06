// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use codespan_reporting::{diagnostic::Severity, term::termcolor::Buffer};
use structopt::StructOpt;

use bytecode::{
    function_target_pipeline::{
        FunctionTargetProcessor, FunctionTargetsHolder, ProcessorResultDisplay,
    },
    options::ProverOptions,
    pipeline_factory::default_pipeline_with_options,
};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, TypeTag},
    parser::{parse_transaction_argument, parse_type_tag},
    transaction_argument::TransactionArgument,
    value::{MoveStruct, MoveValue},
    vm_status::StatusCode,
};
use move_model::{
    model::{FunctionEnv, GlobalEnv},
    ty::{PrimitiveType as ModelPrimitiveType, Type as ModelType},
};

pub mod concrete;
pub mod shared;

use crate::concrete::{
    runtime::{convert_move_type_tag, Runtime},
    settings::InterpreterSettings,
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

    /// Skip checking of expressions
    #[structopt(long = "no-expr-check")]
    pub no_expr_check: bool,
    /// Level of verbosity
    #[structopt(short = "v", long = "verbose")]
    pub verbose: Option<u64>,
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
) -> (VMResult<Vec<Vec<u8>>>, ChangeSet, GlobalState) {
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

    // collect settings
    let settings = InterpreterSettings {
        no_expr_check: options.no_expr_check,
        verbose_stepwise: options.verbose.map_or(false, |level| level > 0),
        verbose_bytecode: options.verbose.map_or(false, |level| level > 1),
        verbose_expression: options.verbose.map_or(false, |level| level > 2),
    };

    // run the actual interpreter
    let interpreter = StacklessBytecodeInterpreter::new(env, None, settings);
    interpreter.interpret(
        &options.entrypoint.0,
        &options.entrypoint.1,
        &options.ty_args,
        &args,
        &GlobalState::default(),
    )
}

pub struct StacklessBytecodeInterpreter<'env> {
    pub env: &'env GlobalEnv,
    targets: FunctionTargetsHolder,
}

impl<'env> StacklessBytecodeInterpreter<'env> {
    pub fn new(
        env: &'env GlobalEnv,
        options_opt: Option<ProverOptions>,
        settings: InterpreterSettings,
    ) -> Self {
        // make sure that the global env does not have an error
        if env.has_errors() {
            let mut buffer = Buffer::no_color();
            env.report_diag(&mut buffer, Severity::Error);
            panic!(
                "errors accumulated in the model builder and the transformation pipeline\n{}",
                String::from_utf8_lossy(&buffer.into_inner())
            )
        }

        // create the pipeline
        let options = options_opt.unwrap_or_else(|| ProverOptions {
            for_interpretation: true,
            ..Default::default()
        });
        let pipeline = default_pipeline_with_options(&options);
        env.set_extension(options);

        // collect and transform function targets
        let mut targets = FunctionTargetsHolder::default();
        for module_env in env.get_modules() {
            for func_env in module_env.get_functions() {
                targets.add_target(&func_env)
            }
        }
        if settings.verbose_stepwise {
            pipeline.run_with_hook(
                env,
                &mut targets,
                |holder| verbose_stepwise_processing(env, 0, None, holder),
                |step, processor, holders| {
                    verbose_stepwise_processing(env, step, Some(processor), holders)
                },
            )
        } else {
            pipeline.run(env, &mut targets);
        }

        // dump the bytecode if requested
        if settings.verbose_stepwise {
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

        // register settings with the env before returning
        env.set_extension(settings);
        Self { env, targets }
    }

    fn interpret_internal(
        &self,
        fun_env: &FunctionEnv,
        ty_args: &[TypeTag],
        args: &[MoveValue],
        global_state: &GlobalState,
    ) -> (VMResult<Vec<Vec<u8>>>, ChangeSet, GlobalState) {
        let mut new_global_state = global_state.clone();

        // execute and convert results
        let vm = Runtime::new(self.env, &self.targets);
        let vm_result = vm.execute(fun_env, ty_args, args, &mut new_global_state);
        let serialized_vm_result = vm_result.map(|rets| {
            rets.into_iter()
                .map(|v| {
                    let (ty, val, _) = v.decompose();
                    let move_val = convert_typed_value_to_move_value(ty.get_base_type(), val);
                    move_val.simple_serialize().unwrap()
                })
                .collect::<Vec<_>>()
        });

        let change_set = new_global_state.delta(global_state);
        (serialized_vm_result, change_set, new_global_state)
    }

    pub fn interpret(
        &self,
        module_id: &ModuleId,
        func_name: &IdentStr,
        ty_args: &[TypeTag],
        args: &[MoveValue],
        global_state: &GlobalState,
    ) -> (VMResult<Vec<Vec<u8>>>, ChangeSet, GlobalState) {
        // find the entrypoint
        let entrypoint_env = match derive_entrypoint_env(self.env, module_id, func_name) {
            Ok(func_env) => func_env,
            Err(err) => {
                return (
                    Err(err.finish(Location::Undefined)),
                    ChangeSet::new(),
                    global_state.clone(),
                )
            }
        };

        // run the actual interpretation
        self.interpret_internal(&entrypoint_env, ty_args, args, global_state)
    }

    pub fn interpret_with_bcs_arguments(
        &self,
        module_id: &ModuleId,
        func_name: &IdentStr,
        ty_args: &[TypeTag],
        bcs_args: &[Vec<u8>],
        senders_opt: Option<&[AccountAddress]>,
        global_state: &GlobalState,
    ) -> (VMResult<Vec<Vec<u8>>>, ChangeSet, GlobalState) {
        // find the entrypoint
        let entrypoint_env = match derive_entrypoint_env(self.env, module_id, func_name) {
            Ok(func_env) => func_env,
            Err(err) => {
                return (
                    Err(err.finish(Location::Undefined)),
                    ChangeSet::new(),
                    global_state.clone(),
                )
            }
        };

        // convert the args
        let args = match convert_bcs_arguments_to_move_value_arguments(
            &entrypoint_env,
            bcs_args,
            senders_opt,
        ) {
            Ok(args) => args,
            Err(err) => {
                return (
                    Err(err.finish(Location::Undefined)),
                    ChangeSet::new(),
                    global_state.clone(),
                )
            }
        };

        // run the actual interpretation
        self.interpret_internal(&entrypoint_env, ty_args, &args, global_state)
    }

    pub fn report_property_checking_results(&self) -> Option<String> {
        if self.env.has_errors() {
            let mut buffer = Buffer::no_color();
            self.env.report_diag(&mut buffer, Severity::Error);
            self.env.clear_diag();
            Some(String::from_utf8_lossy(&buffer.into_inner()).to_string())
        } else {
            None
        }
    }
}

fn verbose_stepwise_processing(
    env: &GlobalEnv,
    step: usize,
    processor_opt: Option<&dyn FunctionTargetProcessor>,
    targets: &FunctionTargetsHolder,
) {
    // short-circuit the execution if prior phases run into errors
    if env.has_errors() {
        return;
    }

    let name = processor_opt
        .map(|processor| processor.name())
        .unwrap_or_else(|| "stackless".to_string());

    // dump bytecode
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

    // dump analysis results, if any
    if let Some(processor) = processor_opt {
        text += &ProcessorResultDisplay {
            env,
            targets,
            processor,
        }
        .to_string();
    }
    println!("{}", text);
}

fn derive_entrypoint_env<'env>(
    env: &'env GlobalEnv,
    module_id: &ModuleId,
    func_name: &IdentStr,
) -> PartialVMResult<FunctionEnv<'env>> {
    env.find_module_by_language_storage_id(module_id)
        .and_then(|module_env| module_env.find_function(env.symbol_pool().make(func_name.as_str())))
        .ok_or_else(|| PartialVMError::new(StatusCode::LOOKUP_FAILED))
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

fn convert_bcs_arguments_to_move_value_arguments(
    func_env: &FunctionEnv,
    bcs_args: &[Vec<u8>],
    senders_opt: Option<&[AccountAddress]>,
) -> PartialVMResult<Vec<MoveValue>> {
    let env = func_env.module_env.env;
    let mut move_vals = vec![];

    let params = func_env.get_parameters();
    let num_signer_args = match senders_opt {
        None => 0,
        Some(senders) => {
            for (i, param) in params.iter().enumerate() {
                match &param.1 {
                    ModelType::Primitive(ModelPrimitiveType::Signer) => {
                        if i >= senders.len() {
                            return Err(PartialVMError::new(
                                StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH,
                            ));
                        }
                        move_vals.push(MoveValue::Signer(senders[i]))
                    }
                    _ => {
                        if i != 0 && i != senders.len() {
                            return Err(PartialVMError::new(
                                StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH,
                            ));
                        }
                        break;
                    }
                }
            }
            move_vals.len()
        }
    };

    if (num_signer_args + bcs_args.len()) != params.len() {
        return Err(PartialVMError::new(
            StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH,
        ));
    }
    for (arg_bcs, param) in bcs_args.iter().zip(&params[num_signer_args..]) {
        match param.1.clone().into_type_tag(env) {
            None => {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH));
            }
            Some(type_tag) => {
                let ty = convert_move_type_tag(env, &type_tag)?;
                let val = MoveValue::simple_deserialize(arg_bcs, &ty.to_move_type_layout())
                    .map_err(|_| PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT))?;
                move_vals.push(val);
            }
        }
    }
    Ok(move_vals)
}
