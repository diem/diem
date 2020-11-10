// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{LedgerStore, LibraDB, StateStore, TransactionStore};
use anyhow::{ensure, format_err, Result};
use libra_config::config::RocksdbConfig;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, Version},
};
use std::{convert::AsRef, path::Path, sync::Arc};
use storage_interface::StartupInfo;

pub struct Librarian {
    ledger_store: Arc<LedgerStore>,
    transaction_store: Arc<TransactionStore>,
    state_store: Arc<StateStore>,
}

impl Librarian {
    pub fn new<P: AsRef<Path> + Clone>(db_root_path: P) -> Result<Self> {
        let db = LibraDB::open(
            db_root_path,
            true, /* read only */
            None, /* no prune_window */
            RocksdbConfig::default(),
        )?;
        Ok(Librarian {
            ledger_store: Arc::clone(&db.ledger_store),
            transaction_store: Arc::clone(&db.transaction_store),
            state_store: Arc::clone(&db.state_store),
        })
    }

    pub fn get_startup_info(&self) -> Result<StartupInfo> {
        self.ledger_store
            .get_startup_info()?
            .ok_or_else(|| format_err!("DB is empty"))
    }

    pub fn get_committed_version(&self) -> Result<Version> {
        Ok(self
            .get_startup_info()?
            .latest_ledger_info
            .ledger_info()
            .version())
    }

    pub fn scan_txn_by_version(
        &self,
        from_version: Version,
        to_version: Version,
    ) -> Result<Vec<Transaction>> {
        ensure!(
            to_version >= from_version,
            "'from' version {} > 'to' version {}",
            from_version,
            to_version
        );
        let num_txns = to_version - from_version;
        let txn_iter = self
            .transaction_store
            .get_transaction_iter(from_version, num_txns as usize)?;
        txn_iter.collect::<Result<Vec<_>>>()
    }

    pub fn get_txn_by_version(&self, version: Version) -> Result<Transaction> {
        self.transaction_store.get_transaction(version)
    }

    pub fn get_account_state_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountStateBlob>> {
        self.state_store
            .get_account_state_with_proof_by_version(address, version)
            .map(|blob_and_proof| blob_and_proof.0)
    }
}
