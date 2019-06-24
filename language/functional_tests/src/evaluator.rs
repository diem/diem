// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{config::Config, errors::*};
use bytecode_verifier::{
    verify_module, verify_module_dependencies, verify_script, verify_script_dependencies,
};
use compiler::{compiler::compile_program, parser::parse_program, util::build_stdlib};
use config::config::VMPublishingOption;
use std::time::Duration;
use types::{
    transaction::{Program, RawTransaction, TransactionOutput, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus},
};
use vm::{
    errors::VerificationError,
    file_format::{CompiledModule, CompiledProgram, CompiledScript},
};
use vm_runtime_tests::executor::FakeExecutor;

/// Indicates one step in the pipeline the given move module/program goes through.
//  Ord is derived as we need to be able to determine if one stage is before another.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stage {
    Parser,
    // Right now parser is a separate stage.
    // However it could be merged into the compiler.
    Compiler,
    Verifier,
    Runtime,
}

impl Stage {
    /// Parses the input string as Stage.
    pub fn parse(s: &str) -> Result<Stage> {
        match s {
            "parser" => Ok(Stage::Parser),
            "compiler" => Ok(Stage::Compiler),
            "verifier" => Ok(Stage::Verifier),
            "runtime" => Ok(Stage::Runtime),
            _ => Err(ErrorKind::Other(format!("unrecognized stage '{:?}'", s)).into()),
        }
    }
}

/// Evaluation status: success or failure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Status {
    Success,
    Failure,
}

/// An entry in the `EvaluationResult`.
#[derive(Debug)]
pub enum EvaluationOutput {
    Stage(Stage),
    Output(String),
    Error(String),
}

/// A log consisting of outputs from all stages and the final status.
/// This is checked against the directives.
#[derive(Debug)]
pub struct EvaluationResult {
    pub outputs: Vec<EvaluationOutput>,
    pub status: Status,
}

fn check_verification_errors(errors: Vec<VerificationError>) -> Result<()> {
    if !errors.is_empty() {
        return Err(ErrorKind::VerificationFailure(errors).into());
    }
    Ok(())
}

fn do_verify_module(module: &CompiledModule, deps: &[CompiledModule]) -> Result<()> {
    check_verification_errors(verify_module(module.clone()).1)?;
    check_verification_errors(verify_module_dependencies(module.clone(), deps).1)
}

fn do_verify_script(script: &CompiledScript, deps: &[CompiledModule]) -> Result<()> {
    check_verification_errors(verify_script(script.clone()).1)?;
    check_verification_errors(verify_script_dependencies(script.clone(), deps).1)
}

// TODO: Add a helper function to the verifier
fn do_verify_program(program: &CompiledProgram, deps: &[CompiledModule]) -> Result<()> {
    let mut deps = deps.to_vec();
    for m in &program.modules {
        do_verify_module(m, &deps)?;
        deps.push(m.clone());
    }
    do_verify_script(&program.script, &deps)
}

fn create_transaction_program(program: &CompiledProgram) -> Result<Program> {
    let mut script_blob = vec![];
    program.script.serialize(&mut script_blob)?;

    let module_blobs = program
        .modules
        .iter()
        .map(|m| {
            let mut module_blob = vec![];
            m.serialize(&mut module_blob)?;
            Ok(module_blob)
        })
        .collect::<Result<Vec<_>>>()?;

    // Currently we do not support transaction arguments in functional tests.
    Ok(Program::new(script_blob, module_blobs, vec![]))
}

/// Runs a single transaction using the fake executor.
fn run_transaction(config: &Config, program: &CompiledProgram) -> Result<TransactionOutput> {
    let mut exec = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }
    let data = config.accounts.get("alice").unwrap();
    let account = data.account();

    let program = create_transaction_program(program)?;

    let transaction = RawTransaction::new(
        *data.address(),
        data.sequence_number(),
        program,
        1_000_000,
        // Right now, max gas cost is an arbitratry large number.
        // TODO: allow the user to specify this in the config.
        1,
        Duration::from_secs(u64::max_value()),
    )
    .sign(&account.privkey, account.pubkey)?
    .into_inner();

    let mut outputs = exec.execute_block(vec![transaction]);
    if outputs.len() == 1 {
        let output = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)) => Ok(output),
            TransactionStatus::Keep(_) => Err(ErrorKind::VMExecutionFailure(output).into()),
            TransactionStatus::Discard(_) => Err(ErrorKind::DiscardedTransaction(output).into()),
        }
    } else {
        panic!("transaction outputs size mismatch");
    }
}

/// Tries to unwrap the given result. Upon failure, log the error and aborts.
macro_rules! unwrap_or_log {
    ($res: expr, $log: expr) => {{
        match $res {
            Ok(r) => r,
            Err(e) => {
                $log.outputs
                    .push(EvaluationOutput::Error(format!("{:?}", e)));
                return Ok($log);
            }
        }
    }};
}

/// Feeds the input through the pipeline and produces an EvaluationResult.
pub fn eval(config: &Config, text: &str) -> Result<EvaluationResult> {
    let mut res = EvaluationResult {
        outputs: vec![],
        status: Status::Failure,
    };

    let deps = build_stdlib();
    let account_data = config.accounts.get("alice").unwrap();
    let addr = account_data.address();

    res.outputs.push(EvaluationOutput::Stage(Stage::Parser));
    let parsed_program = unwrap_or_log!(parse_program(&text), res);
    res.outputs
        .push(EvaluationOutput::Output(format!("{:?}", parsed_program)));

    res.outputs.push(EvaluationOutput::Stage(Stage::Compiler));
    let compiled_program = unwrap_or_log!(compile_program(addr, &parsed_program, &deps), res);
    res.outputs
        .push(EvaluationOutput::Output(format!("{:?}", compiled_program)));

    if !config.no_verify {
        res.outputs.push(EvaluationOutput::Stage(Stage::Verifier));
        unwrap_or_log!(do_verify_program(&compiled_program, &deps), res);
        res.outputs.push(EvaluationOutput::Output("".to_string()));
    }

    if !config.no_execute {
        res.outputs.push(EvaluationOutput::Stage(Stage::Runtime));
        let txn_output = unwrap_or_log!(run_transaction(config, &compiled_program), res);
        res.outputs
            .push(EvaluationOutput::Output(format!("{:?}", txn_output)));
    }

    res.status = Status::Success;
    Ok(res)
}
