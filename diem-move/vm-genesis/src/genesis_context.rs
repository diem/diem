// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use diem_state_view::StateView;
use diem_types::{
    access_path::AccessPath, account_address::AccountAddress, account_state::AccountState,
};
use move_core_types::language_storage::ModuleId;
use std::collections::HashMap;

// `StateView` has no data given we are creating the genesis
pub(crate) struct GenesisStateView {
    data: HashMap<AccessPath, Vec<u8>>,
}

impl GenesisStateView {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn add_module(&mut self, module_id: &ModuleId, blob: &[u8]) {
        let access_path = AccessPath::from(module_id);
        self.data.insert(access_path, blob.to_vec());
    }
}

impl StateView for GenesisStateView {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(access_path).cloned())
    }

    fn get_account_state(&self, account: AccountAddress) -> Result<Option<AccountState>> {
        let mut account_data = self
            .data
            .iter()
            .filter(|(k, _)| k.address == account)
            .peekable();
        if account_data.peek().is_none() {
            return Ok(None);
        }

        let mut account_state = AccountState::default();
        for (ap, bytes) in account_data {
            if ap.address == account {
                account_state.insert(ap.path.to_vec(), bytes.to_vec());
            }
        }
        Ok(Some(account_state))
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
