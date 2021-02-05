// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, bail, format_err, Result};
use diem_types::{
    access_path,
    account_address::AccountAddress,
    account_config::diem_root_address,
    account_state::AccountState,
    transaction::{ChangeSet, Transaction, TransactionOutput, Version, WriteSetPayload},
    write_set::WriteOp,
};
use diem_validator_interface::{
    DBDebuggerInterface, DebuggerStateView, DiemValidatorInterface, JsonRpcDebuggerInterface,
};
use diem_vm::{data_cache::RemoteStorage, txn_effects_to_writeset_and_events, DiemVM, VMExecutor};
use move_cli::OnDiskStateView;
use move_core_types::gas_schedule::{GasAlgebra, GasUnits};
use move_lang::{compiled_unit::CompiledUnit, move_compile, shared::Address};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM, session::Session};
use move_vm_test_utils::{ChangeSet as MoveChanges, DeltaStorage};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use resource_viewer::{AnnotatedAccountStateBlob, MoveValueAnnotator};
use std::{
    convert::TryFrom,
    path::{Path, PathBuf},
};
use vm::{errors::VMResult, file_format::CompiledModule};

#[cfg(test)]
mod unit_tests;

pub struct DiemDebugger {
    debugger: Box<dyn DiemValidatorInterface>,
    build_dir: PathBuf,
    storage_dir: PathBuf,
}

impl DiemDebugger {
    pub fn new(debugger: Box<dyn DiemValidatorInterface>) -> Self {
        Self {
            debugger,
            build_dir: PathBuf::from(move_cli::DEFAULT_BUILD_DIR),
            storage_dir: PathBuf::from(move_cli::DEFAULT_STORAGE_DIR),
        }
    }

    pub fn json_rpc(url: &str) -> Result<Self> {
        Ok(Self::new(Box::new(JsonRpcDebuggerInterface::new(url)?)))
    }

    pub fn db<P: AsRef<Path> + Clone>(db_root_path: P) -> Result<Self> {
        Ok(Self::new(Box::new(DBDebuggerInterface::open(
            db_root_path,
        )?)))
    }

