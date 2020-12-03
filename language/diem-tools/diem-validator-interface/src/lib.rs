// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod json_rpc_interface;
mod storage_interface;

pub use crate::storage_interface::DBDebuggerInterface;
pub use json_rpc_interface::JsonRpcDebuggerInterface;

use anyhow::{bail, Result};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_state::AccountState,
    account_state_blob::AccountStateBlob,
    transaction::{Transaction, Version},
};
use std::convert::TryFrom;

pub trait DiemValidatorInterface {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountStateBlob>>;
    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>>;
    fn get_latest_version(&self) -> Result<Version>;
    fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>>;
}

pub struct DebuggerStateView<'a> {
    db: &'a dyn DiemValidatorInterface,
    version: Version,
}

impl<'a> DebuggerStateView<'a> {
    pub fn new(db: &'a dyn DiemValidatorInterface, version: Version) -> Self {
        Self { db, version }
    }
}

impl<'a> StateView for DebuggerStateView<'a> {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        if self.version == 0 {
            return Ok(None);
        }
        Ok(
            match self
                .db
                .get_account_state_by_version(access_path.address, self.version - 1)?
            {
                Some(blob) => AccountState::try_from(&blob)?
                    .get(&access_path.path)
                    .cloned(),
                None => None,
            },
        )
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        bail!("unimplemeneted")
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
