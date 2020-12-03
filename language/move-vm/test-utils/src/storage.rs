// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
};
use move_vm_runtime::data_cache::RemoteCache;
// use move_vm_txn_effect_converter::convert_txn_effects_to_move_changeset_and_events;
use std::collections::{btree_map, BTreeMap};
use vm::errors::{PartialVMResult, VMResult};

use crate::effects::{AccountChangeSet, ChangeSet};

/// A dummy storage containing no modules or resources.
#[derive(Debug, Clone)]
pub struct BlankStorage;

impl BlankStorage {
    pub fn new() -> Self {
        Self
    }
}

impl RemoteCache for BlankStorage {
    fn get_module(&self, _module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        Ok(None)
    }

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        Ok(None)
    }
}

// A storage adapter created by stacking a change set on top of an existing storage backend.
/// The new storage can be used for additional computations without modifying the base.
#[derive(Debug, Clone)]
pub struct DeltaStorage<'a, 'b, S> {
    base: &'a S,
    delta: &'b ChangeSet,
}

impl<'a, 'b, S: RemoteCache> RemoteCache for DeltaStorage<'a, 'b, S> {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.delta.accounts.get(module_id.address()) {
            if let Some(blob_opt) = account_storage.modules.get(module_id.name()) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_module(module_id)
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.delta.accounts.get(address) {
            if let Some(blob_opt) = account_storage.resources.get(tag) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_resource(address, tag)
    }
}

impl<'a, 'b, S: RemoteCache> DeltaStorage<'a, 'b, S> {
    pub fn new(base: &'a S, delta: &'b ChangeSet) -> Self {
        Self { base, delta }
    }
}

/// Simple in-memory storage for modules and resources under an account.
#[derive(Debug, Clone)]
struct InMemoryAccountStorage {
    resources: BTreeMap<StructTag, Vec<u8>>,
    modules: BTreeMap<Identifier, Vec<u8>>,
}

/// Simple in-memory storage that can be used as a Move VM storage backend for testing purposes.
#[derive(Debug, Clone)]
pub struct InMemoryStorage {
    accounts: BTreeMap<AccountAddress, InMemoryAccountStorage>,
}

fn apply_changes<K, V, F, E>(
    tree: &mut BTreeMap<K, V>,
    changes: impl IntoIterator<Item = (K, Option<V>)>,
    make_err: F,
) -> std::result::Result<(), E>
where
    K: Ord,
    F: FnOnce(K) -> E,
{
    for (k, v_opt) in changes.into_iter() {
        match (tree.entry(k), v_opt) {
            (btree_map::Entry::Vacant(entry), None) => return Err(make_err(entry.into_key())),
            (btree_map::Entry::Vacant(entry), Some(v)) => {
                entry.insert(v);
            }
            (btree_map::Entry::Occupied(entry), None) => {
                entry.remove();
            }
            (btree_map::Entry::Occupied(entry), Some(v)) => {
                *entry.into_mut() = v;
            }
        }
    }
    Ok(())
}

impl InMemoryAccountStorage {
    fn apply(&mut self, account_changeset: AccountChangeSet) -> Result<()> {
        apply_changes(
            &mut self.modules,
            account_changeset.modules,
            |module_name| {
                format_err!(
                    "Failed to delete module {}: module does not exist.",
                    module_name
                )
            },
        )?;

        apply_changes(
            &mut self.resources,
            account_changeset.resources,
            |struct_tag| {
                format_err!(
                    "Failed to delete resource {}: resource does not exist.",
                    struct_tag
                )
            },
        )?;

        Ok(())
    }

    fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            resources: BTreeMap::new(),
        }
    }
}

impl InMemoryStorage {
    pub fn apply(&mut self, changeset: ChangeSet) -> Result<()> {
        for (addr, account_changeset) in changeset.accounts {
            match self.accounts.entry(addr) {
                btree_map::Entry::Occupied(entry) => {
                    entry.into_mut().apply(account_changeset)?;
                }
                btree_map::Entry::Vacant(entry) => {
                    let mut account_storage = InMemoryAccountStorage::new();
                    account_storage.apply(account_changeset)?;
                    entry.insert(account_storage);
                }
            }
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    pub fn publish_or_overwrite_module(&mut self, module_id: ModuleId, blob: Vec<u8>) {
        let mut delta = ChangeSet::new();
        delta.publish_module(module_id, blob).unwrap();
        self.apply(delta).unwrap();
    }

    pub fn publish_or_overwrite_resource(
        &mut self,
        addr: AccountAddress,
        struct_tag: StructTag,
        blob: Vec<u8>,
    ) {
        let mut delta = ChangeSet::new();
        delta.publish_resource(addr, struct_tag, blob).unwrap();
        self.apply(delta).unwrap();
    }
}

impl RemoteCache for InMemoryStorage {
    fn get_module(&self, module_id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.accounts.get(module_id.address()) {
            return Ok(account_storage.modules.get(module_id.name()).cloned());
        }
        Ok(None)
    }

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        if let Some(account_storage) = self.accounts.get(address) {
            return Ok(account_storage.resources.get(tag).cloned());
        }
        Ok(None)
    }
}
