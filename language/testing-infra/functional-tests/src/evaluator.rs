// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compiler::{Compiler, ScriptOrModule},
    config::{global::Config as GlobalConfig, transaction::Config as TransactionConfig},
    errors::*,
};
use bytecode_verifier::{cyclic_dependencies, dependencies};
use diem_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::{CORE_CODE_ADDRESS, XUS_NAME},
    block_metadata::BlockMetadata,
    chain_id::ChainId,
    on_chain_config::VMPublishingOption,
    transaction::{
        Module as TransactionModule, RawTransaction, Script as TransactionScript, ScriptFunction,
        SignedTransaction, Transaction as DiemTransaction, TransactionOutput, TransactionPayload,
        TransactionStatus,
    },
    vm_status::{KeptVMStatus, StatusCode},
};
use language_e2e_tests::{data_store::FakeDataStore, executor::FakeExecutor};
use mirai_annotations::checked_verify;
use move_binary_format::{
    access::ModuleAccess,
    errors::{Location, PartialVMError, VMError},
    file_format::{CompiledModule, CompiledScript},
    views::ModuleView,
};
use move_core_types::{
    gas_schedule::{GasAlgebra, GasConstants},
    identifier::Identifier,
    language_storage::ModuleId,
    transaction_argument,
};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::{
    fmt::{self, Debug},
    str::FromStr,
};

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
    data_store: &FakeDataStore,
    script: &CompiledScript,
) -> Vec<CompiledModule> {
    let idents = script.module_handles.iter().map(|handle| {
        ModuleId::new(
            script.address_identifiers[handle.address.0 as usize],
            script.identifiers[handle.name.0 as usize].clone(),
        )
    });
    fetch_dependencies(data_store, idents)
}

fn fetch_module_dependencies(
    data_store: &FakeDataStore,
    module: &CompiledModule,
) -> Vec<CompiledModule> {
    let idents = ModuleView::new(module)
        .module_handles()
        .map(|handle_view| handle_view.module_id());
    fetch_dependencies(data_store, idents)
}

fn fetch_dependencies(
    data_store: &FakeDataStore,
    idents: impl Iterator<Item = ModuleId>,
) -> Vec<CompiledModule> {
    // idents.into_inner().
    idents
        .flat_map(|ident| fetch_dependency(data_store, ident))
        .collect()
}

fn fetch_dependency(data_store: &FakeDataStore, ident: ModuleId) -> Option<CompiledModule> {
    let ap = AccessPath::from(&ident);
    let blob: Vec<u8> = data_store.get(&ap).ok().flatten()?;
    let compiled: CompiledModule = CompiledModule::deserialize(&blob).ok()?;
    match bytecode_verifier::verify_module(&compiled) {
        Ok(_) => Some(compiled),
        Err(_) => None,
    }
}

/// Verify a script with its dependencies.
pub fn verify_script(
    script: CompiledScript,
    data_store: &FakeDataStore,
) -> std::result::Result<CompiledScript, VMError> {
    bytecode_verifier::verify_script(&script)?;

    let imm_deps = fetch_script_dependencies(data_store, &script);
    dependencies::verify_script(&script, &imm_deps)?;

    Ok(script)
}

/// Verify a module with its dependencies.
pub fn verify_module(
    module: CompiledModule,
    data_store: &FakeDataStore,
) -> std::result::Result<CompiledModule, VMError> {
    bytecode_verifier::verify_module(&module)?;

    let imm_deps = fetch_module_dependencies(data_store, &module);
    dependencies::verify_module(&module, &imm_deps)?;

    cyclic_dependencies::verify_module(
        &module,
        |module_id| {
            fetch_dependency(data_store, module_id.clone())
                .map(|module| module.immediate_dependencies())
                .ok_or_else(|| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
        },
        |module_id| {
            fetch_dependency(data_store, module_id.clone())
                .map(|module| module.immediate_friends())
                .ok_or_else(|| PartialVMError::new(StatusCode::MISSING_DEPENDENCY))
        },
    )?;

    Ok(module)
}

#[allow(dead_code)]
/// A set of common parameters required to create transactions.
struct TransactionParameters<'a> {
    pub sender_addr: AccountAddress,
    pub secondary_signer_addresses: Vec<AccountAddress>,
    pub pubkey: &'a Ed25519PublicKey,
    pub privkey: &'a Ed25519PrivateKey,
    pub _secondary_pubkeys: Vec<Ed25519PublicKey>,
    pub secondary_privkeys: Vec<&'a Ed25519PrivateKey>,
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
        secondary_signer_addresses: config
            .secondary_signers
            .iter()
            .map(|signer| *signer.address())
            .collect(),
        pubkey: &config.sender.pubkey,
        privkey: &config.sender.privkey,
        _secondary_pubkeys: config
            .secondary_signers
            .iter()
            .map(|signer| signer.pubkey.clone())
            .collect(),
        secondary_privkeys: config
            .secondary_signers
            .iter()
            .map(|signer| &signer.privkey)
            .collect(),
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
    blob: Vec<u8>,
) -> Result<SignedTransaction> {
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
        .sign(params.privkey, params.pubkey.clone())?
    } else if config.secondary_signers.is_empty() {
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
        .sign(params.privkey, params.pubkey.clone())?
    } else {
        RawTransaction::new(
            params.sender_addr,
            params.sequence_number,
            TransactionPayload::Script(script),
            params.max_gas_amount,
            params.gas_unit_price,
            params.gas_currency_code,
            params.expiration_timestamp_secs,
            ChainId::test(),
        )
        .sign_multi_agent(
            params.privkey,
            params.secondary_signer_addresses,
            params.secondary_privkeys,
        )?
    }
    .into_inner())
}

