// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_types::{
    account_address::AccountAddress, account_state_blob::AccountStateBlob,
    transaction::Transaction, transaction::Version,
};
use libradb::LibraDBTrait;
use std::collections::BTreeMap;

/// Lightweight mock of LibraDB
#[derive(Clone)]
pub(crate) struct MockLibraDB {
    pub version: u64,
    pub timestamp: u64,
    pub all_accounts: BTreeMap<AccountAddress, AccountStateBlob>,
    pub all_txns: Vec<Transaction>,
}

impl LibraDBTrait for MockLibraDB {
    fn get_latest_account_state(
        &self,
        address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        if let Some(blob) = self.all_accounts.get(&address) {
            Ok(Some(blob.clone()))
        } else {
            Ok(None)
        }
    }

    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)> {
        Ok((self.version, self.timestamp))
    }
}
