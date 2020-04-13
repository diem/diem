// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::HashValue;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    block_info::BlockInfo,
    contract_event::ContractEvent,
    event::EventKey,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    proof::{
        AccumulatorConsistencyProof, AccumulatorRangeProof, SparseMerkleProof,
        TransactionAccumulatorProof, TransactionListProof, TransactionProof,
    },
    transaction::{
        Transaction, TransactionInfo, TransactionListWithProof, TransactionWithProof, Version,
    },
    validator_change::ValidatorChangeProof,
    vm_error::StatusCode,
};
use std::collections::BTreeMap;
use storage_interface::DbReader;
use storage_proto::{StartupInfo, TreeState};

/// Lightweight mock of LibraDB
#[derive(Clone)]
pub(crate) struct MockLibraDB {
    pub version: u64,
    pub timestamp: u64,
    pub all_accounts: BTreeMap<AccountAddress, AccountStateBlob>,
    pub all_txns: Vec<(Transaction, StatusCode)>,
    pub events: Vec<(u64, ContractEvent)>,
    pub account_state_with_proof: Vec<AccountStateWithProof>,
}

impl DbReader for MockLibraDB {
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

    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        Ok(LedgerInfoWithSignatures::new(
            LedgerInfo::new(
                BlockInfo::new(
                    0,
                    self.version,
                    HashValue::zero(),
                    HashValue::zero(),
                    self.version,
                    self.timestamp,
                    None,
                ),
                HashValue::zero(),
            ),
            BTreeMap::new(),
        ))
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
            .find(|(_, (x, _))| {
                if let Ok(t) = x.as_signed_user_txn() {
                    t.sender() == address && t.sequence_number() == seq_num
                } else {
                    false
                }
            })
            .map(|(v, (x, status))| TransactionWithProof {
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
                        *status,
                    ),
                ),
            }))
    }

    fn get_transactions(
        &self,
        start_version: u64,
        limit: u64,
        _ledger_version: u64,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof, Error> {
        let mut transactions = vec![];
        let mut txn_infos = vec![];
        self.all_txns
            .iter()
            .skip(start_version as usize)
            .take(limit as usize)
            .for_each(|(t, status)| {
                transactions.push(t.clone());
                txn_infos.push(TransactionInfo::new(
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    0,
                    *status,
                ));
            });
        let first_transaction_version = transactions.first().map(|_| start_version);
        let proof = TransactionListProof::new(AccumulatorRangeProof::new_empty(), txn_infos);

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
            proof,
        })
    }

    fn get_events(
        &self,
        key: &EventKey,
        start: u64,
        _ascending: bool,
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
        let li = self.get_latest_ledger_info()?;
        let proofs = self.get_state_proof_with_ledger_info(known_version, li.clone())?;
        Ok((
            LedgerInfoWithSignatures::new(li.ledger_info().clone(), BTreeMap::new()),
            proofs.0,
            proofs.1,
        ))
    }

    fn get_state_proof_with_ledger_info(
        &self,
        _known_version: u64,
        _ledger_info: LedgerInfoWithSignatures,
    ) -> Result<(ValidatorChangeProof, AccumulatorConsistencyProof)> {
        Ok((
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

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        address: AccountAddress,
        _version: u64,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        Ok((
            self.get_latest_account_state(address)?,
            SparseMerkleProof::new(None, vec![]),
        ))
    }

    fn get_latest_state_root(&self) -> Result<(u64, HashValue)> {
        unimplemented!()
    }

    fn get_latest_tree_state(&self) -> Result<TreeState> {
        unimplemented!()
    }

    fn get_epoch_change_ledger_infos(
        &self,
        _start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<ValidatorChangeProof> {
        unimplemented!()
    }

    fn batch_fetch_config(&self, _access_paths: Vec<AccessPath>) -> Result<Vec<Vec<u8>>> {
        unimplemented!()
    }

    fn get_ledger_info(&self, _: u64) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }
}
