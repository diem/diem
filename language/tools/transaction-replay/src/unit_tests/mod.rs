// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod bisection_tests;

use crate::StorageDebuggerInterface;
use anyhow::{bail, Result};
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, Version},
};
use std::collections::HashMap;

pub struct TestInterface {
    state_db: HashMap<(Version, AccountAddress), AccountStateBlob>,
    transaction_store: Vec<Transaction>,
    latest_version: u64,
}

impl TestInterface {
    #[allow(dead_code)]
    pub fn new(
        state_db: HashMap<(Version, AccountAddress), AccountStateBlob>,
        transaction_store: Vec<Transaction>,
        latest_version: u64,
    ) -> Self {
        Self {
            state_db,
            transaction_store,
            latest_version,
        }
    }

    pub fn empty(version: u64) -> Self {
        Self {
            state_db: HashMap::new(),
            transaction_store: vec![],
            latest_version: version,
        }
    }
}

impl StorageDebuggerInterface for TestInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountStateBlob>> {
        Ok(self.state_db.get(&(version, account)).cloned())
    }

    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>> {
        if start + limit >= self.transaction_store.len() as u64 {
            bail!("Unexpected length")
        }
        let mut result = vec![];
        for i in start..start + limit {
            result.push(self.transaction_store[i as usize].clone())
        }
        Ok(result)
    }

    fn get_latest_version(&self) -> Result<Version> {
        Ok(self.latest_version)
    }

    fn get_version_by_account_sequence(
        &self,
        _account: AccountAddress,
        _seq: u64,
    ) -> Result<Option<Version>> {
        unimplemented!()
    }
}
