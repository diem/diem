// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiler::{Compiler, ScriptOrModule},
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::DependencyChecker;
use compiled_stdlib::{stdlib_modules, transaction_scripts::StdlibScript, StdLibOptions};
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::XUS_NAME,
    block_metadata::BlockMetadata,
    chain_id::ChainId,
    on_chain_config::VMPublishingOption,
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript,
        SignedTransaction, Transaction as DiemTransaction, TransactionOutput, TransactionStatus,
    },
    vm_status::KeptVMStatus,
};
use language_e2e_tests::executor::FakeExecutor;
use mirai_annotations::checked_verify;
use move_core_types::{
    gas_schedule::{GasAlgebra, GasConstants},
    language_storage::ModuleId,
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
    str::FromStr,
};
use vm::{
    errors::{Location, VMError},
    file_format::{CompiledModule, CompiledScript},
    views::ModuleView,
};

static PRECOMPILED_TXN_SCRIPTS: Lazy<HashMap<String, CompiledScript>> = Lazy::new(|| {
    StdlibScript::all()
        .into_iter()
        .map(|script| {
            let name = script.name();
            let bytes = script.compiled_bytes().into_vec();
            let compiled_script = CompiledScript::deserialize(&bytes).unwrap();
            (name, compiled_script)
        })
        .collect::<HashMap<String, CompiledScript>>()
});

/// A transaction to be evaluated by the testing infra.
/// Contains code and a transaction config.
#[derive(Debug)]
pub struct Transaction<'a> {
    pub config: TransactionConfig<'a>,
    pub input: String,
}

/// Commands that drives the operation of DiemVM. Such as:
/// 1. Execute user transaction
/// 2. Publish a new block metadata
///
/// In the future we will add more commands to mimic the full public API of DiemVM,
/// including reloading the on-chain configuration that will affect the code path for DiemVM,
/// cleaning the cache in the DiemVM, etc.
#[derive(Debug)]
pub enum Command<'a> {
    Transaction(Transaction<'a>),
    BlockMetadata(BlockMetadata),
}

/// Indicates one step in the pipeline the given Move module/program goes through.
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
    PlainMessage(String),
}

impl EvaluationOutput {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn to_string_for_matching(&self) -> String {
        match self {
            EvaluationOutput::Error(e) => format!("{:?}", e.root_cause()),
            EvaluationOutput::PlainMessage(s) => s.to_string(),
            _ => format!("{:?}", self),
        }
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

