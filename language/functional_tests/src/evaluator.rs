// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::verifier::{
    verify_module_dependencies, verify_script_dependencies, VerifiedModule, VerifiedScript,
};
use config::config::VMPublishingOption;
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::parse_script_or_module,
};
use ir_to_bytecode_syntax::ast::ScriptOrModule;
use language_e2e_tests::{account::Account, executor::FakeExecutor};
use libra_types::access_path::AccessPath;
use libra_types::channel_account::{channel_account_struct_tag, ChannelAccountResource};
use libra_types::{
    transaction::{
        ChannelScriptBody, Module as TransactionModule, RawTransaction,
        Script as TransactionScript, SignedTransaction, TransactionOutput, TransactionStatus,
    },
    vm_error::StatusCode,
    write_set::WriteSet,
};
use std::{fmt, str::FromStr, time::Duration};
use stdlib::stdlib_modules;
use vm::file_format::{CompiledModule, CompiledScript};
use vm::gas_schedule::{GasAlgebra, MAXIMUM_NUMBER_OF_GAS_UNITS};

/// A transaction to be evaluated by the testing infra.
/// Contains code and a transaction config.
#[derive(Debug)]
pub struct Transaction {
    pub config: TransactionConfig,
    pub input: String,
}

/// Indicates one step in the pipeline the given move module/program goes through.
//  Ord is derived as we need to be able to determine if one stage is before another.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stage {
    Parser,
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

#[derive(Debug, Clone)]
pub enum OutputType {
    CompiledModule(CompiledModule),
    CompiledScript(CompiledScript),
    Ast(ScriptOrModule),
    TransactionOutput(TransactionOutput),
}

impl OutputType {
    pub fn to_check_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub type TransactionId = usize;

/// An entry in the `EvaluationLog`.
#[derive(Debug)]
pub enum EvaluationOutput {
    Transaction(TransactionId),
    Stage(Stage),
    Output(Box<OutputType>),
    Error(Box<Error>),
    Status(Status),
}

/// A log consisting of outputs from all stages and the final status.
/// This is checked against the directives.
#[derive(Debug)]
pub struct EvaluationLog {
    pub outputs: Vec<EvaluationOutput>,
}

impl EvaluationLog {
    pub fn get_failed_transactions(&self) -> Vec<(usize, Stage)> {
        let mut res = vec![];
        let mut last_txn = None;
        let mut last_stage = None;

        for output in &self.outputs {
            match output {
                EvaluationOutput::Transaction(idx) => last_txn = Some(idx),
                EvaluationOutput::Stage(stage) => last_stage = Some(stage),
                EvaluationOutput::Status(Status::Failure) => match (last_txn, last_stage) {
                    (Some(idx), Some(stage)) => res.push((*idx, *stage)),
                    _ => unreachable!(),
                },
                _ => (),
            }
        }

        res
    }

    fn append(&mut self, output: EvaluationOutput) {
        self.outputs.push(output);
    }
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use OutputType::*;
        match self {
            CompiledModule(cm) => write!(f, "{:#?}", cm),
            CompiledScript(cs) => write!(f, "{:#?}", cs),
            Ast(ast) => write!(f, "{}", ast),
            TransactionOutput(output) => write!(f, "{:#?}", output),
        }
    }
}

impl fmt::Display for EvaluationOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use EvaluationOutput::*;
        match self {
            Transaction(idx) => write!(f, "Transaction {}", idx),
            Stage(stage) => write!(f, "Stage: {:?}", stage),
            Output(output) => write!(f, "{}", output),
            Error(error) => write!(f, "Error: {:#?}", error),
            Status(status) => write!(f, "Status: {:?}", status),
        }
    }
}

impl fmt::Display for EvaluationLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "---------------------Outputs----------------")?;
        for (i, output) in self.outputs.iter().enumerate() {
            writeln!(f, "[{}] {}", i, output)?;
        }
        Ok(())
    }
}

/// Verifies a script & its dependencies.
fn do_verify_script(script: CompiledScript, deps: &[VerifiedModule]) -> Result<VerifiedScript> {
    let verified_script =
        VerifiedScript::new(script).map_err(|(_, errs)| ErrorKind::VerificationFailure(errs))?;
    let errs = verify_script_dependencies(&verified_script, deps);
    if !errs.is_empty() {
        return Err(ErrorKind::VerificationFailure(errs).into());
    }
    Ok(verified_script)
}

