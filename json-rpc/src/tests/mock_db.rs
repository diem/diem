// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_types::{
    account_address::AccountAddress, account_state_blob::AccountStateBlob,
    contract_event::ContractEvent, event::EventKey, transaction::Transaction, transaction::Version,
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
    pub events: Vec<(u64, ContractEvent)>,
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

    fn get_events(
        &self,
        key: &EventKey,
        start: u64,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        let events = self
            .events
            .iter()
            .filter(|(_, e)| {
                e.key() == key
                    && start <= e.sequence_number()
                    && e.sequence_number() < start + limit
            })
            .cloned()
            .collect();
        Ok(events)
    }
}
