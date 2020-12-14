// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module provides mock dbreader for tests.

use crate::{DbReader, Order, StartupInfo, TreeState};
use anyhow::Result;
use diem_crypto::HashValue;
use diem_types::{
    account_address::AccountAddress,
    account_config::AccountResource,
    account_state::AccountState,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    epoch_change::EpochChangeProof,
    event::{EventHandle, EventKey},
    ledger_info::LedgerInfoWithSignatures,
    proof::{AccumulatorConsistencyProof, SparseMerkleProof},
    transaction::{TransactionListWithProof, TransactionWithProof, Version},
};
use move_core_types::move_resource::MoveResource;
use std::convert::TryFrom;

/// This is a mock of the dbreader in tests.
pub struct MockDbReader;

impl DbReader for MockDbReader {
    fn get_epoch_ending_ledger_infos(
        &self,
        _start_epoch: u64,
        _end_epoch: u64,
    ) -> Result<EpochChangeProof> {
        unimplemented!()
    }

    fn get_transactions(
        &self,
        _start_version: Version,
        _batch_size: u64,
        _ledger_version: Version,
        _fetch_events: bool,
    ) -> Result<TransactionListWithProof> {
        unimplemented!()
    }

    /// Returns events by given event key
    fn get_events(
        &self,
        _event_key: &EventKey,
        _start: u64,
        _order: Order,
        _limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>> {
        unimplemented!()
    }

    fn get_block_timestamp(&self, _version: u64) -> Result<u64> {
        unimplemented!()
    }

    fn get_latest_account_state(
        &self,
        _address: AccountAddress,
    ) -> Result<Option<AccountStateBlob>> {
        Ok(Some(get_mock_account_state_blob()))
    }

    /// Returns the latest ledger info.
    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }

    fn get_startup_info(&self) -> Result<Option<StartupInfo>> {
        unimplemented!()
    }

    fn get_txn_by_account(
        &self,
        _address: AccountAddress,
        _seq_num: u64,
        _ledger_version: Version,
        _fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>> {
        unimplemented!()
    }

    fn get_state_proof_with_ledger_info(
        &self,
        _known_version: u64,
        _ledger_info: LedgerInfoWithSignatures,
    ) -> Result<(EpochChangeProof, AccumulatorConsistencyProof)> {
        unimplemented!()
    }

    fn get_state_proof(
        &self,
        _known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        EpochChangeProof,
        AccumulatorConsistencyProof,
    )> {
        unimplemented!()
    }

    fn get_account_state_with_proof(
        &self,
        _address: AccountAddress,
        _version: Version,
        _ledger_version: Version,
    ) -> Result<AccountStateWithProof> {
        unimplemented!()
    }

    fn get_account_state_with_proof_by_version(
        &self,
        _address: AccountAddress,
        _version: Version,
    ) -> Result<(Option<AccountStateBlob>, SparseMerkleProof)> {
        unimplemented!()
    }

    fn get_latest_state_root(&self) -> Result<(Version, HashValue)> {
        unimplemented!()
    }

    fn get_latest_tree_state(&self) -> Result<TreeState> {
        unimplemented!()
    }

    fn get_epoch_ending_ledger_info(
        &self,
        _known_version: u64,
    ) -> Result<LedgerInfoWithSignatures> {
        unimplemented!()
    }
}

fn get_mock_account_state_blob() -> AccountStateBlob {
    let account_resource = AccountResource::new(
        0,
        vec![],
        None,
        None,
        EventHandle::random_handle(0),
        EventHandle::random_handle(0),
    );

    let mut account_state = AccountState::default();
    account_state.insert(
        AccountResource::resource_path(),
        bcs::to_bytes(&account_resource).unwrap(),
    );

    AccountStateBlob::try_from(&account_state).unwrap()
}
