// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    change_set::ChangeSet, ledger_store::LedgerStore, state_store::StateStore,
    transaction_store::TransactionStore,
};
use anyhow::{ensure, Result};
use jellyfish_merkle::{restore::JellyfishMerkleRestore, TreeReader, TreeWriter};
use libra_crypto::HashValue;
use libra_types::{ledger_info::LedgerInfoWithSignatures, transaction::Version};
use schemadb::DB;
use std::sync::Arc;

/// Provides functionalities for LibraDB data restore.
#[derive(Clone)]
pub struct RestoreHandler {
    db: Arc<DB>,
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl RestoreHandler {
    pub(crate) fn new(
        db: Arc<DB>,
        ledger_store: Arc<LedgerStore>,
        transaction_store: Arc<TransactionStore>,
        state_store: Arc<StateStore>,
    ) -> Self {
        Self {
            db,
            ledger_store,
            transaction_store,
            state_store,
        }
    }

    pub fn get_state_restore_receiver(
        &self,
        version: Version,
        expected_root_hash: HashValue,
    ) -> Result<JellyfishMerkleRestore<impl TreeReader + TreeWriter>> {
        JellyfishMerkleRestore::new(&*self.state_store, version, expected_root_hash)
    }

    pub fn save_ledger_infos(&self, ledger_infos: &[LedgerInfoWithSignatures]) -> Result<()> {
        ensure!(!ledger_infos.is_empty(), "No LedgerInfos to save.");

        let mut cs = ChangeSet::new();
        ledger_infos
            .iter()
            .map(|li| self.ledger_store.put_ledger_info(li, &mut cs))
            .collect::<Result<Vec<_>>>()?;
        self.db.write_schemas(cs.batch)?;

        if let Some(li) = self.ledger_store.get_latest_ledger_info_option() {
            if li.ledger_info().epoch() > ledger_infos.last().unwrap().ledger_info().epoch() {
                // No need to update latest ledger info.
                return Ok(());
            }
        }

        self.ledger_store
            .set_latest_ledger_info(ledger_infos.last().unwrap().clone());
        Ok(())
    }
}
