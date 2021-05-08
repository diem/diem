// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::{fs, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

use diem_types::{
    access_path::Path as AP,
    account_address::AccountAddress,
    account_config::{
        from_currency_code_string, reserved_vm_address, type_tag_for_currency_code, ACCOUNT_MODULE,
    },
    block_metadata::BlockMetadata,
    transaction::{
        Script, ScriptFunction, Transaction, TransactionArgument, TransactionOutput,
        TransactionPayload, TransactionStatus, WriteSetPayload,
    },
};
use diem_vm::{
    convert_changeset_and_events,
    script_to_script_function::remapping,
    system_module_names::{
        BLOCK_PROLOGUE, DIEM_BLOCK_MODULE, SCRIPT_PROLOGUE_NAME, USER_EPILOGUE_NAME,
        WRITESET_EPILOGUE_NAME, WRITESET_PROLOGUE_NAME,
    },
    transaction_metadata::TransactionMetadata,
};
use language_e2e_tests::{
    data_store::FakeDataStore,
    executor::{
        TraceSeqMapping, TRACE_DIR_DATA, TRACE_DIR_INPUT, TRACE_DIR_META, TRACE_DIR_OUTPUT,
        TRACE_FILE_ERROR, TRACE_FILE_NAME,
    },
};
use move_binary_format::errors::VMResult;
use move_core_types::{
    effects::{ChangeSet, Event},
    gas_schedule::GasAlgebra,
    identifier::IdentStr,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::{KeptVMStatus, VMStatus},
};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM, session::Session};
use move_vm_types::gas_schedule::GasStatus;

//**************************************************************************************************
// Utilities
//**************************************************************************************************

fn script_to_script_function(script: &Script) -> Option<ScriptFunction> {
    remapping(script.code()).map(|(module_id, func_name)| {
        ScriptFunction::new(
            module_id.clone(),
            func_name.to_owned(),
            script.ty_args().to_vec(),
            script
                .args()
                .iter()
                .map(|arg| {
                    match arg {
                        TransactionArgument::U8(i) => MoveValue::U8(*i),
                        TransactionArgument::U64(i) => MoveValue::U64(*i),
                        TransactionArgument::U128(i) => MoveValue::U128(*i),
                        TransactionArgument::Address(a) => MoveValue::Address(*a),
                        TransactionArgument::Bool(b) => MoveValue::Bool(*b),
                        TransactionArgument::U8Vector(v) => MoveValue::vector_u8(v.clone()),
                    }
                    .simple_serialize()
                    .unwrap()
                })
                .collect(),
        )
    })
}

fn compare_output(
    expect_output: &TransactionOutput,
    actual_output: VMResult<(ChangeSet, Vec<Event>)>,
) {
    match actual_output {
        Ok((change_set, events)) => {
            let (actual_write_set, actual_events) =
                convert_changeset_and_events(change_set, events).unwrap();
            assert_eq!(expect_output.write_set(), &actual_write_set);
            assert_eq!(expect_output.events(), &actual_events);
        }
        Err(err) => {
            assert!(expect_output.write_set().is_empty());
            assert!(expect_output.events().is_empty());
            let expect_status = expect_output.status();
            let actual_status = &err.into_vm_status();
            match (expect_status, actual_status) {
                (
                    TransactionStatus::Keep(KeptVMStatus::MoveAbort(expect_loc, expect_code)),
                    VMStatus::MoveAbort(actual_loc, actual_code),
                ) => {
                    assert_eq!(expect_loc, actual_loc);
                    assert_eq!(expect_code, actual_code);
                }
                (
                    TransactionStatus::Keep(KeptVMStatus::ExecutionFailure {
                        location: expect_loc,
                        function: expect_func,
                        code_offset: expect_offset,
                    }),
                    VMStatus::ExecutionFailure {
                        status_code: _,
                        location: actual_loc,
                        function: actual_func,
                        code_offset: actual_offset,
                    },
                ) => {
                    assert_eq!(expect_loc, actual_loc);
                    assert_eq!(expect_func, actual_func);
                    assert_eq!(expect_offset, actual_offset);
                }
                _ => panic!(
                    "Execution error does not match\nExpect: {:?}\nActual: {}",
                    expect_status, actual_status
                ),
            }
        }
    }
}

