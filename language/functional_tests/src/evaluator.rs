// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::{
    verify_module, verify_module_dependencies, verify_script, verify_script_dependencies,
};
use compiler::{compiler::compile_program, parser::parse_program, util::build_stdlib};
use config::config::VMPublishingOption;
use std::time::Duration;
use types::{
    transaction::{
        Program, RawTransaction, TransactionArgument, TransactionOutput, TransactionStatus,
    },
    vm_error::{ExecutionStatus, VMStatus},
};
use vm::{
    errors::VerificationError,
    file_format::{CompiledModule, CompiledProgram, CompiledScript},
};
use vm_runtime_tests::{
    account::{AccountData, AccountResource},
    executor::FakeExecutor,
};

/// A transaction to be evaluated by the testing infra.
/// Contains code and a transaction config.
#[derive(Debug)]
pub struct Transaction {
    pub config: TransactionConfig,
    pub program: String,
}

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
    Transaction,
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

impl EvaluationResult {
    pub fn get_transaction_count(&self) -> usize {
        self.outputs
            .iter()
            .filter(|output| match output {
                EvaluationOutput::Transaction => true,
                _ => false,
            })
            .count()
    }

    pub fn get_last_stage(&self) -> Option<Stage> {
        let mut stage = None;
        for output in self.outputs.iter().rev() {
            if let EvaluationOutput::Stage(s) = output {
                stage = Some(s.clone());
                break;
            }
        }
        stage
    }
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

fn create_transaction_program(
    program: &CompiledProgram,
    args: &[TransactionArgument],
) -> Result<Program> {
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
    Ok(Program::new(script_blob, module_blobs, args.to_vec()))
}

/// Runs a single transaction using the fake executor.
fn run_transaction(
    exec: &mut FakeExecutor,
    data: &AccountData,
    program: &CompiledProgram,
    args: &[TransactionArgument],
) -> Result<TransactionOutput> {
    let account = data.account();

    let program = create_transaction_program(program, args)?;
    let account_resource = exec.read_account_resource(&account).unwrap();

    let transaction = RawTransaction::new(
        *data.address(),
        AccountResource::read_sequence_number(&account_resource),
        program,
        AccountResource::read_balance(&account_resource),
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

/// Feeds all given transactions through the pipeline and produces an EvaluationResult.
pub fn eval(config: &GlobalConfig, transactions: &[Transaction]) -> Result<EvaluationResult> {
    // set up empty evaluation result
    let mut res = EvaluationResult {
        outputs: vec![],
        status: Status::Failure,
    };

    // set up a fake executor with the genesis block and create the accounts
    let mut exec = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }

    // set up standard library
    // needed to compile transaction programs
    let stdlib = build_stdlib();

    for transaction in transactions {
        // get the account data of the sender
        let data = config.accounts.get(&transaction.config.sender).unwrap();
        let addr = data.address();

        // start processing a new transaction
        // insert a barrier in the output
        res.outputs.push(EvaluationOutput::Transaction);

        // stage 1: parse the program
        res.outputs.push(EvaluationOutput::Stage(Stage::Parser));
        let parsed_program = unwrap_or_log!(parse_program(&transaction.program), res);
        res.outputs
            .push(EvaluationOutput::Output(format!("{:?}", parsed_program)));

        // stage 2: compile the program
        res.outputs.push(EvaluationOutput::Stage(Stage::Compiler));
        let compiled_program = unwrap_or_log!(compile_program(addr, &parsed_program, &stdlib), res);
        res.outputs
            .push(EvaluationOutput::Output(format!("{:?}", compiled_program)));

        // stage 3: verify the program
        if !transaction.config.no_verify {
            res.outputs.push(EvaluationOutput::Stage(Stage::Verifier));
            unwrap_or_log!(do_verify_program(&compiled_program, &stdlib), res);
            res.outputs.push(EvaluationOutput::Output("".to_string()));
        }

        // stage 4: execute the program
        if !transaction.config.no_execute {
            res.outputs.push(EvaluationOutput::Stage(Stage::Runtime));
            let txn_output = unwrap_or_log!(
                run_transaction(&mut exec, data, &compiled_program, &transaction.config.args),
                res
            );
            res.outputs
                .push(EvaluationOutput::Output(format!("{:?}", txn_output)));

            // apply the writeset
            exec.apply_write_set(txn_output.write_set());
        }
    }

    res.status = Status::Success;
    Ok(res)
}
