// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use libra_types::{
    account_address::AccountAddress,
    account_config::libra_root_address,
    account_state::AccountState,
    transaction::{ChangeSet, Transaction, TransactionOutput, Version},
};
use libra_validator_interface::{
    DBDebuggerInterface, DebuggerStateView, JsonRpcDebuggerInterface, LibraValidatorInterface,
};
use libra_vm::{
    data_cache::RemoteStorage, txn_effects_to_writeset_and_events, LibraVM, VMExecutor,
};
use move_core_types::gas_schedule::{GasAlgebra, GasUnits};
use move_lang::{compiled_unit::CompiledUnit, move_compile_no_report, shared::Address};
use move_vm_runtime::{logging::NoContextLog, move_vm::MoveVM, session::Session};
use move_vm_test_utils::{ChangeSet as MoveChanges, DeltaStorage};
use move_vm_types::gas_schedule::{zero_cost_schedule, CostStrategy};
use resource_viewer::{AnnotatedAccountStateBlob, MoveValueAnnotator};
use std::{convert::TryFrom, path::Path};
use vm::errors::VMResult;

#[cfg(test)]
mod unit_tests;

pub struct LibraDebugger {
    debugger: Box<dyn LibraValidatorInterface>,
}

impl LibraDebugger {
    pub fn new(debugger: Box<dyn LibraValidatorInterface>) -> Self {
        Self { debugger }
    }

    pub fn json_rpc(url: &str) -> Result<Self> {
        Ok(Self {
            debugger: Box::new(JsonRpcDebuggerInterface::new(url)?),
        })
    }

    pub fn db<P: AsRef<Path> + Clone>(db_root_path: P) -> Result<Self> {
        Ok(Self {
            debugger: Box::new(DBDebuggerInterface::open(db_root_path)?),
        })
    }

    pub fn execute_transactions_at_version(
        &self,
        version: Version,
        txns: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutput>> {
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        LibraVM::execute_block(txns, &state_view)
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
    }

    pub fn execute_past_transactions(
        &self,
        mut begin: Version,
        mut limit: u64,
    ) -> Result<Vec<TransactionOutput>> {
        let mut txns = self.debugger.get_committed_transactions(begin, limit)?;
        let mut ret = vec![];
        while limit != 0 {
            println!(
                "Starting epoch execution at {:?}, {:?} transactions remaining",
                begin, limit
            );
            let mut epoch_result = self.execute_transactions_by_epoch(begin, txns.clone())?;
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
    ) -> Result<Vec<TransactionOutput>> {
        let results = self.execute_transactions_at_version(begin, txns)?;
        let mut ret = vec![];
        let mut is_reconfig = false;

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

    pub fn annotate_account_state_at_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AnnotatedAccountStateBlob>> {
        let state_view = DebuggerStateView::new(&*self.debugger, version);
        let remote_storage = RemoteStorage::new(&state_view);
        let annotator = MoveValueAnnotator::new(&remote_storage);
        Ok(
            match self
                .debugger
                .get_account_state_by_version(account, version)?
            {
                Some(s) => {
                    let account_state = AccountState::try_from(&s)?;
                    Some(annotator.view_account_state(&account_state)?)
                }
                None => None,
            },
        )
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
                    vec![libra_root_address(), sender],
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
    let new_epoch_event_key = libra_types::on_chain_config::new_epoch_event_key();
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
    let (files, units_or_errors) =
        move_compile_no_report(targets, &stdlib::stdlib_files(), sender_opt, None)?;
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