//**************************************************************************************************
// Executors
//**************************************************************************************************

fn execute_function_via_move_vm(
    session: &mut Session<FakeDataStore>,
    module_id: &ModuleId,
    function_name: &IdentStr,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
) -> VMResult<Vec<Vec<u8>>> {
    let mut gas_status = GasStatus::new_unmetered();
    let log_context = NoContextLog::new();
    session.execute_function(
        module_id,
        function_name,
        ty_args,
        args,
        &mut gas_status,
        &log_context,
    )
}

fn execute_script_function_via_move_vm(
    session: &mut Session<FakeDataStore>,
    module_id: &ModuleId,
    function_name: &IdentStr,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
    senders: Vec<AccountAddress>,
) -> VMResult<()> {
    let mut gas_status = GasStatus::new_unmetered();
    let log_context = NoContextLog::new();
    session.execute_script_function(
        module_id,
        function_name,
        ty_args,
        args,
        senders,
        &mut gas_status,
        &log_context,
    )
}

//**************************************************************************************************
// Transaction replay
//**************************************************************************************************

fn replay_txn_block_metadata(
    block_metadata: BlockMetadata,
    data_store: &FakeDataStore,
    expect_output: &TransactionOutput,
) {
    let move_vm = MoveVM::new();
    let mut session = move_vm.new_session(data_store);

    // args
    let signer = reserved_vm_address();
    let (round, timestamp, previous_votes, proposer) = block_metadata.into_inner();
    let args = vec![
        MoveValue::Signer(signer),
        MoveValue::U64(round),
        MoveValue::U64(timestamp),
        MoveValue::Vector(previous_votes.into_iter().map(MoveValue::Address).collect()),
        MoveValue::Address(proposer),
    ]
    .into_iter()
    .map(|v| v.simple_serialize().unwrap())
    .collect();

    // execute
    let result = execute_function_via_move_vm(
        &mut session,
        &*DIEM_BLOCK_MODULE,
        &*BLOCK_PROLOGUE,
        vec![],
        args,
    );
    let actual_output = result.and_then(|rets| {
        assert!(rets.is_empty());
        session.finish()
    });

    // compare
    compare_output(expect_output, actual_output);
}

fn execute_txn_user_script_prologue(
    session: &mut Session<FakeDataStore>,
    txn_meta: &TransactionMetadata,
    gas_currency_ty: &TypeTag,
) -> VMResult<()> {
    let TransactionMetadata {
        sender,
        authentication_key_preimage,
        sequence_number,
        max_gas_amount,
        gas_unit_price,
        expiration_timestamp_secs,
        chain_id,
        script_hash,
        ..
    } = txn_meta;
    let args = vec![
        MoveValue::Signer(*sender),
        MoveValue::U64(*sequence_number),
        MoveValue::vector_u8(authentication_key_preimage.clone()),
        MoveValue::U64(gas_unit_price.get()),
        MoveValue::U64(max_gas_amount.get()),
        MoveValue::U64(*expiration_timestamp_secs),
        MoveValue::U8(chain_id.id()),
        MoveValue::vector_u8(script_hash.clone()),
    ]
    .into_iter()
    .map(|v| v.simple_serialize().unwrap())
    .collect();

    let result = execute_function_via_move_vm(
        session,
        &*ACCOUNT_MODULE,
        &*SCRIPT_PROLOGUE_NAME,
        vec![gas_currency_ty.clone()],
        args,
    );
    result.map(|rets| assert!(rets.is_empty()))
}

fn execute_txn_user_script_epilogue(
    session: &mut Session<FakeDataStore>,
    txn_meta: &TransactionMetadata,
    gas_currency_ty: &TypeTag,
    gas_usage: u64,
) -> VMResult<()> {
    let TransactionMetadata {
        sender,
        sequence_number,
        max_gas_amount,
        gas_unit_price,
        ..
    } = txn_meta;
    let gas_remaining = max_gas_amount.get() - gas_usage;
    let args = vec![
        MoveValue::Signer(*sender),
        MoveValue::U64(*sequence_number),
        MoveValue::U64(gas_unit_price.get()),
        MoveValue::U64(max_gas_amount.get()),
        MoveValue::U64(gas_remaining),
    ]
    .into_iter()
    .map(|v| v.simple_serialize().unwrap())
    .collect();

    let result = execute_function_via_move_vm(
        session,
        &*ACCOUNT_MODULE,
        &*USER_EPILOGUE_NAME,
        vec![gas_currency_ty.clone()],
        args,
    );
    result.map(|rets| assert!(rets.is_empty()))
}

