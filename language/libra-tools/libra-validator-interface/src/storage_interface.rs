// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::LibraValidatorInterface;
use anyhow::{anyhow, Result};
use libra_config::config::RocksdbConfig;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, Version},
};
use libradb::LibraDB;
use std::{path::Path, sync::Arc};
use storage_interface::DbReader;

pub struct DBDebuggerInterface(Arc<dyn DbReader>);

impl DBDebuggerInterface {
    pub fn open<P: AsRef<Path> + Clone>(db_root_path: P) -> Result<Self> {
        Ok(Self(Arc::new(LibraDB::open(
            db_root_path,
            true,
            None,
            RocksdbConfig::default(),
        )?)))
    }
}

impl LibraValidatorInterface for DBDebuggerInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountStateBlob>> {
        Ok(self
            .0
            .get_account_state_with_proof_by_version(account, version)?
            .0)
    }

    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>> {
        Ok(self
            .0
            .get_transactions(start, limit, self.get_latest_version()?, false)?
            .transactions)
    }

    fn get_latest_version(&self) -> Result<Version> {
        let (version, _) = self
            .0
            .get_latest_transaction_info_option()?
            .ok_or_else(|| anyhow!("DB doesn't have any transaction."))?;
        Ok(version)
    }

    fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>> {
        Ok(self
            .0
            .get_txn_by_account(account, seq, self.get_latest_version()?, false)?
            .map(|info| info.version))
    }
}
