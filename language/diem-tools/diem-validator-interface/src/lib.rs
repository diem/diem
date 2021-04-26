// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod json_rpc_interface;
mod storage_interface;

pub use crate::storage_interface::DBDebuggerInterface;
pub use json_rpc_interface::JsonRpcDebuggerInterface;

use anyhow::{anyhow, Result};
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_state::AccountState,
    contract_event::EventWithProof,
    event::EventKey,
    on_chain_config::ValidatorSet,
    transaction::{Transaction, Version},
};
use move_binary_format::file_format::CompiledModule;

pub trait DiemValidatorInterface: Sync {
    fn get_account_state_by_version(
        &self,
        account: AccountAddress,
        version: Version,
    ) -> Result<Option<AccountState>>;
    fn get_events(&self, key: &EventKey, start_seq: u64, limit: u64)
        -> Result<Vec<EventWithProof>>;
    fn get_committed_transactions(&self, start: Version, limit: u64) -> Result<Vec<Transaction>>;
    fn get_latest_version(&self) -> Result<Version>;
    fn get_version_by_account_sequence(
        &self,
        account: AccountAddress,
        seq: u64,
    ) -> Result<Option<Version>>;

    fn get_diem_framework_modules_by_version(
        &self,
        version: Version,
    ) -> Result<Vec<CompiledModule>> {
        let mut acc = vec![];
        for module_bytes in self
            .get_account_state_by_version(account_config::CORE_CODE_ADDRESS, version)?
            .ok_or_else(|| anyhow!("Failure reading diem root address state"))?
            .get_modules()
        {
            acc.push(
                CompiledModule::deserialize(module_bytes)
                    .map_err(|e| anyhow!("Failure deserializing module: {:?}", e))?,
            )
        }
        Ok(acc)
    }

    /// Get the account states of the most critical accounts, including:
    /// 1. Diem Framework code address
    /// 2. Diem Root address
    /// 3. Treasury Compliance address
    /// 4. All validator addresses
    fn get_admin_accounts(&self, version: Version) -> Result<Vec<(AccountAddress, AccountState)>> {
        let mut result = vec![];
        let diem_root = self
            .get_account_state_by_version(account_config::diem_root_address(), version)?
            .ok_or_else(|| anyhow!("diem_root_address doesn't exist"))?;

        // Get all validator accounts
        let validators = diem_root
            .get_config::<ValidatorSet>()?
            .ok_or_else(|| anyhow!("validator_config doesn't exist"))?;

        // Get code account, diem_root and treasury compliance accounts.
        result.push((
            account_config::CORE_CODE_ADDRESS,
            self.get_account_state_by_version(account_config::CORE_CODE_ADDRESS, version)?
                .ok_or_else(|| anyhow!("core_code_address doesn't exist"))?,
        ));
        result.push((account_config::diem_root_address(), diem_root));
        result.push((
            account_config::treasury_compliance_account_address(),
            self.get_account_state_by_version(
                account_config::treasury_compliance_account_address(),
                version,
            )?
            .ok_or_else(|| anyhow!("treasury_compliance_account doesn't exist"))?,
        ));

        // Get all validator accounts
        for validator_info in validators.payload() {
            let addr = *validator_info.account_address();
            result.push((
                addr,
                self.get_account_state_by_version(addr, version)?
                    .ok_or_else(|| anyhow!("validator {:?} doesn't exist", addr))?,
            ));
        }
        Ok(result)
    }
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
                Some(blob) => blob.get(&access_path.path).cloned(),
                None => None,
            },
        )
    }

    fn is_genesis(&self) -> bool {
        false
    }
}