fn execute_txn_admin_script_prologue(
    session: &mut Session<FakeDataStore>,
    txn_meta: &TransactionMetadata,
) -> VMResult<()> {
    let TransactionMetadata {
        sender,
        authentication_key_preimage,
        sequence_number,
        expiration_timestamp_secs,
        chain_id,
        ..
    } = txn_meta;
    let args = vec![
        MoveValue::Signer(*sender),
        MoveValue::U64(*sequence_number),
        MoveValue::vector_u8(authentication_key_preimage.clone()),
        MoveValue::U64(*expiration_timestamp_secs),
        MoveValue::U8(chain_id.id()),
    ]
    .into_iter()
    .map(|v| v.simple_serialize().unwrap())
    .collect();

    let result = execute_function_via_move_vm(
        session,
        &*ACCOUNT_MODULE,
        &*WRITESET_PROLOGUE_NAME,
        vec![],
        args,
    );
    result.map(|rets| assert!(rets.is_empty()))
}

fn execute_txn_admin_script_epilogue(
    session: &mut Session<FakeDataStore>,
    txn_meta: &TransactionMetadata,
) -> VMResult<()> {
    let TransactionMetadata {
        sender,
        sequence_number,
        ..
    } = txn_meta;
    let args = vec![
        MoveValue::Signer(*sender),
        MoveValue::U64(*sequence_number),
        MoveValue::Bool(false), // admin script do not trigger reconfiguration
    ]
    .into_iter()
    .map(|v| v.simple_serialize().unwrap())
    .collect();

    let result = execute_function_via_move_vm(
        session,
        &*ACCOUNT_MODULE,
        &*WRITESET_EPILOGUE_NAME,
        vec![],
        args,
    );
    result.map(|rets| assert!(rets.is_empty()))
}

fn replay_txn_user_script_function_internal(
    senders: Vec<AccountAddress>,
    txn_meta: TransactionMetadata,
    script_fun: ScriptFunction,
    gas_currency: &str,
    gas_usage: u64,
    data_store: &FakeDataStore,
) -> VMResult<(ChangeSet, Vec<Event>)> {
    let gas_currency_ty =
        type_tag_for_currency_code(from_currency_code_string(gas_currency).unwrap());

    let move_vm = MoveVM::new();
    let mut session = move_vm.new_session(data_store);

    // prologue -> main -> epilogue
    execute_txn_user_script_prologue(&mut session, &txn_meta, &gas_currency_ty)?;

    let result = execute_script_function_via_move_vm(
        &mut session,
        script_fun.module(),
        script_fun.function(),
        script_fun.ty_args().to_vec(),
        script_fun.args().to_vec(),
        senders,
    );
    match result {
        Ok(_) => {
            execute_txn_user_script_epilogue(&mut session, &txn_meta, &gas_currency_ty, gas_usage)?;
            session.finish()
        }
        Err(err) => {
            let status = TransactionStatus::from(err.clone().into_vm_status());
            if status.is_discarded() {
                return Err(err);
            }
            let mut new_session = move_vm.new_session(data_store);
            execute_txn_user_script_epilogue(
                &mut new_session,
                &txn_meta,
                &gas_currency_ty,
                gas_usage,
            )?;
            new_session.finish()
        }
    }
}

fn replay_txn_admin_script_function_internal(
    senders: Vec<AccountAddress>,
    txn_meta: TransactionMetadata,
    script_fun: ScriptFunction,
    data_store: &FakeDataStore,
) -> VMResult<(ChangeSet, Vec<Event>)> {
    let move_vm = MoveVM::new();
    let mut session = move_vm.new_session(data_store);

    // prologue -> main -> epilogue
    execute_txn_admin_script_prologue(&mut session, &txn_meta)?;

    let result = execute_script_function_via_move_vm(
        &mut session,
        script_fun.module(),
        script_fun.function(),
        script_fun.ty_args().to_vec(),
        script_fun.args().to_vec(),
        senders,
    );
    match result {
        Ok(_) => {
            execute_txn_admin_script_epilogue(&mut session, &txn_meta)?;
            session.finish()
        }
        Err(err) => {
            let status = TransactionStatus::from(err.clone().into_vm_status());
            if status.is_discarded() {
                return Err(err);
            }
            let mut new_session = move_vm.new_session(data_store);
            execute_txn_admin_script_epilogue(&mut new_session, &txn_meta)?;
            new_session.finish()
        }
    }
}

