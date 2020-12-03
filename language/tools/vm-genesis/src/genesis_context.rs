// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use anyhow::Result;
use diem_state_view::StateView;
use diem_types::access_path::AccessPath;
use move_core_types::language_storage::ModuleId;
use std::collections::HashMap;
use vm::CompiledModule;

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

    pub(crate) fn add_module(&mut self, module_id: &ModuleId, module: &CompiledModule) {
        let access_path = AccessPath::from(module_id);
        let mut blob = vec![];
        module
            .serialize(&mut blob)
            .expect("serializing stdlib must work");
        self.data.insert(access_path, blob);
    }
}

impl StateView for GenesisStateView {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        Ok(self.data.get(access_path).cloned())
    }

    fn multi_get(&self, _access_paths: &[AccessPath]) -> Result<Vec<Option<Vec<u8>>>> {
        unimplemented!()
    }

    fn is_genesis(&self) -> bool {
        true
    }
}