    pub fn execute_transactions_at_version(
        &self,
        version: Version,
        txns: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>> {
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        DiemVM::execute_block(txns, &state_view)
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
    }

    pub fn execute_past_transactions(
        &self,
        mut begin: Version,
        mut limit: u64,
        save_write_sets: bool,
    ) -> Result<Vec<TransactionOutput>> {
        let mut txns = self.debugger.get_committed_transactions(begin, limit)?;
        let mut ret = vec![];
        while limit != 0 {
            println!(
                "Starting epoch execution at {:?}, {:?} transactions remaining",
                begin, limit
            );
            let mut epoch_result =
                self.execute_transactions_by_epoch(begin, txns.clone(), save_write_sets)?;
            begin += epoch_result.len() as u64;
            limit -= epoch_result.len() as u64;
            txns = txns.split_off(epoch_result.len());
            ret.append(&mut epoch_result);
        }
        Ok(ret)
    }

    pub fn execute_transactions_by_epoch(
        &self,
        begin: Version,
        txns: Vec<Transaction>,
        save_write_sets: bool,
    ) -> Result<Vec<TransactionOutput>> {
        let results = self.execute_transactions_at_version(begin, txns)?;
        let mut ret = vec![];
        let mut is_reconfig = false;

        if save_write_sets {
            for result in &results {
                self.save_write_sets(result)?
            }
        }

        for result in results.into_iter() {
            if is_reconfig {
                continue;
            }
            if is_reconfiguration(&result) {
                is_reconfig = true;
            }
            ret.push(result)
        }
        Ok(ret)
    }

    pub fn execute_writeset_at_version(
        &self,
        version: Version,
        payload: &WriteSetPayload,
        save_write_set: bool,
    ) -> Result<TransactionOutput> {
        let state_view = DebuggerStateView::new(&*self.debugger, version + 1);
        let vm = DiemVM::new(&state_view);
        let cache = diem_vm::data_cache::StateViewCache::new(&state_view);
        let log_context = NoContextLog::new();
        let mut txn_data = diem_vm::transaction_metadata::TransactionMetadata::default();
        txn_data.sequence_number = match self
            .debugger
            .get_account_state_by_version(diem_root_address(), version)?
        {
            Some(account) => account
                .get_account_resource()?
                .ok_or_else(|| anyhow!("Diem root account doesn't exist"))?
                .sequence_number(),
            None => bail!("Diem root account blob doesn't exist"),
        };
        txn_data.sender = diem_root_address();

        let (_, output) = vm
            .execute_writeset_transaction(&cache, &payload, txn_data, &log_context)
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))?;
        if save_write_set {
            self.save_write_sets(&output)?;
        }
        Ok(output)
    }

    fn save_write_sets(&self, o: &TransactionOutput) -> Result<()> {
        let state_view = OnDiskStateView::create(&self.build_dir, &self.storage_dir)?;
        for (ap, op) in o.write_set() {
            let addr = ap.address;
            match ap.get_path() {
                access_path::Path::Resource(tag) => match op {
                    WriteOp::Deletion => state_view.delete_resource(addr, tag)?,
                    WriteOp::Value(bytes) => state_view.save_resource_bytes(addr, tag, bytes)?,
                },
                access_path::Path::Code(module_id) => match op {
                    WriteOp::Deletion => state_view.delete_module(&module_id)?,
                    WriteOp::Value(bytes) => state_view.save_module(&module_id, bytes)?,
                },
            }
        }
        for event in o.events() {
            state_view.save_contract_event(event.clone())?
        }
        Ok(())
    }

    fn save_account_state(
        &self,
        account: AccountAddress,
        account_state: &AccountState,
    ) -> Result<()> {
        let disk_view = OnDiskStateView::create(&self.build_dir, &self.storage_dir)?;
        for (key, value) in account_state.iter() {
            let key: access_path::Path = bcs::from_bytes(key)?;
            match key {
                access_path::Path::Code(m) => disk_view.save_module(&m, value)?,
                access_path::Path::Resource(struct_tag) => {
                    disk_view.save_resource_bytes(account, struct_tag, value)?
                }
            }
        }
        Ok(())
    }

    pub fn get_diem_framework_modules_at_version(
        &self,
        version: Version,
        save_write_sets: bool,
    ) -> Result<Vec<CompiledModule>> {
        let modules = self
            .debugger
            .get_diem_framework_modules_by_version(version)?;
        if save_write_sets {
            let state_view = OnDiskStateView::create(&self.build_dir, &self.storage_dir)?;
            for m in &modules {
                let mut module_bytes = vec![];
                m.serialize(&mut module_bytes)?;
                state_view.save_module(&m.self_id(), &module_bytes)?
            }
        }
        Ok(modules)
    }

    pub fn annotate_account_state_at_version(
        &self,
        account: AccountAddress,
        version: Version,
        save_write_sets: bool,
    ) -> Result<Option<AnnotatedAccountStateBlob>> {
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        let remote_storage = RemoteStorage::new(&state_view);
        let annotator = MoveValueAnnotator::new(&remote_storage);
        Ok(
            match self
                .debugger
                .get_account_state_by_version(account, version)?
            {
                Some(account_state) => {
                    if save_write_sets {
                        self.save_account_state(account, &account_state)?;
                    }
                    Some(annotator.view_account_state(&account_state)?)
                }
                None => None,
            },
        )
    }

    pub fn annotate_key_accounts_at_version(
        &self,
        version: Version,
        save_write_sets: bool,
    ) -> Result<Vec<(AccountAddress, AnnotatedAccountStateBlob)>> {
        let accounts = self.debugger.get_admin_accounts(version)?;
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        let remote_storage = RemoteStorage::new(&state_view);
        let annotator = MoveValueAnnotator::new(&remote_storage);

        let mut result = vec![];
        for (addr, state) in accounts.into_iter() {
            if save_write_sets {
                self.save_account_state(addr, &state)?;
            }
            result.push((addr, annotator.view_account_state(&state)?));
        }
        Ok(result)
    }

    pub fn get_latest_version(&self) -> Result<Version> {
        self.debugger.get_latest_version()
    }

    pub fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>> {
        self.debugger.get_version_by_account_sequence(account, seq)
    }

    pub fn run_session_at_version<F>(
        &self,
        version: Version,
        override_changeset: Option<MoveChanges>,
        f: F,
    ) -> Result<ChangeSet>
    where
        F: FnOnce(&mut Session<DeltaStorage<RemoteStorage<DebuggerStateView>>>) -> VMResult<()>,
    {
        let move_vm = MoveVM::new();
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        let state_view_storage = RemoteStorage::new(&state_view);
        let move_changes = override_changeset.unwrap_or_else(MoveChanges::new);
        let remote_storage = DeltaStorage::new(&state_view_storage, &move_changes);
        let mut session = move_vm.new_session(&remote_storage);
        f(&mut session).map_err(|err| format_err!("Unexpected VM Error: {:?}", err))?;
        let txn_effect = session
            .finish()
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))?;
        let (write_set, events) = txn_effects_to_writeset_and_events(txn_effect)
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))?;
        Ok(ChangeSet::new(write_set, events))
    }

    pub fn bisect_transactions_by_script(
        &self,
        code_path: &str,
        sender: AccountAddress,
        begin: Version,
        end: Version,
        override_changeset: Option<MoveChanges>,
    ) -> Result<Option<Version>> {
        // TODO: The code here is compiled against the local move stdlib instead of the one from on
        // chain storage.
        let predicate = compile_move_script(code_path, sender)?;
        let gas_table = zero_cost_schedule();
        let is_version_ok = |version| {
            self.run_session_at_version(version, override_changeset.clone(), |session| {
                let mut cost_strategy = CostStrategy::system(&gas_table, GasUnits::new(0));
                let log_context = NoContextLog::new();
                session.execute_script(
                    predicate.clone(),
                    vec![],
                    vec![],
                    vec![diem_root_address(), sender],
                    &mut cost_strategy,
                    &log_context,
                )
            })
            .map(|_| ())
        };

        self.bisect_transaction_impl(is_version_ok, begin, end)
    }

    /// Find the first version between [begin, end) that nullify the predicate using binary search.
    fn bisect_transaction_impl<F>(
        &self,
        predicate: F,
        mut begin: Version,
        mut end: Version,
    ) -> Result<Option<Version>>
    where
        F: Fn(Version) -> Result<()>,
    {
        if self.get_latest_version()? + 1 < end || begin > end {
            bail!("Unexpected Version");
        }

        let mut result = None;
        while begin < end {
            let mid = begin + (end - begin) / 2;
            let mid_result = predicate(mid);
            println!("Checking Version: {:?}, got {:?}", mid, mid_result);
            if mid_result.is_err() {
                result = Some(mid);
                end = mid;
            } else {
                begin = mid + 1;
            }
        }
        Ok(result)
    }
}

fn is_reconfiguration(vm_output: &TransactionOutput) -> bool {
    let new_epoch_event_key = diem_types::on_chain_config::new_epoch_event_key();
    vm_output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key)
}

fn compile_move_script(file_path: &str, sender: AccountAddress) -> Result<Vec<u8>> {
    let sender_addr = Address::try_from(sender.as_ref()).unwrap();
    let cur_path = file_path.to_owned();
    let targets = &vec![cur_path];
    let sender_opt = Some(sender_addr);
    let (files, units_or_errors) = move_compile(
        targets,
        &diem_framework::stdlib_files(),
        sender_opt,
        None,
        false,
    )?;
    let unit = match units_or_errors {
        Err(errors) => {
            let error_buffer = move_lang::errors::report_errors_to_color_buffer(files, errors);
            bail!(String::from_utf8(error_buffer).unwrap());
        }
        Ok(mut units) => {
            let len = units.len();
            if len != 1 {
                bail!("Invalid input. Expected 1 compiled unit but got {}", len)
            }
            units.pop().unwrap()
        }
    };
    let mut out = vec![];
    match unit {
        CompiledUnit::Script { script, .. } => script.serialize(&mut out)?,
        _ => bail!("Unexpected module"),
    };
    Ok(out)
}
