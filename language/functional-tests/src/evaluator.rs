// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiler::{Compiler, ScriptOrModule},
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::verifier::{
    verify_module_dependencies, verify_script_dependencies, VerifiedModule, VerifiedScript,
};
use language_e2e_tests::executor::FakeExecutor;
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::LBR_NAME,
    block_metadata::BlockMetadata,
    language_storage::ModuleId,
    on_chain_config::VMPublishingOption,
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript,
        SignedTransaction, Transaction as LibraTransaction, TransactionOutput, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
};
use mirai_annotations::checked_verify;
use std::{
    fmt::{self, Debug},
    str::FromStr,
    time::Duration,
};
use stdlib::{stdlib_modules, StdLibOptions};
use vm::{
    file_format::{CompiledModule, CompiledScript},
    gas_schedule::{GasAlgebra, MAXIMUM_NUMBER_OF_GAS_UNITS},
    views::ModuleView,
};

/// A transaction to be evaluated by the testing infra.
/// Contains code and a transaction config.
#[derive(Debug)]
pub struct Transaction<'a> {
    pub config: TransactionConfig<'a>,
    pub input: String,
}

/// Commands that drives the operation of LibraVM. Such as:
/// 1. Execute user transaction
/// 2. Publish a new block metadata
///
/// In the future we will add more commands to mimic the full public API of LibraVM,
/// including reloading the on-chain configuration that will affect the code path for LibraVM,
/// cleaning the cache in the LibraVM, etc.
#[derive(Debug)]
pub enum Command<'a> {
    Transaction(Transaction<'a>),
    BlockMetadata(BlockMetadata),
}

/// Indicates one step in the pipeline the given move module/program goes through.
//  Ord is derived as we need to be able to determine if one stage is before another.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Stage {
    Compiler,
    Verifier,
    Serializer,
    Runtime,
}

impl FromStr for Stage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
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
    CompiledModule(Box<CompiledModule>),
    CompiledScript(Box<CompiledScript>),
    CompilerLog(String),
    TransactionOutput(Box<TransactionOutput>),
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
    Output(OutputType),
    Error(Box<Error>),
    Status(Status),
}

impl EvaluationOutput {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

/// A log consisting of outputs from all stages and the final status.
/// This is checked against the directives.
#[derive(Debug, Default)]
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
            CompilerLog(s) => write!(f, "{}", s),
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

fn fetch_script_dependencies(
    exec: &mut FakeExecutor,
    script: &CompiledScript,
) -> Vec<VerifiedModule> {
    let module = script.clone().into_module();
    let idents = ModuleView::new(&module)
        .module_handles()
        .map(|handle_view| handle_view.module_id());
    fetch_dependencies(exec, idents)
}

fn fetch_module_dependencies(
    exec: &mut FakeExecutor,
    module: &CompiledModule,
) -> Vec<VerifiedModule> {
    let idents = ModuleView::new(module)
        .module_handles()
        .map(|handle_view| handle_view.module_id());
    fetch_dependencies(exec, idents)
}

fn fetch_dependencies(
    exec: &mut FakeExecutor,
    idents: impl Iterator<Item = ModuleId>,
) -> Vec<VerifiedModule> {
    // idents.into_inner().
    idents
        .flat_map(|ident| fetch_dependency(exec, ident))
        .collect()
}

fn fetch_dependency(exec: &mut FakeExecutor, ident: ModuleId) -> Option<VerifiedModule> {
    let ap = AccessPath::from(&ident);
    let blob: Vec<u8> = exec.get_state_view().get(&ap).ok().flatten()?;
    let compiled: CompiledModule = CompiledModule::deserialize(&blob).ok()?;
    VerifiedModule::new(compiled).ok()
}

/// Verify a script with its dependencies.
pub fn verify_script(
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

/// Verify a module with its dependencies.
pub fn verify_module(
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
    let account_balance = exec.read_balance_resource(config.sender).unwrap();

    TransactionParameters {
        sender_addr: *config.sender.address(),
        pubkey: &config.sender.pubkey,
        privkey: &config.sender.privkey,
        sequence_number: config
            .sequence_number
            .unwrap_or_else(|| account_resource.sequence_number()),
        max_gas_amount: config.max_gas.unwrap_or_else(|| {
            std::cmp::min(MAXIMUM_NUMBER_OF_GAS_UNITS.get(), account_balance.coin())
        }),
        gas_unit_price: config.gas_price.unwrap_or(1),
        // TTL is 86400s. Initial time was set to 0.
        expiration_time: config
            .expiration_time
            .unwrap_or_else(|| Duration::from_secs(40000)),
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
    let script = TransactionScript::new(blob, config.ty_args.clone(), config.args.clone());

    let params = get_transaction_parameters(exec, config);
    Ok(RawTransaction::new_script(
        params.sender_addr,
        params.sequence_number,
        script,
        params.max_gas_amount,
        params.gas_unit_price,
        LBR_NAME.to_string(),
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
        LBR_NAME.to_string(),
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
            TransactionStatus::Discard(_) | TransactionStatus::Retry => {
                checked_verify!(output.write_set().is_empty());
                Err(ErrorKind::DiscardedTransaction(output).into())
            }
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

fn eval_transaction<TComp: Compiler>(
    compiler: &mut TComp,
    exec: &mut FakeExecutor,
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

    // stage 1: Compile the script/module
    if transaction.config.is_stage_disabled(Stage::Compiler) {
        return Ok(Status::Success);
    }
    log.append(EvaluationOutput::Stage(Stage::Compiler));
    let compiler_log = |s| log.append(EvaluationOutput::Output(OutputType::CompilerLog(s)));

    let parsed_script_or_module =
        unwrap_or_abort!(compiler.compile(compiler_log, sender_addr, &transaction.input));

    match parsed_script_or_module {
        ScriptOrModule::Script(compiled_script) => {
            log.append(EvaluationOutput::Output(OutputType::CompiledScript(
                Box::new(compiled_script.clone()),
            )));

            // stage 2: verify the script
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Verifier));
            let deps = fetch_script_dependencies(exec, &compiled_script);
            let compiled_script = match verify_script(compiled_script, &deps) {
                Ok(script) => script.into_inner(),
                Err(errs) => {
                    for err in errs.into_iter() {
                        let err: Error = ErrorKind::VerificationError(err).into();
                        log.append(EvaluationOutput::Error(Box::new(err)));
                    }
                    return Ok(Status::Failure);
                }
            };

            // stage 3: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_script(&compiled_script));
            }

            // stage 4: execute the script
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let script_transaction =
                make_script_transaction(&exec, &transaction.config, compiled_script)?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, script_transaction));
            log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                Box::new(txn_output),
            )));
        }
        ScriptOrModule::Module(compiled_module) => {
            log.append(EvaluationOutput::Output(OutputType::CompiledModule(
                Box::new(compiled_module.clone()),
            )));

            // stage 2: verify the module
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Verifier));
            let deps = fetch_module_dependencies(exec, &compiled_module);
            let compiled_module = match verify_module(compiled_module, &deps) {
                Ok(module) => module.into_inner(),
                Err(errs) => {
                    for err in errs.into_iter() {
                        let err: Error = ErrorKind::VerificationError(err).into();
                        log.append(EvaluationOutput::Error(Box::new(err)));
                    }
                    return Ok(Status::Failure);
                }
            };

            // stage 3: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                log.append(EvaluationOutput::Stage(Stage::Serializer));
                unwrap_or_abort!(serialize_and_deserialize_module(&compiled_module));
            }

            // stage 4: publish the module
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            log.append(EvaluationOutput::Stage(Stage::Runtime));
            let module_transaction =
                make_module_transaction(&exec, &transaction.config, compiled_module)?;
            let txn_output = unwrap_or_abort!(run_transaction(exec, module_transaction));
            log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                Box::new(txn_output),
            )));
        }
    }
    Ok(Status::Success)
}

