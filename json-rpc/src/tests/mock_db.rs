// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    contract_event::ContractEvent,
    event::EventKey,
    proof::{TransactionAccumulatorProof, TransactionListProof, TransactionProof},
    transaction::{
        Transaction, TransactionInfo, TransactionListWithProof, TransactionWithProof, Version,
    },
    vm_error::StatusCode,
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

    fn get_txn_by_account(
        &self,
        address: AccountAddress,
        seq_num: u64,
        _ledger_version: u64,
        _fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>, Error> {
        Ok(self
            .all_txns
            .iter()
            .find(|x| {
                if let Some(t) = x.as_signed_user_txn().ok() {
                    t.sender() == address && t.sequence_number() == seq_num
                } else {
                    false
                }
            })
            .map(|x| TransactionWithProof {
                version: 0,
                transaction: x.clone(),
                events: None,
                proof: TransactionProof::new(
                    TransactionAccumulatorProof::new(vec![]),
                    TransactionInfo::new(
                        Default::default(),
                        Default::default(),
                        Default::default(),
                        0,
                        StatusCode::EXECUTED,
                    ),
                ),
            }))
    }

    fn get_latest_version(&self) -> Result<u64, Error> {
        Ok(self.version)
    }

    fn get_transactions(
        &self,
        start_version: u64,
        limit: u64,
        _ledger_version: u64,
        _fetch_events: bool,
    ) -> Result<TransactionListWithProof, Error> {
        let transactions: Vec<Transaction> = self
            .all_txns
            .iter()
            .skip(start_version as usize)
            .take(limit as usize)
            .map(|x| x.clone())
            .collect();
        let first_transaction_version = transactions.first().map(|_| start_version);
        Ok(TransactionListWithProof {
            transactions,
            events: None,
            first_transaction_version,
            proof: TransactionListProof::new_empty(),
        })
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
