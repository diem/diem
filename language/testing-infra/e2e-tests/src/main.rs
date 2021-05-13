// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::{fs, path::Path};
use structopt::StructOpt;
use walkdir::WalkDir;

use bytecode_interpreter::{
    concrete::{
        runtime::{convert_move_struct_tag, convert_move_value},
        settings::InterpreterSettings,
        ty::BaseType,
        value::GlobalState,
    },
    interpret_with_default_pipeline_and_bcs_arguments,
};
use diem_framework::diem_stdlib_files;
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
use move_model::{model::GlobalEnv, run_model_builder};
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
// Cross-VM comparison
//**************************************************************************************************

struct CrossRunner<'env> {
    env: &'env GlobalEnv,
    move_vm_state: FakeDataStore,
    stackless_vm_state: GlobalState,
    stackless_vm_settings: InterpreterSettings,
}

impl<'env> CrossRunner<'env> {
    pub fn new(env: &'env GlobalEnv, data_store: &FakeDataStore, flags: &'env ReplayFlags) -> Self {
        let mut move_vm_state = data_store.clone();
        let mut stackless_vm_state = GlobalState::default();
        for (ap, blob) in data_store.inner() {
            match ap.get_path() {
                AP::Code(module_id) => {
                    let module_env = env.find_module_by_language_storage_id(&module_id).unwrap();
                    let mut code = vec![];
                    module_env
                        .get_verified_module()
                        .serialize(&mut code)
                        .unwrap();
                    // update the module code to the same version as the one in the GlobalEnv
                    move_vm_state.add_module(&module_id, code);
                }
                AP::Resource(struct_tag) => {
                    let inst = convert_move_struct_tag(env, &struct_tag).unwrap();
                    let struct_ty = BaseType::mk_struct(inst);
                    let struct_val =
                        MoveValue::simple_deserialize(blob, &struct_ty.to_move_type_layout())
                            .unwrap();
                    let resource = convert_move_value(env, &struct_val, &struct_ty).unwrap();
                    let inst = struct_ty.into_struct_inst();
                    stackless_vm_state.put_resource(ap.address, inst, resource);
                }
            }
        }

        let settings = if flags.verbose_stackless_vm {
            InterpreterSettings::verbose_default()
        } else {
            InterpreterSettings::default()
        };

        Self {
            env,
            move_vm_state,
            stackless_vm_state,
            stackless_vm_settings: settings,
        }
    }

    pub fn step_function_and_compare(
        &mut self,
        module_id: &ModuleId,
        function_name: &IdentStr,
        ty_args: &[TypeTag],
        args: &[Vec<u8>],
    ) {
        // execute via move VM
        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.move_vm_state);
        let move_vm_return_values = execute_function_via_session(
            &mut session,
            module_id,
            function_name,
            ty_args.to_vec(),
            args.to_vec(),
        );
        let (move_vm_change_set, move_events) = session.finish().unwrap();

        // execute via stackless VM
        let (stackless_vm_return_values, stackless_vm_change_set, new_stackless_vm_state) =
            interpret_with_default_pipeline_and_bcs_arguments(
                self.env,
                module_id,
                function_name,
                ty_args,
                args,
                &self.stackless_vm_state,
                self.stackless_vm_settings.clone(),
            );

        // compare
        assert_eq!(move_vm_return_values, stackless_vm_return_values);
        assert_eq!(move_vm_change_set, stackless_vm_change_set);

        // update the states
        let (move_write_set, _) =
            convert_changeset_and_events(move_vm_change_set, move_events).unwrap();
        self.move_vm_state.add_write_set(&move_write_set);
        self.stackless_vm_state = new_stackless_vm_state;
    }

    pub fn step_script_function_and_compare(
        &mut self,
        module_id: &ModuleId,
        function_name: &IdentStr,
        ty_args: &[TypeTag],
        args: &[Vec<u8>],
        senders: &[AccountAddress],
    ) {
        // execute via move VM
        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.move_vm_state);
        let move_vm_return_values = execute_script_function_via_session(
            &mut session,
            module_id,
            function_name,
            ty_args.to_vec(),
            args.to_vec(),
            senders.to_vec(),
        );
        let (move_vm_change_set, move_events) = session.finish().unwrap();

        // execute via stackless VM
        let (stackless_vm_return_values, stackless_vm_change_set, new_stackless_vm_state) =
            interpret_with_default_pipeline_and_bcs_arguments(
                self.env,
                module_id,
                function_name,
                ty_args,
                args,
                &self.stackless_vm_state,
                self.stackless_vm_settings.clone(),
                // TODO (mengxu): add senders
            );
        let stackless_vm_return_values =
            stackless_vm_return_values.map(|rets| assert!(rets.is_empty()));

        // compare
        assert_eq!(move_vm_return_values, stackless_vm_return_values);
        assert_eq!(move_vm_change_set, stackless_vm_change_set);

        // update the states
        let (move_write_set, _) =
            convert_changeset_and_events(move_vm_change_set, move_events).unwrap();
        self.move_vm_state.add_write_set(&move_write_set);
        self.stackless_vm_state = new_stackless_vm_state;
    }
}

