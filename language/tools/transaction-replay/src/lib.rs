// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{json_rpc_debugger::JsonRpcDebuggerInterface, storage_debugger::DBDebuggerInterface};
use anyhow::{format_err, Result};
use libra_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    transaction::{Transaction, TransactionOutput, Version},
};
use libra_vm::{LibraVM, VMExecutor};
use resource_viewer::{AnnotatedAccountStateBlob, MoveValueAnnotator};
use std::{convert::TryFrom, path::Path};

pub use crate::transaction_debugger_interface::{DebuggerStateView, StorageDebuggerInterface};

mod json_rpc_debugger;
mod storage_debugger;
mod transaction_debugger_interface;

pub struct LibraDebugger {
    debugger: Box<dyn StorageDebuggerInterface>,
}

impl LibraDebugger {
    pub fn new(debugger: Box<dyn StorageDebuggerInterface>) -> Self {
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
        let annotator = MoveValueAnnotator::new(&state_view);
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
}

fn is_reconfiguration(vm_output: &TransactionOutput) -> bool {
    let new_epoch_event_key = libra_types::on_chain_config::new_epoch_event_key();
    vm_output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key)
}
