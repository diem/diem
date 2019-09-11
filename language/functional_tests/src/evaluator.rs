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
use language_e2e_tests::{account::AccountData, executor::FakeExecutor};
use std::{str::FromStr, time::Duration};
use stdlib::stdlib_modules;
use types::{
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript,
        SignedTransaction, TransactionArgument, TransactionOutput, TransactionStatus,
    },
    vm_error::StatusCode,
};
use vm::file_format::{CompiledModule, CompiledScript};
use types::account_address::AccountAddress;
use types::transaction::ChannelScriptPayload;
use types::write_set::WriteSet;

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

/// Verifies a script & its dependencies.
fn do_verify_script(script: CompiledScript, deps: &[VerifiedModule]) -> Result<VerifiedScript> {
    let verified_script = match VerifiedScript::new(script) {
        Ok(verified_script) => verified_script,
        Err((_, errs)) => return Err(ErrorKind::VerificationFailure(errs).into()),
    };
    let errs = verify_script_dependencies(&verified_script, deps);
    if !errs.is_empty() {
        return Err(ErrorKind::VerificationFailure(errs).into());
    }
    Ok(verified_script)
}

/// Verifies a module & its dependencies.
fn do_verify_module(module: CompiledModule, deps: &[VerifiedModule]) -> Result<VerifiedModule> {
    let verified_module = match VerifiedModule::new(module) {
        Ok(verified_module) => verified_module,
        Err((_, errs)) => return Err(ErrorKind::VerificationFailure(errs).into()),
    };
    let errs = verify_module_dependencies(&verified_module, deps);
    if !errs.is_empty() {
        return Err(ErrorKind::VerificationFailure(errs).into());
    }
    Ok(verified_module)
}

/// Creates and signs a script transaction.
fn make_script_transaction(
    exec: &FakeExecutor,
    data: &AccountData,
    script: CompiledScript,
    args: Vec<TransactionArgument>,
    receiver: Option<AccountAddress>,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    script.serialize(&mut blob)?;
    let script = TransactionScript::new(blob, args);

    let account = data.account();
    let account_resource = exec.read_account_resource(&account).unwrap();
    let mut txn = match receiver {
        //TODO support channel sequence number
        Some(receiver) => RawTransaction::new_channel_script(
            *data.address(),
            account_resource.sequence_number(),
            ChannelScriptPayload::new(0, WriteSet::default(), receiver, script),
            account_resource.balance(),
            1,
            Duration::from_secs(u64::max_value()),
        ).sign(&account.privkey, account.pubkey.clone())?
            .into_inner(),
        None => RawTransaction::new_script(
            *data.address(),
            account_resource.sequence_number(),
            script,
            account_resource.balance(),
            1,
            Duration::from_secs(u64::max_value()),
        ).sign(&account.privkey, account.pubkey.clone())?
            .into_inner()
    };
    Ok(txn)
}

/// Creates and signs a module transaction.
fn make_module_transaction(
    exec: &FakeExecutor,
    data: &AccountData,
    module: CompiledModule,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    module.serialize(&mut blob)?;
    let module = TransactionModule::new(blob);

    let account = data.account();
    let account_resource = exec.read_account_resource(&account).unwrap();
    Ok(RawTransaction::new_module(
        *data.address(),
        account_resource.sequence_number(),
        module,
        account_resource.balance(),
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
            TransactionStatus::Keep(status) if status.major_status == StatusCode::EXECUTED => {
                Ok(output)
            }
            TransactionStatus::Keep(_) => Err(ErrorKind::VMExecutionFailure(output).into()),
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
        let receiver = transaction.config.receiver.as_ref().and_then(|receiver|config.accounts.get(receiver)).map(|account_data|account_data.address().clone());
        let addr = data.address();

        // start processing a new transaction
        // insert a barrier in the output
        res.outputs.push(EvaluationOutput::Transaction);

        // stage 1: parse the script/module
        if transaction.config.is_stage_disabled(Stage::Parser) {
            continue;
        }
        res.outputs.push(EvaluationOutput::Stage(Stage::Parser));
        let parsed_script_or_module =
            unwrap_or_log!(parse_script_or_module(&transaction.input), res);
        res.outputs.push(EvaluationOutput::Output(format!(
            "{:?}",
            parsed_script_or_module
        )));

        match parsed_script_or_module {
            ScriptOrModule::Script(parsed_script) => {
                // stage 2: compile the script
                if transaction.config.is_stage_disabled(Stage::Compiler) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Compiler));

                let compiled_script =
                    unwrap_or_log!(compile_script(*addr, parsed_script, &deps), res);
                res.outputs
                    .push(EvaluationOutput::Output(format!("{:?}", compiled_script)));

                // stage 3: verify the script
                if transaction.config.is_stage_disabled(Stage::Verifier) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Verifier));
                let compiled_script =
                    unwrap_or_log!(do_verify_script(compiled_script, &deps), res).into_inner();

                // stage 4: serializer round trip
                if !transaction.config.is_stage_disabled(Stage::Serializer) {
                    res.outputs.push(EvaluationOutput::Stage(Stage::Serializer));
                    unwrap_or_log!(serialize_and_deserialize_script(&compiled_script), res);
                }

                // stage 5: execute the script
                if transaction.config.is_stage_disabled(Stage::Runtime) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Runtime));
                let script_transaction = make_script_transaction(
                    &exec,
                    data,
                    compiled_script,
                    transaction.config.args.clone(),
                    receiver,
                )?;
                let txn_output =
                    unwrap_or_log!(run_transaction(&mut exec, script_transaction), res);
                exec.apply_write_set(txn_output.write_set());
            }
            ScriptOrModule::Module(parsed_module) => {
                // stage 2: compile the module
                if transaction.config.is_stage_disabled(Stage::Compiler) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Compiler));

                let compiled_module =
                    unwrap_or_log!(compile_module(*addr, parsed_module, &deps), res);
                res.outputs
                    .push(EvaluationOutput::Output(format!("{:?}", compiled_module)));

                // module is added to the list of dependencies despite it passes the verifier or
                // not
                deps.push(VerifiedModule::bypass_verifier_DANGEROUS_FOR_TESTING_ONLY(
                    compiled_module.clone(),
                ));

                // stage 3: verify the module
                if transaction.config.is_stage_disabled(Stage::Verifier) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Verifier));
                let compiled_module =
                    unwrap_or_log!(do_verify_module(compiled_module, &deps), res).into_inner();

                // stage 4: serializer round trip
                if !transaction.config.is_stage_disabled(Stage::Serializer) {
                    res.outputs.push(EvaluationOutput::Stage(Stage::Serializer));
                    unwrap_or_log!(serialize_and_deserialize_module(&compiled_module), res);
                }

                // stage 5: publish the module
                if transaction.config.is_stage_disabled(Stage::Runtime) {
                    continue;
                }
                res.outputs.push(EvaluationOutput::Stage(Stage::Runtime));
                let module_transaction = make_module_transaction(&exec, data, compiled_module)?;
                let txn_output =
                    unwrap_or_log!(run_transaction(&mut exec, module_transaction), res);
                exec.apply_write_set(txn_output.write_set());
            }
        }
    }

    res.status = Status::Success;
    Ok(res)
}