/// Verifies a module & its dependencies.
fn do_verify_module(module: CompiledModule, deps: &[VerifiedModule]) -> Result<VerifiedModule> {
    let verified_module =
        VerifiedModule::new(module).map_err(|(_, errs)| ErrorKind::VerificationFailure(errs))?;
    let errs = verify_module_dependencies(&verified_module, deps);
    if !errs.is_empty() {
        return Err(ErrorKind::VerificationFailure(errs).into());
    }
    Ok(verified_module)
}

/// Creates and signs a script transaction.
fn make_script_transaction(
    exec: &FakeExecutor,
    account: &Account,
    config: &TransactionConfig,
    script: CompiledScript,
    receiver: Option<(&Account, u64)>,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    script.serialize(&mut blob)?;
    let script = TransactionScript::new(blob, config.args.clone());

    let account_resource = exec.read_account_resource(&account).unwrap();
    let txn = match receiver {
        //TODO support channel sequence number
        Some((receiver, channel_sequence_number)) => {
            let body = ChannelScriptBody::new(
                channel_sequence_number,
                WriteSet::default(),
                *receiver.address(),
                script,
            );
            let payload = body.sign(&receiver.privkey, receiver.pubkey.clone());
            let txn = RawTransaction::new_channel(
                *account.address(),
                account_resource.sequence_number(),
                payload,
                std::cmp::min(
                    MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                    account_resource.balance(),
                ),
                1,
                Duration::from_secs(u64::max_value()),
            )
            .sign(&account.privkey, account.pubkey.clone())?
            .into_inner();
            txn
        }
        None => RawTransaction::new_script(
            *account.address(),
            config
                .sequence_number
                .unwrap_or_else(|| account_resource.sequence_number()),
            script,
            config.max_gas.unwrap_or_else(|| {
                std::cmp::min(
                    MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                    account_resource.balance(),
                )
            }),
            1,
            Duration::from_secs(u64::max_value()),
        )
        .sign(&account.privkey, account.pubkey.clone())?
        .into_inner(),
    };
    Ok(txn)
}

/// Creates and signs a module transaction.
fn make_module_transaction(
    exec: &FakeExecutor,
    account: &Account,
    config: &TransactionConfig,
    module: CompiledModule,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    module.serialize(&mut blob)?;
    let module = TransactionModule::new(blob);

    let account_resource = exec.read_account_resource(&account).unwrap();
    Ok(RawTransaction::new_module(
        *account.address(),
        config
            .sequence_number
            .unwrap_or_else(|| account_resource.sequence_number()),
        module,
        config.max_gas.unwrap_or_else(|| {
            std::cmp::min(
                MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                account_resource.balance(),
            )
        }),
        1,
        Duration::from_secs(u64::max_value()),
    )
    .sign(&account.privkey, account.pubkey.clone())?
    .into_inner())
}

/// Runs a single transaction using the fake executor.
fn run_transaction(
    exec: &mut FakeExecutor,
    transaction: SignedTransaction,
) -> Result<TransactionOutput> {
    let mut outputs = exec.execute_block(vec![transaction]);
    if outputs.len() == 1 {
        let output = outputs.pop().unwrap();
        match output.status() {
            TransactionStatus::Keep(status) => {
                exec.apply_write_set(output.write_set());
                if status.major_status == StatusCode::EXECUTED {
                    Ok(output)
                } else {
                    Err(ErrorKind::VMExecutionFailure(output).into())
                }
            }
            TransactionStatus::Discard(_) => Err(ErrorKind::DiscardedTransaction(output).into()),
        }
    } else {
        panic!("transaction outputs size mismatch");
    }
}

/// Serializes the script then deserializes it.
fn serialize_and_deserialize_script(script: &CompiledScript) -> Result<()> {
    let mut script_blob = vec![];
    script.serialize(&mut script_blob)?;
    let deserialized_script = CompiledScript::deserialize(&script_blob)?;

    if *script != deserialized_script {
        return Err(ErrorKind::Other(
            "deserialized script different from original one".to_string(),
        )
        .into());
    }

    Ok(())
}

/// Serializes the module then deserializes it.
fn serialize_and_deserialize_module(module: &CompiledModule) -> Result<()> {
    let mut module_blob = vec![];
    module.serialize(&mut module_blob)?;
    let deserialized_module = CompiledModule::deserialize(&module_blob)?;

    if *module != deserialized_module {
        return Err(ErrorKind::Other(
            "deserialized module different from original one".to_string(),
        )
        .into());
    }

    Ok(())
}

/// Tries to unwrap the given result. Upon failure, log the error and aborts.
macro_rules! unwrap_or_abort {
    ($res: expr, $log: expr) => {{
        match $res {
            Ok(r) => r,
            Err(e) => {
                $log.append(EvaluationOutput::Error(Box::new(e)));
                return Ok(Status::Failure);
            }
        }
    }};
}