    pub fn to_text_for_matching(&self) -> Vec<String> {
        self.outputs
            .iter()
            .map(EvaluationOutput::to_string_for_matching)
            .collect()
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
            PlainMessage(s) => write!(f, "{}", s),
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
) -> Vec<CompiledModule> {
    let inner = script.as_inner();
    let idents = inner.module_handles.iter().map(|handle| {
        ModuleId::new(
            inner.address_identifiers[handle.address.0 as usize],
            inner.identifiers[handle.name.0 as usize].clone(),
        )
    });
    fetch_dependencies(exec, idents)
}

fn fetch_module_dependencies(
    exec: &mut FakeExecutor,
    module: &CompiledModule,
) -> Vec<CompiledModule> {
    let idents = ModuleView::new(module)
        .module_handles()
        .map(|handle_view| handle_view.module_id());
    fetch_dependencies(exec, idents)
}

fn fetch_dependencies(
    exec: &mut FakeExecutor,
    idents: impl Iterator<Item = ModuleId>,
) -> Vec<CompiledModule> {
    // idents.into_inner().
    idents
        .flat_map(|ident| fetch_dependency(exec, ident))
        .collect()
}

fn fetch_dependency(exec: &mut FakeExecutor, ident: ModuleId) -> Option<CompiledModule> {
    let ap = AccessPath::from(&ident);
    let blob: Vec<u8> = exec.get_state_view().get(&ap).ok().flatten()?;
    let compiled: CompiledModule = CompiledModule::deserialize(&blob).ok()?;
    match bytecode_verifier::verify_module(&compiled) {
        Ok(_) => Some(compiled),
        Err(_) => None,
    }
}

/// Verify a script with its dependencies.
pub fn verify_script(
    script: CompiledScript,
    deps: &[CompiledModule],
) -> std::result::Result<CompiledScript, VMError> {
    bytecode_verifier::verify_script(&script)?;
    DependencyChecker::verify_script(&script, deps)?;
    Ok(script)
}

/// Verify a module with its dependencies.
pub fn verify_module(
    module: CompiledModule,
    deps: &[CompiledModule],
) -> std::result::Result<CompiledModule, VMError> {
    bytecode_verifier::verify_module(&module)?;
    DependencyChecker::verify_module(&module, deps)?;
    Ok(module)
}

/// A set of common parameters required to create transactions.
struct TransactionParameters<'a> {
    pub sender_addr: AccountAddress,
    pub pubkey: &'a Ed25519PublicKey,
    pub privkey: &'a Ed25519PrivateKey,
    pub sequence_number: u64,
    pub max_gas_amount: u64,
    pub gas_unit_price: u64,
    pub gas_currency_code: String,
    pub expiration_timestamp_secs: u64,
}

/// Gets the transaction parameters from the current execution environment and the config.
fn get_transaction_parameters<'a>(
    exec: &'a FakeExecutor,
    config: &'a TransactionConfig,
) -> TransactionParameters<'a> {
    let account_resource = exec.read_account_resource(config.sender).unwrap();
    let gas_unit_price = config.gas_price.unwrap_or(0);
    let gas_currency_code = config
        .gas_currency_code
        .clone()
        .unwrap_or_else(|| XUS_NAME.to_owned());
    let max_number_of_gas_units = GasConstants::default().maximum_number_of_gas_units;
    let max_gas_amount = config.max_gas.unwrap_or_else(|| {
        if gas_unit_price == 0 {
            max_number_of_gas_units.get()
        } else {
            let account_balance = exec
                .read_balance_resource(
                    config.sender,
                    account_config::from_currency_code_string(&gas_currency_code).unwrap(),
                )
                .unwrap_or_else(|| panic!(
                        "Couldn't read balance of type {:?} for account {:?}; did you forget to specify //! gas-currency: {:?} ?",
                        config.sender.address(),
                        gas_currency_code,
                        gas_currency_code
                ));
            std::cmp::min(
                max_number_of_gas_units.get(),
                account_balance.coin() / gas_unit_price,
            )
        }
    });

    TransactionParameters {
        sender_addr: *config.sender.address(),
        pubkey: &config.sender.pubkey,
        privkey: &config.sender.privkey,
        sequence_number: config
            .sequence_number
            .unwrap_or_else(|| account_resource.sequence_number()),
        max_gas_amount,
        gas_unit_price,
        gas_currency_code,
        // TTL is 86400s. Initial time was set to 0.
        expiration_timestamp_secs: config.expiration_timestamp_secs.unwrap_or(40000),
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
    Ok(if let Some(execute_as) = config.execute_as {
        RawTransaction::new_writeset_script(
            params.sender_addr,
            params.sequence_number,
            script,
            execute_as,
            ChainId::test(),
        )
    } else {
        RawTransaction::new_script(
            params.sender_addr,
            params.sequence_number,
            script,
            params.max_gas_amount,
            params.gas_unit_price,
            params.gas_currency_code,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
    }
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
        params.gas_currency_code,
        params.expiration_timestamp_secs,
        ChainId::test(),
    )
    .sign(params.privkey, params.pubkey.clone())?
    .into_inner())
}

