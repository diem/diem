// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::verifier::{VerifiedModule, VerifiedProgram};
use config::config::VMPublishingOption;
use ir_to_bytecode::{compiler::compile_program, parser::parse_program};
use language_e2e_tests::{account::AccountData, executor::FakeExecutor};
use std::{str::FromStr, time::Duration};
use stdlib::stdlib_modules;
use transaction_builder::transaction::{make_transaction_program, serialize_program};
use types::{
    transaction::{RawTransaction, TransactionArgument, TransactionOutput, TransactionStatus},
    vm_error::{ExecutionStatus, VMStatus},
};
use vm::{
    access::ModuleAccess,
    file_format::{CompiledModule, CompiledProgram, CompiledScript},
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
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stage {
    Parser,
    // Right now parser is a separate stage.
    // However it could be merged into the compiler.
    Compiler,
    Verifier,
    Serializer,
    Runtime,
}

impl FromStr for Stage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "parser" => Ok(Stage::Parser),
            "compiler" => Ok(Stage::Compiler),
            "verifier" => Ok(Stage::Verifier),
            "serializer" => Ok(Stage::Serializer),
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
                stage = Some(*s);
                break;
            }
        }
        stage
    }
}

fn do_verify_program(program: CompiledProgram, deps: &[VerifiedModule]) -> Result<VerifiedProgram> {
    Ok(VerifiedProgram::new(program, deps).map_err(ErrorKind::VerificationFailure)?)
}

/// Runs a single transaction using the fake executor.
fn run_transaction(
    exec: &mut FakeExecutor,
    data: &AccountData,
    program: &CompiledProgram,
    args: &[TransactionArgument],
) -> Result<TransactionOutput> {
    let account = data.account();

    let program = make_transaction_program(program, args)?;
    let account_resource = exec.read_account_resource(&account).unwrap();

    let transaction = RawTransaction::new(
        *data.address(),
        account_resource.sequence_number(),
        program,
        account_resource.balance(),
        1,
        Duration::from_secs(u64::max_value()),
    )
    .sign(&account.privkey, account.pubkey.clone())?
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

/// Serializes the program then deserializes it.
fn run_serializer_round_trip(program: &CompiledProgram) -> Result<()> {
    let (script_blob, module_blobs) = serialize_program(program)?;

    assert_eq!(module_blobs.len(), program.modules.len());

    let script = CompiledScript::deserialize(&script_blob)?;
    if script != program.script {
        return Err(ErrorKind::Other(
            "deserialized script different from original one".to_string(),
        )
        .into());
    }

    for (i, blob) in module_blobs.iter().enumerate() {
        let module = CompiledModule::deserialize(blob)?;
        if module != program.modules[i] {
            return Err(ErrorKind::Other(format!(
                "deserialized module {} different from original one",
                program.modules[i].name()
            ))
            .into());
        }
    }
    Ok(())
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
    let mut deps = stdlib_modules().to_vec();

    for transaction in transactions {
        // get the account data of the sender
        let data = config.accounts.get(&transaction.config.sender).unwrap();
        let addr = data.address();

        // start processing a new transaction
        // insert a barrier in the output
        res.outputs.push(EvaluationOutput::Transaction);

        // stage 1: parse the program
        if transaction.config.is_stage_disabled(Stage::Parser) {
            continue;
        }
        res.outputs.push(EvaluationOutput::Stage(Stage::Parser));
        let parsed_program = unwrap_or_log!(parse_program(&transaction.program), res);
        res.outputs
            .push(EvaluationOutput::Output(format!("{:?}", parsed_program)));

        // stage 2: compile the program
        if transaction.config.is_stage_disabled(Stage::Compiler) {
            continue;
        }
        res.outputs.push(EvaluationOutput::Stage(Stage::Compiler));
        let compiled_program = unwrap_or_log!(compile_program(addr, &parsed_program, &deps), res);
        res.outputs
            .push(EvaluationOutput::Output(format!("{:?}", compiled_program)));

        // stage 3: verify the program
        let compiled_program = if !transaction.config.is_stage_disabled(Stage::Verifier) {
            res.outputs.push(EvaluationOutput::Stage(Stage::Verifier));
            let verified_program = unwrap_or_log!(do_verify_program(compiled_program, &deps), res);
            res.outputs.push(EvaluationOutput::Output("".to_string()));

            // add all modules to be published to the vec of dependencies
            // TODO: currently the compiler only checks the module name when looking up a module
            //       it should check that both the name and address match
            let new_modules = verified_program.modules().to_vec();
            // This has to be before deps.extend since verified_program holds a reference to the
            // deps.
            let compiled_program = verified_program.into_inner();
            deps.extend(new_modules);

            compiled_program
        } else {
            // TODO: Should this add unverified modules to the deps list using the bypass function?
            // Not totally sure at the moment.
            compiled_program
        };

        // stage 4: serializer round trip
        if !transaction.config.is_stage_disabled(Stage::Serializer) {
            res.outputs.push(EvaluationOutput::Stage(Stage::Serializer));
            unwrap_or_log!(run_serializer_round_trip(&compiled_program), res);
        }

        // stage 5: execute the program
        if !transaction.config.is_stage_disabled(Stage::Runtime) {
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