pub fn eval_block_metadata(
    executor: &mut FakeExecutor,
    block_metadata: BlockMetadata,
    log: &mut EvaluationLog,
) -> Result<Status> {
    let outputs =
        executor.execute_transaction_block(vec![LibraTransaction::BlockMetadata(block_metadata)]);

    match outputs {
        Ok(mut outputs) => {
            let output = outputs
                .pop()
                .expect("There should be one output in the result");
            executor.apply_write_set(output.write_set());
            log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                Box::new(output),
            )));
            Ok(Status::Success)
        }
        Err(err) => {
            let err: Error = ErrorKind::VerificationError(err).into();
            log.append(EvaluationOutput::Error(Box::new(err)));
            Ok(Status::Failure)
        }
    }
}

/// Feeds all given transactions through the pipeline and produces an EvaluationLog.
pub fn eval<TComp: Compiler>(
    config: &GlobalConfig,
    mut compiler: TComp,
    commands: &[Command],
) -> Result<EvaluationLog> {
    let mut log = EvaluationLog { outputs: vec![] };

    // Set up a fake executor with the genesis block and create the accounts.
    let mut exec = if config.validator_set.payload().is_empty() {
        if compiler.use_staged_genesis() {
            FakeExecutor::from_genesis_file()
        } else {
            FakeExecutor::from_fresh_genesis()
        }
    } else {
        // use custom validator set. this requires dynamically generating a new genesis tx and
        // is thus more expensive.
        FakeExecutor::custom_genesis(
            stdlib_modules(if compiler.use_staged_genesis() {
                StdLibOptions::Staged
            } else {
                StdLibOptions::Fresh
            })
            .to_vec(),
            Some(config.validator_set.clone()),
            VMPublishingOption::Open,
        )
    };
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }

    for (idx, command) in commands.iter().enumerate() {
        match command {
            Command::Transaction(transaction) => {
                let status =
                    eval_transaction(&mut compiler, &mut exec, idx, transaction, &mut log)?;
                log.append(EvaluationOutput::Status(status));
            }
            Command::BlockMetadata(block_metadata) => {
                let status = eval_block_metadata(&mut exec, block_metadata.clone(), &mut log)?;
                log.append(EvaluationOutput::Status(status));
            }
        }
    }

    Ok(log)
}