/// Runs a single transaction using the fake executor.
fn run_transaction(
    exec: &mut FakeExecutor,
    transaction: SignedTransaction,
) -> Result<TransactionOutput> {
    let mut outputs = exec
        .execute_block_and_keep_vm_status(vec![transaction])
        .unwrap();
    if outputs.len() == 1 {
        let (vm_status, txn_output) = outputs.pop().unwrap();
        match txn_output.status() {
            TransactionStatus::Keep(status) => {
                exec.apply_write_set(txn_output.write_set());
                if status == &KeptVMStatus::Executed {
                    Ok(txn_output)
                } else {
                    Err(ErrorKind::VMExecutionFailure(vm_status, txn_output).into())
                }
            }
            TransactionStatus::Discard(_) | TransactionStatus::Retry => {
                checked_verify!(txn_output.write_set().is_empty());
                Err(ErrorKind::DiscardedTransaction(txn_output).into())
            }
        }
    } else {
        unreachable!("transaction outputs size mismatch")
    }
}

fn run_transaction_exp_mode(
    exec: &mut FakeExecutor,
    transaction: SignedTransaction,
    log: &mut EvaluationLog,
    config: &TransactionConfig,
) -> Result<()> {
    let mut outputs = exec
        .execute_block_and_keep_vm_status(vec![transaction])
        .unwrap();
    let (vm_status, txn_output) = match outputs.pop() {
        Some(x) => x,
        None => unreachable!("expected 1 output got {}"),
    };

    log.append(EvaluationOutput::PlainMessage(format!(
        "Move VM Status: {:?}",
        vm_status
    )));
    log.append(EvaluationOutput::PlainMessage(format!(
        "Transaction Status: {:?}",
        txn_output.status()
    )));
    if config.show_gas {
        log.append(EvaluationOutput::PlainMessage(format!(
            "Gas Used: {:?}",
            txn_output.gas_used(),
        )))
    }
    if config.show_writeset {
        log.append(EvaluationOutput::PlainMessage(format!(
            "Write Set: {:?}",
            txn_output.write_set(),
        )))
    }
    if config.show_events {
        log.append(EvaluationOutput::PlainMessage(format!(
            "Events: {:?}",
            txn_output.events(),
        )))
    }

    match txn_output.status() {
        TransactionStatus::Keep(_) => {
            exec.apply_write_set(txn_output.write_set());
        }
        TransactionStatus::Discard(_) | TransactionStatus::Retry => {
            checked_verify!(txn_output.write_set().is_empty());
        }
    }

    Ok(())
}

/// Serializes the script then deserializes it.
fn serialize_and_deserialize_script(script: &CompiledScript) -> Result<()> {
    let mut script_blob = vec![];
    script.serialize(&mut script_blob)?;
    let deserialized_script = CompiledScript::deserialize(&script_blob)
        .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;

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
    let deserialized_module = CompiledModule::deserialize(&module_blob)
        .map_err(|e| e.finish(Location::Undefined).into_vm_status())?;

    if *module != deserialized_module {
        return Err(ErrorKind::Other(
            "deserialized module different from original one".to_string(),
        )
        .into());
    }

    Ok(())
}

fn is_precompiled_script(input_str: &str) -> Option<CompiledScript> {
    if let Some(script_name) = input_str.strip_prefix("stdlib_script::") {
        return PRECOMPILED_TXN_SCRIPTS.get(script_name).cloned();
    }
    None
}