/// Creates and signs a script transaction.
fn make_script_function_transaction(
    exec: &FakeExecutor,
    config: &TransactionConfig,
    module: ModuleId,
    function: Identifier,
) -> Result<SignedTransaction> {
    let script_function = ScriptFunction::new(
        module,
        function,
        config.ty_args.clone(),
        transaction_argument::convert_txn_args(&config.args),
    );

    let params = get_transaction_parameters(exec, config);
    Ok(RawTransaction::new_script_function(
        params.sender_addr,
        params.sequence_number,
        script_function,
        params.max_gas_amount,
        params.gas_unit_price,
        params.gas_currency_code,
        params.expiration_timestamp_secs,
        ChainId::test(),
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
) {
    let mut outputs = exec
        .execute_block_and_keep_vm_status(vec![transaction])
        .unwrap();
    let (vm_status, txn_output) = match outputs.pop() {
        Some(x) => x,
        None => unreachable!("{}", "expected 1 output got {}"),
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

fn is_script_function(input_str: &str) -> Option<(ModuleId, Identifier)> {
    let stdlib_script_regex = Regex::new(r"stdlib_script::(\w+)::(\w+)\s*").unwrap();
    let captures_opt = stdlib_script_regex.captures(input_str);
    captures_opt.and_then(|captures| {
        let module = captures.get(1)?.as_str();
        let function = captures.get(2)?.as_str();
        let module_id = ModuleId::new(CORE_CODE_ADDRESS, Identifier::new(module).ok()?);
        let function_ident = Identifier::new(function).ok()?;
        Some((module_id, function_ident))
    })
}

fn eval_transaction<TComp: Compiler>(
    global_config: &GlobalConfig,
    compiler: &mut TComp,
    exec: &mut FakeExecutor,
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
    // stage 1: Compile the script/module
    if transaction.config.is_stage_disabled(Stage::Compiler) {
        return Ok(Status::Success);
    }
    if !global_config.exp_mode {
        log.append(EvaluationOutput::Stage(Stage::Compiler));
    }

    if let Some((module, function)) = is_script_function(&transaction.input) {
        // execute the script
        if transaction.config.is_stage_disabled(Stage::Runtime) {
            return Ok(Status::Success);
        }
        if !global_config.exp_mode {
            log.append(EvaluationOutput::Stage(Stage::Runtime));
        }
        let script_function_transaction =
            make_script_function_transaction(exec, &transaction.config, module, function)?;

        if global_config.exp_mode {
            run_transaction_exp_mode(exec, script_function_transaction, log, &transaction.config);
        } else {
            let txn_output = unwrap_or_abort!(run_transaction(exec, script_function_transaction));
            log.append(EvaluationOutput::Output(OutputType::TransactionOutput(
                Box::new(txn_output),
            )));
        }

        return Ok(Status::Success);
    }

    let parsed_script_or_module = {
        let compiler_log = |s| {
            if !global_config.exp_mode {
                log.append(EvaluationOutput::Output(OutputType::CompilerLog(s)));
            }
        };

        // Compiler error messages may contain temporary file names, causing tests to be non-deterministic.
        // The following code get them filtered out using regex.
        static FILENAME_PAT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"[A-Za-z0-9_/.]+:([0-9]+):([0-9]+)").unwrap());

        let compiler_res = compiler.compile(compiler_log, sender_addr, &transaction.input);
        let filtered_compiler_res = compiler_res.map_err(|err| {
            let msg = err.to_string();
            let filtered = FILENAME_PAT
                .replace_all(&msg, |caps: &Captures| {
                    format!("(file):{}:{}", &caps[1], &caps[2])
                })
                .to_string();
            format_err!("{}", filtered)
        });
        unwrap_or_abort!(filtered_compiler_res)
    };

    match parsed_script_or_module {
        ScriptOrModule::Script(bytes_opt, compiled_script) => {
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
            let compiled_script = match verify_script(compiled_script, exec.get_state_view()) {
                Ok(script) => script,
                Err(err) => {
                    let err: Error = ErrorKind::VerificationError(err.into_vm_status()).into();
                    log.append(EvaluationOutput::Error(Box::new(err)));
                    return Ok(Status::Failure);
                }
            };

            // stage 3: serializer round trip
            if bytes_opt.is_none() && !transaction.config.is_stage_disabled(Stage::Serializer) {
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
            let bytes = match bytes_opt {
                Some(bytes) => bytes,
                None => {
                    let mut bytes = vec![];
                    compiled_script.serialize(&mut bytes).unwrap();
                    bytes
                }
            };
            let script_transaction = make_script_transaction(exec, &transaction.config, bytes)?;

            if global_config.exp_mode {
                run_transaction_exp_mode(exec, script_transaction, log, &transaction.config);
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
            let compiled_module = match verify_module(compiled_module, exec.get_state_view()) {
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
                make_module_transaction(exec, &transaction.config, compiled_module)?;

            if global_config.exp_mode {
                run_transaction_exp_mode(exec, module_transaction, log, &transaction.config);
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
        let module_blobs = if compiler.use_compiled_genesis() {
            diem_framework_releases::current_module_blobs()
        } else {
            diem_framework::module_blobs()
        };
        // use custom validator set. this requires dynamically generating a new genesis tx and
        // is thus more expensive.
        FakeExecutor::custom_genesis(
            module_blobs,
            Some(config.validator_accounts),
            VMPublishingOption::open(),
        )
    };
    for data in config.accounts.values() {
        exec.add_account_data(data);
    }

    for (idx, command) in commands.iter().enumerate() {
        log.append(EvaluationOutput::Transaction(idx));
        match command {
            Command::Transaction(transaction) => {
                let status =
                    eval_transaction(config, &mut compiler, &mut exec, transaction, &mut log)?;
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