fn replay_txn_script_function(
    is_admin: bool,
    senders: Vec<AccountAddress>,
    txn_meta: TransactionMetadata,
    script_fun: ScriptFunction,
    gas_currency: &str,
    data_store: &FakeDataStore,
    expect_output: &TransactionOutput,
) {
    // ignore out-of-gas cases
    if matches!(
        expect_output.status(),
        TransactionStatus::Keep(KeptVMStatus::OutOfGas)
    ) {
        return;
    }

    // execute
    let actual_output = if is_admin {
        replay_txn_admin_script_function_internal(senders, txn_meta, script_fun, data_store)
    } else {
        replay_txn_user_script_function_internal(
            senders,
            txn_meta,
            script_fun,
            gas_currency,
            expect_output.gas_used(),
            data_store,
        )
    };

    // compare
    compare_output(expect_output, actual_output);
}

//**************************************************************************************************
// Trace replay
//**************************************************************************************************

fn replay_trace<P: AsRef<Path>>(wks: P, verbose: bool) -> Result<()> {
    let wks = wks.as_ref();

    // sanity checks
    let test_name = fs::read_to_string(wks.join(TRACE_FILE_NAME))?;
    if verbose {
        eprintln!("[-] Replaying trace: {}", test_name);
    }
    assert!(!wks.join(TRACE_FILE_ERROR).exists());

    let dir_meta = wks.join(TRACE_DIR_META);
    let dir_data = wks.join(TRACE_DIR_DATA);
    let num_blks = fs::read_dir(&dir_meta)?.count();
    assert_eq!(num_blks, fs::read_dir(&dir_data)?.count());

    let dir_input = wks.join(TRACE_DIR_INPUT);
    let dir_output = wks.join(TRACE_DIR_OUTPUT);
    let num_txns = fs::read_dir(&dir_input)?.count();
    assert_eq!(num_txns, fs::read_dir(&dir_output)?.count());

    // iterate over each transaction blocks
    for blk_seq in 0..num_blks {
        let file_meta = dir_meta.join(blk_seq.to_string());
        let meta: TraceSeqMapping = bcs::from_bytes(&fs::read(file_meta)?)?;
        let (blk_id, txn_seqs, res_seqs) = meta;
        assert_eq!(blk_seq, blk_id);
        assert_eq!(txn_seqs, res_seqs);

        // load the global state at the beginning of the block
        let file_data = dir_data.join(blk_id.to_string());
        let mut data: FakeDataStore = bcs::from_bytes(&fs::read(file_data)?)?;

        // iterate over transactions in the block
        for (txn_seq, res_seq) in txn_seqs.into_iter().zip(res_seqs.into_iter()) {
            let file_input = dir_input.join(txn_seq.to_string());
            let txn: Transaction = bcs::from_bytes(&fs::read(file_input)?)?;
            if verbose {
                eprintln!(
                    "[-] {}: {} - {}: txn: {:?}",
                    test_name, blk_seq, txn_seq, txn
                );
            }

            let file_output = dir_output.join(res_seq.to_string());
            let res: TransactionOutput = bcs::from_bytes(&fs::read(file_output)?)?;
            if verbose {
                eprintln!(
                    "[-] {}: {} - {}: res: {:?}",
                    test_name, blk_seq, res_seq, res
                );
            }

            match txn {
                Transaction::GenesisTransaction(_) => {
                    if !matches!(
                        res.status(),
                        TransactionStatus::Keep(KeptVMStatus::Executed)
                    ) {
                        if verbose {
                            eprintln!(
                                "[!] Replay stopped due to failures in genesis transaction: {}",
                                test_name
                            );
                        }
                        return Ok(());
                    }
                }
                Transaction::BlockMetadata(block_metadata) => {
                    if !matches!(
                        res.status(),
                        TransactionStatus::Keep(KeptVMStatus::Executed)
                    ) {
                        if verbose {
                            eprintln!(
                                "[!] Replay stopped due to failures in block metadata transaction: {}",
                                test_name
                            );
                        }
                        return Ok(());
                    }
                    replay_txn_block_metadata(block_metadata, &data, &res);
                    data.add_write_set(res.write_set());
                }
                Transaction::UserTransaction(signed_txn) => {
                    let (senders, script_fun, is_admin) = match signed_txn.payload() {
                        TransactionPayload::Script(script) => {
                            match script_to_script_function(script) {
                                None => {
                                    // TODO: there is not much we can do as an unknown script is
                                    // written in IR, so just apply the write-set and continue
                                    data.add_write_set(res.write_set());
                                    continue;
                                }
                                Some(script_fun) => (vec![signed_txn.sender()], script_fun, false),
                            }
                        }
                        TransactionPayload::ScriptFunction(script_fun) => {
                            // NOTE: if module transaction are not allowed and direct write-sets do
                            // not contain code, then we know that a script function executed should
                            // be in the diem framework.
                            (vec![signed_txn.sender()], script_fun.clone(), false)
                        }
                        TransactionPayload::Module(_) => {
                            // TODO: there is not much we can do as the module is written in IR,
                            // hence, exit the test and call it successful
                            if verbose {
                                eprintln!(
                                    "[!] Replay stopped due to non-Diem module compilation: {}",
                                    test_name
                                );
                            }
                            return Ok(());
                        }
                        TransactionPayload::WriteSet(WriteSetPayload::Direct(change_set)) => {
                            for (ap, _) in change_set.write_set().iter() {
                                match ap.get_path() {
                                    AP::Code(_) => {
                                        // NOTE: a direct write-set can modify the code arbitrarily,
                                        // which we do not model for now
                                        if verbose {
                                            eprintln!(
                                                "[!] Replay stopped due to code modification from \
                                                direct write-set: {}",
                                                test_name
                                            );
                                        }
                                        return Ok(());
                                    }
                                    AP::Resource(_) => (),
                                }
                            }
                            data.add_write_set(res.write_set());
                            continue;
                        }
                        TransactionPayload::WriteSet(WriteSetPayload::Script {
                            execute_as,
                            script,
                        }) => match script_to_script_function(script) {
                            None => {
                                // TODO: there is not much we can do as an unknown script is written
                                // in IR, so just apply the write-set and continue
                                data.add_write_set(res.write_set());
                                continue;
                            }
                            Some(script_fun) => {
                                (vec![signed_txn.sender(), *execute_as], script_fun, true)
                            }
                        },
                    };

                    // only execute scripts that has not been discarded
                    if !res.status().is_discarded() {
                        if verbose {
                            eprintln!(
                                "[-] {}: {} - {}: entrypoint: {}::{}::{}",
                                test_name,
                                blk_seq,
                                res_seq,
                                script_fun.module().address().short_str_lossless(),
                                script_fun.module().name(),
                                script_fun.function(),
                            );
                        }
                        let txn_meta = TransactionMetadata::new(&signed_txn);
                        replay_txn_script_function(
                            is_admin,
                            senders,
                            txn_meta,
                            script_fun,
                            signed_txn.gas_currency_code(),
                            &data,
                            &res,
                        );
                        data.add_write_set(res.write_set());
                    }
                }
            }
        }
    }
    Ok(())
}

fn replay<P: AsRef<Path>>(root: P, verbose: bool) -> Result<()> {
    let root = root.as_ref();
    for entry in WalkDir::new(root).into_iter() {
        let entry = entry?;
        if entry.file_name() == TRACE_FILE_NAME {
            let wks = entry
                .path()
                .parent()
                .ok_or_else(|| anyhow!("Cannot traverse the root directory"))?;
            replay_trace(wks, verbose)?;
        }
    }
    Ok(())
}

//**************************************************************************************************
// Entrypoint
//**************************************************************************************************

#[derive(StructOpt)]
struct ConverterArgs {
    /// Trace files
    #[structopt(short = "t", long = "trace")]
    trace_files: Vec<String>,

    /// Verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

pub fn main() -> Result<()> {
    let args = ConverterArgs::from_args();
    for trace in args.trace_files {
        replay(trace, args.verbose)?;
    }
    Ok(())
}
