// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::HashValue;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    block_info::BlockInfo,
    contract_event::ContractEvent,
    event::EventKey,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::{
        AccumulatorConsistencyProof, TransactionAccumulatorProof, TransactionListProof,
        TransactionProof,
    },
    transaction::{
        Transaction, TransactionInfo, TransactionListWithProof, TransactionWithProof, Version,
    },
    validator_change::ValidatorChangeProof,
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
    pub account_state_with_proof: Vec<AccountStateWithProof>,
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
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>, Error> {
        Ok(self
            .all_txns
            .iter()
            .enumerate()
            .find(|(_, x)| {
                if let Ok(t) = x.as_signed_user_txn() {
                    t.sender() == address && t.sequence_number() == seq_num
                } else {
                    false
                }
            })
            .map(|(v, x)| TransactionWithProof {
                version: v as u64,
                transaction: x.clone(),
                events: if fetch_events {
                    Some(
                        self.events
                            .iter()
                            .filter(|(ev, _)| *ev == v as u64)
                            .map(|(_, e)| e)
                            .cloned()
                            .collect(),
                    )
                } else {
                    None
                },
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
        fetch_events: bool,
    ) -> Result<TransactionListWithProof, Error> {
        let transactions: Vec<Transaction> = self
            .all_txns
            .iter()
            .skip(start_version as usize)
            .take(limit as usize)
            .cloned()
            .collect();
        let first_transaction_version = transactions.first().map(|_| start_version);
        Ok(TransactionListWithProof {
            transactions,
            events: if fetch_events {
                Some(
                    (start_version..start_version + limit)
                        .map(|version| {
                            self.events
                                .iter()
                                .filter(|(v, _)| *v == version)
                                .map(|(_, e)| e)
                                .cloned()
                                .collect()
                        })
                        .collect(),
                )
            } else {
                None
            },
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

    fn get_state_proof(
        &self,
        known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )> {
        let li = LedgerInfo::new(
            BlockInfo::new(
                0,
                known_version,
                HashValue::zero(),
                HashValue::zero(),
                known_version,
                0,
                None,
            ),
            HashValue::zero(),
        );
        Ok((
            LedgerInfoWithSignatures::new(li, BTreeMap::new()),
            ValidatorChangeProof::new(vec![], false),
            AccumulatorConsistencyProof::new(vec![]),
        ))
    }

    fn get_account_state_with_proof(
        &self,
        _address: AccountAddress,
        _version: Version,
        _ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        Ok(self.account_state_with_proof[0].clone())
    }
}
