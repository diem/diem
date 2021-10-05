// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! This crate defines [`trait StateView`](StateView).

use anyhow::Result;
use diem_crypto::HashValue;
use diem_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
    transaction::Version,
};

/// `StateView` is a trait that defines a read-only snapshot of the global state. It is passed to
/// the VM for transaction execution, during which the VM is guaranteed to read anything at the
/// given state.
pub trait StateView: Sync {
    /// For logging and debugging purpose, identifies what this view is for.
    fn id(&self) -> StateViewId {
        StateViewId::Miscellaneous
    }

    /// Gets the state for a single access path.
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>>;

    /// Gets the account state for `account` returning all access paths and resource blobs underneath it
    fn get_account_state(&self, account: AccountAddress) -> Result<Option<AccountState>>;

    /// VM needs this method to know whether the current state view is for genesis state creation.
    /// Currently TransactionPayload::WriteSet is only valid for genesis state creation.
    fn is_genesis(&self) -> bool;
}

#[derive(Copy, Clone)]
pub enum StateViewId {
    /// State-sync applying a chunk of transactions.
    ChunkExecution { first_version: Version },
    /// LEC applying a block.
    BlockExecution { block_id: HashValue },
    /// VmValidator verifying incoming transaction.
    TransactionValidation { base_version: Version },
    /// For test, db-bootstrapper, etc. Usually not aimed to pass to VM.
    Miscellaneous,
}
