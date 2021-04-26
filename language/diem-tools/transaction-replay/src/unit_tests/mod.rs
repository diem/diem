// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod bisection_tests;

use crate::DiemValidatorInterface;
use anyhow::{bail, Result};
use diem_types::{
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    contract_event::EventWithProof,
    event::EventKey,
    transaction::{Transaction, Version, WriteSetPayload},
    write_set::WriteOp,
};
use std::{collections::HashMap, convert::TryFrom};
use vm_genesis::{generate_genesis_change_set_for_testing, GenesisOptions};

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

    pub fn genesis() -> Self {
        let changeset = generate_genesis_change_set_for_testing(GenesisOptions::Compiled);
        let mut state_db = HashMap::new();
        for (ap, op) in changeset.write_set().iter() {
            match op {
                WriteOp::Value(v) => state_db
                    .entry((0, ap.address))
                    .or_insert_with(AccountState::default)
                    .insert(ap.path.clone(), v.clone()),
                _ => panic!("Unexpected delete"),
            };
        }
        Self {
            state_db: state_db
                .into_iter()
                .map(|(k, v)| (k, AccountStateBlob::try_from(&v).unwrap()))
                .collect(),
            transaction_store: vec![Transaction::GenesisTransaction(WriteSetPayload::Direct(
                changeset,
            ))],
            latest_version: 1,
        }
    }
}

impl DiemValidatorInterface for TestInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountState>> {
        self.state_db
            .get(&(version, account))
            .map(AccountState::try_from)
            .transpose()
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

    fn get_events(
        &self,
        _key: &EventKey,
        _start_seq: u64,
        _limit: u64,
    ) -> Result<Vec<EventWithProof>> {
        unimplemented!()
    }

    fn get_version_by_account_sequence(
        &self,
        _account: AccountAddress,
        _seq: u64,
    ) -> Result<Option<Version>> {
        unimplemented!()
    }
}
