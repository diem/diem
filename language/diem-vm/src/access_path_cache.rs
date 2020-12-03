// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_types::access_path::AccessPath;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag},
};
use std::collections::btree_map::{self, BTreeMap};

pub trait AccessPathCache {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath;
    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath;
}

impl AccessPathCache for () {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath {
        AccessPath::from(&module_id)
    }

    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath {
        AccessPath::new(address, struct_tag.access_vector())
    }
}

#[derive(Clone)]
pub struct BTreeAccessPathCache {
    modules: BTreeMap<ModuleId, Vec<u8>>,
    resources: BTreeMap<StructTag, Vec<u8>>,
}

impl AccessPathCache for BTreeAccessPathCache {
    fn get_module_path(&mut self, module_id: ModuleId) -> AccessPath {
        let addr = *module_id.address();
        let access_vec = match self.modules.entry(module_id) {
            btree_map::Entry::Vacant(entry) => {
                let v = entry.key().access_vector();
                entry.insert(v).clone()
            }
            btree_map::Entry::Occupied(entry) => entry.get().clone(),
        };
        AccessPath::new(addr, access_vec)
    }

    fn get_resource_path(&mut self, address: AccountAddress, struct_tag: StructTag) -> AccessPath {
        let access_vec = match self.resources.entry(struct_tag) {
            btree_map::Entry::Vacant(entry) => {
                let v = entry.key().access_vector();
                entry.insert(v).clone()
            }
            btree_map::Entry::Occupied(entry) => entry.get().clone(),
        };
        AccessPath::new(address, access_vec)
    }
}

impl BTreeAccessPathCache {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            resources: BTreeMap::new(),
        }
    }
}