fn eval_transaction<TComp: Compiler>(
    global_config: &GlobalConfig,
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
    if !global_config.exp_mode {
        log.append(EvaluationOutput::Stage(Stage::Compiler));
    }

    let parsed_script_or_module =
        if let Some(compiled_script) = is_precompiled_script(&transaction.input) {
            ScriptOrModule::Script(compiled_script)
        } else {
            let compiler_log = |s| {
                if !global_config.exp_mode {
                    log.append(EvaluationOutput::Output(OutputType::CompilerLog(s)));
                }
            };
            unwrap_or_abort!(compiler.compile(compiler_log, sender_addr, &transaction.input))
        };

    match parsed_script_or_module {
        ScriptOrModule::Script(compiled_script) => {
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Output(OutputType::CompiledScript(
                    Box::new(compiled_script.clone()),
                )));
            }

            // stage 2: verify the script
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Stage(Stage::Verifier));
            }
            let deps = fetch_script_dependencies(exec, &compiled_script);
            let compiled_script = match verify_script(compiled_script, &deps) {
                Ok(script) => script,
                Err(err) => {
                    let err: Error = ErrorKind::VerificationError(err.into_vm_status()).into();
                    log.append(EvaluationOutput::Error(Box::new(err)));
                    return Ok(Status::Failure);
                }
            };

            // stage 3: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                if !global_config.exp_mode {
                    log.append(EvaluationOutput::Stage(Stage::Serializer));
                }
                unwrap_or_abort!(serialize_and_deserialize_script(&compiled_script));
            }

            // stage 4: execute the script
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Stage(Stage::Runtime));
            }
            let script_transaction =
                make_script_transaction(&exec, &transaction.config, compiled_script)?;

            if global_config.exp_mode {
                run_transaction_exp_mode(exec, script_transaction, log, &transaction.config)?;
            } else {
                let txn_output = unwrap_or_abort!(run_transaction(exec, script_transaction));
                log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                    Box::new(txn_output),
                )));
            }
        }
        ScriptOrModule::Module(compiled_module) => {
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Output(OutputType::CompiledModule(
                    Box::new(compiled_module.clone()),
                )));
            }

            // stage 2: verify the module
            if transaction.config.is_stage_disabled(Stage::Verifier) {
                return Ok(Status::Success);
            }
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Stage(Stage::Verifier));
            }
            let deps = fetch_module_dependencies(exec, &compiled_module);
            let compiled_module = match verify_module(compiled_module, &deps) {
                Ok(module) => module,
                Err(err) => {
                    let err: Error = ErrorKind::VerificationError(err.into_vm_status()).into();
                    log.append(EvaluationOutput::Error(Box::new(err)));
                    return Ok(Status::Failure);
                }
            };

            // stage 3: serializer round trip
            if !transaction.config.is_stage_disabled(Stage::Serializer) {
                if !global_config.exp_mode {
                    log.append(EvaluationOutput::Stage(Stage::Serializer));
                }
                unwrap_or_abort!(serialize_and_deserialize_module(&compiled_module));
            }

            // stage 4: publish the module
            if transaction.config.is_stage_disabled(Stage::Runtime) {
                return Ok(Status::Success);
            }
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Stage(Stage::Runtime));
            }
            let module_transaction =
                make_module_transaction(&exec, &transaction.config, compiled_module)?;

            if global_config.exp_mode {
                run_transaction_exp_mode(exec, module_transaction, log, &transaction.config)?;
            } else {
                let txn_output = unwrap_or_abort!(run_transaction(exec, module_transaction));
                log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                    Box::new(txn_output),
                )));
            }
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
        executor.execute_transaction_block(vec![DiemTransaction::BlockMetadata(block_metadata)]);

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
    let mut exec = if config.validator_accounts == 0 {
        if compiler.use_compiled_genesis() {
            FakeExecutor::from_genesis_file()
        } else {
            FakeExecutor::from_fresh_genesis()
        }
    } else {
        // use custom validator set. this requires dynamically generating a new genesis tx and
        // is thus more expensive.
        FakeExecutor::custom_genesis(
            stdlib_modules(if compiler.use_compiled_genesis() {
                StdLibOptions::Compiled
            } else {
                StdLibOptions::Fresh
            })
            .to_vec(),
            Some(config.validator_accounts),
            VMPublishingOption::open(),
        )
    };
    for data in config.accounts.values() {
        exec.add_account_data(&data);
    }

    for (idx, command) in commands.iter().enumerate() {
        match command {
            Command::Transaction(transaction) => {
                let status =
                    eval_transaction(config, &mut compiler, &mut exec, idx, transaction, &mut log)?;
                if !config.exp_mode {
                    log.append(EvaluationOutput::Status(status));
                }
            }
            Command::BlockMetadata(block_metadata) => {
                let status = eval_block_metadata(&mut exec, block_metadata.clone(), &mut log)?;
                if !config.exp_mode {
                    log.append(EvaluationOutput::Status(status));
                }
            }
        }
    }

    Ok(log)
}
