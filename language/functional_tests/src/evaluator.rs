// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::verifier::{
    verify_module_dependencies, verify_script_dependencies, VerifiedModule, VerifiedScript,
};
use ir_to_bytecode::{
    compiler::{compile_module, compile_script},
    parser::parse_script_or_module,
};
use ir_to_bytecode_syntax::ast::ScriptOrModule;
use language_e2e_tests::executor::FakeExecutor;
use libra_config::config::VMPublishingOption;
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use libra_types::{
    account_address::AccountAddress,
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript,
        SignedTransaction, TransactionOutput, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
};
use std::{
    fmt::{self, Debug},
    str::FromStr,
    time::Duration,
};
use stdlib::stdlib_modules;
use vm::file_format::{CompiledModule, CompiledScript};
use vm::gas_schedule::{GasAlgebra, MAXIMUM_NUMBER_OF_GAS_UNITS};

/// A transaction to be evaluated by the testing infra.
/// Contains code and a transaction config.
#[derive(Debug)]
pub struct Transaction<'a> {
    pub config: TransactionConfig<'a>,
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

impl EvaluationOutput {
    pub fn is_error(&self) -> bool {
        match self {
            Self::Error(_) => true,
            _ => false,
        }
    }
}

/// A log consisting of outputs from all stages and the final status.
/// This is checked against the directives.
#[derive(Debug)]
pub struct EvaluationLog {
    pub outputs: Vec<EvaluationOutput>,
}

impl EvaluationLog {
    pub fn new() -> Self {
        Self { outputs: vec![] }
    }

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

    pub fn append(&mut self, output: EvaluationOutput) {
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
        for (i, output) in self.outputs.iter().enumerate() {
            writeln!(f, "[{}] {}", i, output)?;
        }
        Ok(())
    }
}

/// Verifies a script & its dependencies.
fn do_verify_script(
    script: CompiledScript,
    deps: &[VerifiedModule],
) -> std::result::Result<VerifiedScript, Vec<VMStatus>> {
    let verified_script = VerifiedScript::new(script).map_err(|(_, errs)| errs)?;
    let errs = verify_script_dependencies(&verified_script, deps);
    if !errs.is_empty() {
        return Err(errs);
    }
    Ok(verified_script)
}

/// Verifies a module & its dependencies.
fn do_verify_module(
    module: CompiledModule,
    deps: &[VerifiedModule],
) -> std::result::Result<VerifiedModule, Vec<VMStatus>> {
    let verified_module = VerifiedModule::new(module).map_err(|(_, errs)| errs)?;
    let errs = verify_module_dependencies(&verified_module, deps);
    if !errs.is_empty() {
        return Err(errs);
    }
    Ok(verified_module)
}

/// A set of common parameters required to create transactions.
struct TransactionParameters<'a> {
    pub sender_addr: AccountAddress,
    pub pubkey: &'a Ed25519PublicKey,
    pub privkey: &'a Ed25519PrivateKey,
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub expiration_time: Duration,
}

/// Gets the transaction parameters from the current execution environment and the config.
fn get_transaction_parameters<'a>(
    exec: &'a FakeExecutor,
    config: &'a TransactionConfig,
) -> TransactionParameters<'a> {
    let account_resource = exec.read_account_resource(config.sender).unwrap();

    TransactionParameters {
        sender_addr: *config.sender.address(),
        pubkey: &config.sender.pubkey,
        privkey: &config.sender.privkey,
        sequence_number: config
            .sequence_number
            .unwrap_or_else(|| account_resource.sequence_number()),
        max_gas_amount: config.max_gas.unwrap_or_else(|| {
            std::cmp::min(
                MAXIMUM_NUMBER_OF_GAS_UNITS.get(),
                account_resource.balance(),
            )
        }),
        gas_unit_price: 1,
        expiration_time: Duration::from_secs(u64::max_value()),
    }
}

/// Creates and signs a script transaction.
fn make_script_transaction(
    exec: &FakeExecutor,
    config: &TransactionConfig,
    script: CompiledScript,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    script.serialize(&mut blob)?;
    let script = TransactionScript::new(blob, config.args.clone());

    let params = get_transaction_parameters(exec, config);
    Ok(RawTransaction::new_script(
        params.sender_addr,
        params.sequence_number,
        script,
        params.max_gas_amount,
        params.gas_unit_price,
        params.expiration_time,
    )
    .sign(params.privkey, params.pubkey.clone())?
    .into_inner())
}