//**************************************************************************************************
// Executors
//**************************************************************************************************

fn execute_function_via_session(
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

fn execute_function_via_session_and_xrunner(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
    module_id: &ModuleId,
    function_name: &IdentStr,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
) -> VMResult<Vec<Vec<u8>>> {
    if let Some(runner) = xrunner {
        runner.step_function_and_compare(module_id, function_name, &ty_args, &args);
    }
    execute_function_via_session(session, module_id, function_name, ty_args, args)
}

fn execute_script_function_via_session(
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

fn execute_script_function_via_session_and_xrunner(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
    module_id: &ModuleId,
    function_name: &IdentStr,
    ty_args: Vec<TypeTag>,
    args: Vec<Vec<u8>>,
    senders: Vec<AccountAddress>,
) -> VMResult<()> {
    if let Some(runner) = xrunner {
        runner.step_script_function_and_compare(
            module_id,
            function_name,
            &ty_args,
            &args,
            &senders,
        );
    }
    execute_script_function_via_session(session, module_id, function_name, ty_args, args, senders)
}

//**************************************************************************************************
// Transaction replay
//**************************************************************************************************

struct TraceReplayer<'env> {
    env: &'env GlobalEnv,
    data_store: FakeDataStore,
    check_stackless_vm: bool,
    flags: &'env ReplayFlags,
}

impl<'env> TraceReplayer<'env> {
    pub fn new(
        env: &'env GlobalEnv,
        data_store: FakeDataStore,
        check_stackless_vm: bool,
        flags: &'env ReplayFlags,
    ) -> Self {
        Self {
            env,
            data_store,
            check_stackless_vm,
            flags,
        }
    }

    pub fn replay_txn_block_metadata(
        &self,
        block_metadata: BlockMetadata,
        expect_output: &TransactionOutput,
    ) {
        // args
        let signer = reserved_vm_address();
        let (round, timestamp, previous_votes, proposer) = block_metadata.into_inner();
        let args: Vec<_> = vec![
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
        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.data_store);
        let mut xrunner = if self.check_stackless_vm {
            Some(CrossRunner::new(self.env, &self.data_store, self.flags))
        } else {
            None
        };

        let result = execute_function_via_session_and_xrunner(
            &mut session,
            xrunner.as_mut(),
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

    fn replay_txn_user_script_function_internal(
        &self,
        senders: Vec<AccountAddress>,
        txn_meta: TransactionMetadata,
        script_fun: ScriptFunction,
        gas_currency: &str,
        gas_usage: u64,
    ) -> VMResult<(ChangeSet, Vec<Event>)> {
        let gas_currency_ty =
            type_tag_for_currency_code(from_currency_code_string(gas_currency).unwrap());

        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.data_store);
        let mut xrunner = if self.check_stackless_vm {
            Some(CrossRunner::new(self.env, &self.data_store, self.flags))
        } else {
            None
        };

        // prologue -> main -> epilogue
        execute_txn_user_script_prologue(
            &mut session,
            xrunner.as_mut(),
            &txn_meta,
            &gas_currency_ty,
        )?;

        let result = execute_script_function_via_session_and_xrunner(
            &mut session,
            xrunner.as_mut(),
            script_fun.module(),
            script_fun.function(),
            script_fun.ty_args().to_vec(),
            script_fun.args().to_vec(),
            senders,
        );
        match result {
            Ok(_) => {
                execute_txn_user_script_epilogue(
                    &mut session,
                    xrunner.as_mut(),
                    &txn_meta,
                    &gas_currency_ty,
                    gas_usage,
                )?;
                session.finish()
            }
            Err(err) => {
                let status = TransactionStatus::from(err.clone().into_vm_status());
                if status.is_discarded() {
                    return Err(err);
                }
                let mut new_session = move_vm.new_session(&self.data_store);
                let mut new_xrunner = if self.check_stackless_vm {
                    Some(CrossRunner::new(self.env, &self.data_store, self.flags))
                } else {
                    None
                };
                execute_txn_user_script_epilogue(
                    &mut new_session,
                    new_xrunner.as_mut(),
                    &txn_meta,
                    &gas_currency_ty,
                    gas_usage,
                )?;
                new_session.finish()
            }
        }
    }

    fn replay_txn_admin_script_function_internal(
        &self,
        senders: Vec<AccountAddress>,
        txn_meta: TransactionMetadata,
        script_fun: ScriptFunction,
    ) -> VMResult<(ChangeSet, Vec<Event>)> {
        let move_vm = MoveVM::new();
        let mut session = move_vm.new_session(&self.data_store);
        let mut xrunner = if self.check_stackless_vm {
            Some(CrossRunner::new(self.env, &self.data_store, self.flags))
        } else {
            None
        };

        // prologue -> main -> epilogue
        execute_txn_admin_script_prologue(&mut session, xrunner.as_mut(), &txn_meta)?;

        let result = execute_script_function_via_session_and_xrunner(
            &mut session,
            xrunner.as_mut(),
            script_fun.module(),
            script_fun.function(),
            script_fun.ty_args().to_vec(),
            script_fun.args().to_vec(),
            senders,
        );
        match result {
            Ok(_) => {
                execute_txn_admin_script_epilogue(&mut session, xrunner.as_mut(), &txn_meta)?;
                session.finish()
            }
            Err(err) => {
                let status = TransactionStatus::from(err.clone().into_vm_status());
                if status.is_discarded() {
                    return Err(err);
                }
                let mut new_session = move_vm.new_session(&self.data_store);
                let mut new_xrunner = if self.check_stackless_vm {
                    Some(CrossRunner::new(self.env, &self.data_store, self.flags))
                } else {
                    None
                };
                execute_txn_admin_script_epilogue(
                    &mut new_session,
                    new_xrunner.as_mut(),
                    &txn_meta,
                )?;
                new_session.finish()
            }
        }
    }

    pub fn replay_txn_script_function(
        &self,
        is_admin: bool,
        senders: Vec<AccountAddress>,
        txn_meta: TransactionMetadata,
        script_fun: ScriptFunction,
        gas_currency: &str,
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
            self.replay_txn_admin_script_function_internal(senders, txn_meta, script_fun)
        } else {
            self.replay_txn_user_script_function_internal(
                senders,
                txn_meta,
                script_fun,
                gas_currency,
                expect_output.gas_used(),
            )
        };

        // compare
        compare_output(expect_output, actual_output);
    }
}

fn execute_txn_user_script_prologue(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
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

    let rets = execute_function_via_session_and_xrunner(
        session,
        xrunner,
        &*ACCOUNT_MODULE,
        &*SCRIPT_PROLOGUE_NAME,
        vec![gas_currency_ty.clone()],
        args,
    )?;
    assert!(rets.is_empty());
    Ok(())
}

fn execute_txn_user_script_epilogue(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
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

    let rets = execute_function_via_session_and_xrunner(
        session,
        xrunner,
        &*ACCOUNT_MODULE,
        &*USER_EPILOGUE_NAME,
        vec![gas_currency_ty.clone()],
        args,
    )?;
    assert!(rets.is_empty());
    Ok(())
}

fn execute_txn_admin_script_prologue(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
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

    let rets = execute_function_via_session_and_xrunner(
        session,
        xrunner,
        &*ACCOUNT_MODULE,
        &*WRITESET_PROLOGUE_NAME,
        vec![],
        args,
    )?;
    assert!(rets.is_empty());
    Ok(())
}

fn execute_txn_admin_script_epilogue(
    session: &mut Session<FakeDataStore>,
    xrunner: Option<&mut CrossRunner>,
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

    let rets = execute_function_via_session_and_xrunner(
        session,
        xrunner,
        &*ACCOUNT_MODULE,
        &*WRITESET_EPILOGUE_NAME,
        vec![],
        args,
    )?;
    assert!(rets.is_empty());
    Ok(())
}

//**************************************************************************************************
// Trace replay
//**************************************************************************************************

fn replay_trace<P: AsRef<Path>>(
    wks: P,
    env: &GlobalEnv,
    check_stackless_vm: bool,
    flags: &ReplayFlags,
) -> Result<()> {
    let wks = wks.as_ref();

    // sanity checks
    let test_name = fs::read_to_string(wks.join(TRACE_FILE_NAME))?;
    if flags.verbose_trace_meta {
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
        let data: FakeDataStore = bcs::from_bytes(&fs::read(file_data)?)?;

        // construct the trace replayer
        let mut replayer = TraceReplayer::new(env, data, check_stackless_vm, flags);

        // iterate over transactions in the block
        for (txn_seq, res_seq) in txn_seqs.into_iter().zip(res_seqs.into_iter()) {
            let file_input = dir_input.join(txn_seq.to_string());
            let txn: Transaction = bcs::from_bytes(&fs::read(file_input)?)?;
            if flags.verbose_trace_step {
                eprintln!(
                    "[-] {}: {} - {}: txn: {:?}",
                    test_name, blk_seq, txn_seq, txn
                );
            }

            let file_output = dir_output.join(res_seq.to_string());
            let res: TransactionOutput = bcs::from_bytes(&fs::read(file_output)?)?;
            if flags.verbose_trace_step {
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
                        if flags.warning {
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
                        if flags.warning {
                            eprintln!(
                                "[!] Replay stopped due to failures in block metadata transaction: {}",
                                test_name
                            );
                        }
                        return Ok(());
                    }
                    replayer.replay_txn_block_metadata(block_metadata, &res);
                    replayer.data_store.add_write_set(res.write_set());
                }
                Transaction::UserTransaction(signed_txn) => {
                    let (senders, script_fun, is_admin) = match signed_txn.payload() {
                        TransactionPayload::Script(script) => {
                            match script_to_script_function(script) {
                                None => {
                                    // TODO: there is not much we can do as an unknown script is
                                    // written in IR, so just apply the write-set and continue
                                    replayer.data_store.add_write_set(res.write_set());
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
                            if flags.warning {
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
                                        if flags.warning {
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
                            replayer.data_store.add_write_set(res.write_set());
                            continue;
                        }
                        TransactionPayload::WriteSet(WriteSetPayload::Script {
                            execute_as,
                            script,
                        }) => match script_to_script_function(script) {
                            None => {
                                // TODO: there is not much we can do as an unknown script is written
                                // in IR, so just apply the write-set and continue
                                replayer.data_store.add_write_set(res.write_set());
                                continue;
                            }
                            Some(script_fun) => {
                                (vec![signed_txn.sender(), *execute_as], script_fun, true)
                            }
                        },
                    };

                    // only execute scripts that has not been discarded
                    if !res.status().is_discarded() {
                        if flags.verbose_trace_step {
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
                        replayer.replay_txn_script_function(
                            is_admin,
                            senders,
                            txn_meta,
                            script_fun,
                            signed_txn.gas_currency_code(),
                            &res,
                        );
                        replayer.data_store.add_write_set(res.write_set());
                    }
                }
            }
        }
    }
    Ok(())
}

fn replay<P: AsRef<Path>>(
    root: P,
    env: &GlobalEnv,
    filters: &[String],
    check_stackless_vm: bool,
    flags: &ReplayFlags,
) -> Result<()> {
    let root = root.as_ref();
    for entry in WalkDir::new(root).into_iter() {
        let entry = entry?;
        if entry.file_name() == TRACE_FILE_NAME {
            let wks = entry
                .path()
                .parent()
                .ok_or_else(|| anyhow!("Cannot traverse the root directory"))?;

            let should_replay = if filters.is_empty() {
                true
            } else {
                let wks_name = wks
                    .file_name()
                    .ok_or_else(|| anyhow!("Cannot get the trace directory name"))?
                    .to_str()
                    .unwrap();
                filters.iter().any(|f| wks_name.contains(f))
            };
            if should_replay {
                replay_trace(wks, env, check_stackless_vm, flags)?;
            }
        }
    }
    Ok(())
}

//**************************************************************************************************
// Entrypoint
//**************************************************************************************************

#[derive(StructOpt)]
struct ReplayArgs {
    /// Trace files
    #[structopt(short = "t", long = "trace")]
    trace_files: Vec<String>,

    /// Filter
    #[structopt(short = "f", long = "filter")]
    filters: Vec<String>,

    /// Cross check the stackless VM against the Move VM
    #[structopt(short = "s", long = "stackless")]
    stackless: bool,

    /// Verbose mode
    #[structopt(short = "v", long = "verbose")]
    verbose: Option<u64>,

    /// Warning mode
    #[structopt(short = "w", long = "warning")]
    warning: Option<u64>,
}

struct ReplayFlags {
    /// Print information per trace
    verbose_trace_meta: bool,
    /// Print information per-step in the trace
    verbose_trace_step: bool,
    /// Enable the verbose mode in stackless VM
    verbose_stackless_vm: bool,
    /// Print warnings
    warning: bool,
}

pub fn main() -> Result<()> {
    let args = ReplayArgs::from_args();
    let flags = ReplayFlags {
        verbose_trace_meta: args.verbose.map_or(false, |level| level > 0),
        verbose_trace_step: args.verbose.map_or(false, |level| level > 1),
        verbose_stackless_vm: args.verbose.map_or(false, |level| level > 2),
        warning: args.warning.map_or(false, |level| level > 0),
    };
    let env = run_model_builder(&diem_stdlib_files(), &[])?;
    for trace in args.trace_files {
        replay(trace, &env, &args.filters, args.stackless, &flags)?;
    }
    Ok(())
}
