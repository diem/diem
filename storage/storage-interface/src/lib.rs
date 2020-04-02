// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_types::{
    account_address::AccountAddress,
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::ContractEvent,
    event::EventKey,
    ledger_info::LedgerInfoWithSignatures,
    proof::AccumulatorConsistencyProof,
    transaction::{TransactionListWithProof, TransactionWithProof, Version},
    validator_change::ValidatorChangeProof,
};
use storage_proto::StartupInfo;

/// Trait that is implemented by a DB that supports certain public (to client) read APIs
/// expected of a Libra DB
pub trait DbReader: Send + Sync {
    /// See [`LibraDB::get_transactions`].
    ///
    /// [`LibraDB::get_transactions`]: ../libradb/struct.LibraDB.html#method.get_transactions
    fn get_transactions(
        &self,
        start_version: Version,
        batch_size: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<TransactionListWithProof>;

    /// Returns events by given event key
    fn get_events(
        &self,
        event_key: &EventKey,
        start: u64,
        ascending: bool,
        limit: u64,
    ) -> Result<Vec<(u64, ContractEvent)>>;

    /// See [`LibraDB::get_latest_account_state`].
    ///
    /// [`LibraDB::get_latest_account_state`]:
    /// ../libradb/struct.LibraDB.html#method.get_latest_account_state
    fn get_latest_account_state(&self, address: AccountAddress)
        -> Result<Option<AccountStateBlob>>;

    /// Returns the latest ledger info.
    fn get_latest_ledger_info(&self) -> Result<LedgerInfoWithSignatures>;

    /// Returns the latest ledger info.
    fn get_latest_version(&self) -> Result<Version> {
        Ok(self.get_latest_ledger_info()?.ledger_info().version())
    }

    /// Returns the latest version and committed block timestamp
    fn get_latest_commit_metadata(&self) -> Result<(Version, u64)> {
        let ledger_info_with_sig = self.get_latest_ledger_info()?;
        let ledger_info = ledger_info_with_sig.ledger_info();
        Ok((ledger_info.version(), ledger_info.timestamp_usecs()))
    }

    /// Gets information needed from storage during the main node startup.
    /// See [`LibraDB::get_startup_info`].
    ///
    /// [`LibraDB::get_startup_info`]:
    /// ../libradb/struct.LibraDB.html#method.get_startup_info
    fn get_startup_info(&self) -> Result<Option<StartupInfo>>;

    fn get_txn_by_account(
        &self,
        address: AccountAddress,
        seq_num: u64,
        ledger_version: Version,
        fetch_events: bool,
    ) -> Result<Option<TransactionWithProof>>;

    /// Returns proof of new state relative to version known to client
    fn get_state_proof(
        &self,
        known_version: u64,
    ) -> Result<(
        LedgerInfoWithSignatures,
        ValidatorChangeProof,
        AccumulatorConsistencyProof,
    )>;

    /// Returns the account state corresponding to the given version and account address with proof
    /// based on `ledger_version`
    fn get_account_state_with_proof(
        &self,
        address: AccountAddress,
        version: Version,
        ledger_version: Version,
    ) -> Result<AccountStateWithProof>;
}