fn eval_transaction(
    config: &GlobalConfig,
    exec: &mut FakeExecutor,
    deps: &mut Vec<VerifiedModule>,
    idx: usize,
    transaction: &Transaction,
    log: &mut EvaluationLog,
) -> Result<Status> {
    // get the account of the sender
    let account = config
        .get_account_for_name(&transaction.config.sender)
        .unwrap();
    let addr = account.address();

    let receiver = transaction
        .config
        .receiver
        .as_ref()
        .and_then(|receiver| config.accounts.get(receiver))
        .and_then(|receiver_account| {
            let access_path = AccessPath::channel_resource_access_path(
                *account.address(),
                *receiver_account.address(),
                channel_account_struct_tag(),
            );
            let channel_sequence_number = exec
                .read_from_access_path(&access_path)
                .map(|bytes| {
                    ChannelAccountResource::make_from(bytes)
                        .unwrap()
                        .channel_sequence_number()
                })
                .unwrap_or(0);
            Some((receiver_account.account(), channel_sequence_number))
        });

    // start processing a new transaction
    // insert a barrier in the output
    log.append(EvaluationOutput::Transaction(idx));

    // stage 1: parse the script/module
    if transaction.config.is_stage_disabled(Stage::Parser) {
        return Ok(Status::Success);
    }
    log.append(EvaluationOutput::Stage(Stage::Parser));
    let parsed_script_or_module = unwrap_or_abort!(parse_script_or_module(&transaction.input), log);
    log.append(EvaluationOutput::Output(Box::new(OutputType::Ast(
        parsed_script_or_module.clone(),
    ))));

    match parsed_script_or_module {
        ScriptOrModule::Script(parsed_script) => {
            // stage 2: compile the script
            if transaction.config.is_stage_disabled(Stage::Compiler) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Compiler));

            let compiled_script =
                unwrap_or_abort!(compile_script(*addr, parsed_script, &*deps), log).0;
            log.append(EvaluationOutput::Output(Box::new(
                OutputType::CompiledScript(compiled_script.clone()),
            )));

            // stage 3: verify the script
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Verifier));
            let compiled_script =
                unwrap_or_abort!(do_verify_script(compiled_script, &*deps), log).into_inner();

            // stage 4: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_script(&compiled_script), log);
            }

            // stage 5: execute the script
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let script_transaction = make_script_transaction(
                &exec,
                account,
                &transaction.config,
                compiled_script,
                receiver,
            )?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, script_transaction), log);
            log.append(EvaluationOutput::Output(Box::new(
                OutputType::TransactionOutput(txn_output),
            )));
        }
        ScriptOrModule::Module(parsed_module) => {
            // stage 2: compile the module
            if transaction.config.is_stage_disabled(Stage::Compiler) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Compiler));

            let compiled_module =
                unwrap_or_abort!(compile_module(*addr, parsed_module, &*deps), log).0;
            log.append(EvaluationOutput::Output(Box::new(
                OutputType::CompiledModule(compiled_module.clone()),
            )));

            // module is added to the list of dependencies despite it passes the verifier or
            // not
            deps.push(VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(
                compiled_module.clone(),
            ));

            // stage 3: verify the module
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Verifier));
            let compiled_module =
                unwrap_or_abort!(do_verify_module(compiled_module, &*deps), log).into_inner();

            // stage 4: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_module(&compiled_module), log);
            }

            // stage 5: publish the module
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let module_transaction =
                make_module_transaction(&exec, account, &transaction.config, compiled_module)?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, module_transaction), log);
            log.append(EvaluationOutput::Output(Box::new(
                OutputType::TransactionOutput(txn_output),
            )));
        }
    }
    Ok(Status::Success)
}

/// Feeds all given transactions through the pipeline and produces an EvaluationLog.
pub fn eval(config: &GlobalConfig, transactions: &[Transaction]) -> Result<EvaluationLog> {
    let mut log = EvaluationLog { outputs: vec![] };

    // set up a fake executor with the genesis block and create the accounts
    let mut exec = FakeExecutor::from_genesis_with_options(VMPublishingOption::Open);
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }

    // set up standard library
    // needed to compile transaction programs
    let mut deps = stdlib_modules().to_vec();

    for (idx, transaction) in transactions.iter().enumerate() {
        let status = eval_transaction(config, &mut exec, &mut deps, idx, transaction, &mut log)?;
        log.append(EvaluationOutput::Status(status));
    }

    Ok(log)
}