/// Creates and signs a module transaction.
fn make_module_transaction(
    exec: &FakeExecutor,
    config: &TransactionConfig,
    module: CompiledModule,
) -> Result<SignedTransaction> {
    let mut blob = vec![];
    module.serialize(&mut blob)?;
    let module = TransactionModule::new(blob);

    let params = get_transaction_parameters(exec, config);
    Ok(RawTransaction::new_module(
        params.sender_addr,
        params.sequence_number,
        module,
        params.max_gas_amount,
        params.gas_unit_price,
        params.expiration_time,
    )
    .sign(params.privkey, params.pubkey.clone())?
    .into_inner())
}

/// Runs a single transaction using the fake executor.
fn run_transaction(
    exec: &mut FakeExecutor,
    transaction: SignedTransaction,
) -> Result<TransactionOutput> {
    let mut outputs = exec.execute_block(vec![transaction]).unwrap();
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
        unreachable!("transaction outputs size mismatch")
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

fn eval_transaction(
    exec: &mut FakeExecutor,
    deps: &mut Vec<VerifiedModule>,
    idx: usize,
    transaction: &Transaction,
    log: &mut EvaluationLog,
) -> Result<Status> {
    /// Unwrap the given results. Upon failure, logs the error and aborts.
    macro_rules! unwrap_or_abort {
        ($res: expr) => {{
            match $res {
                Ok(r) => r,
                Err(e) => {
                    log.append(EvaluationOutput::Error(Box::new(e)));
                    return Ok(Status::Failure);
                }
            }
        }};
    }

    let sender_addr = *transaction.config.sender.address();

    // Start processing a new transaction.
    log.append(EvaluationOutput::Transaction(idx));

    // stage 1: parse the script/module
    if transaction.config.is_stage_disabled(Stage::Parser) {
        return Ok(Status::Success);
    }
    log.append(EvaluationOutput::Stage(Stage::Parser));
    let parsed_script_or_module = unwrap_or_abort!(parse_script_or_module(&transaction.input));
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
                unwrap_or_abort!(compile_script(sender_addr, parsed_script, &*deps)).0;
            log.append(EvaluationOutput::Output(Box::new(
                OutputType::CompiledScript(compiled_script.clone()),
            )));

            // stage 3: verify the script
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Verifier));
            let compiled_script = match do_verify_script(compiled_script, &*deps) {
                Ok(script) => script.into_inner(),
                Err(errs) => {
                    for err in errs.into_iter() {
                        let err: Error = ErrorKind::VerificationError(err).into();
                        log.append(EvaluationOutput::Error(Box::new(err)));
                    }
                    return Ok(Status::Failure);
                }
            };

            // stage 4: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_script(&compiled_script));
            }

            // stage 5: execute the script
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let script_transaction =
                make_script_transaction(&exec, &transaction.config, compiled_script)?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, script_transaction));
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
                unwrap_or_abort!(compile_module(sender_addr, parsed_module, &*deps)).0;
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
            let compiled_module = match do_verify_module(compiled_module, &*deps) {
                Ok(module) => module.into_inner(),
                Err(errs) => {
                    for err in errs.into_iter() {
                        let err: Error = ErrorKind::VerificationError(err).into();
                        log.append(EvaluationOutput::Error(Box::new(err)));
                    }
                    return Ok(Status::Failure);
                }
            };

            // stage 4: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_module(&compiled_module));
            }

            // stage 5: publish the module
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let module_transaction =
                make_module_transaction(&exec, &transaction.config, compiled_module)?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, module_transaction));
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

    // Set up a fake executor with the genesis block and create the accounts.
    let mut exec = if config.validator_set.is_empty() {
        // use the default validator set. this uses a precomputed validator set and is cheap
        FakeExecutor::from_genesis_with_options(VMPublishingOption::Open)
    } else {
        // use custom validator set. this requires dynamically generating a new genesis tx and
        // is thus more expensive.
        FakeExecutor::from_validator_set(config.validator_set.clone(), VMPublishingOption::Open)
    };
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }

    // Get the standard library modules (compiled & verified).
    let mut deps = stdlib_modules().to_vec();

    for (idx, transaction) in transactions.iter().enumerate() {
        let status = eval_transaction(&mut exec, &mut deps, idx, transaction, &mut log)?;
        log.append(EvaluationOutput::Status(status));
    }

    Ok(log)
}
