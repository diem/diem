// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use move_core_types::{
    account_address::AccountAddress,
    effects::{AccountChangeSet, ChangeSet},
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    resolver::{ModuleResolver, MoveResolver, ResourceResolver},
};
use std::collections::{btree_map, BTreeMap};

/// A dummy storage containing no modules or resources.
#[derive(Debug, Clone)]
pub struct BlankStorage;

impl BlankStorage {
    pub fn new() -> Self {
        Self
    }
}

impl ModuleResolver for BlankStorage {
    type Error = ();

    fn get_module(&self, _module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(None)
    }
}

impl ResourceResolver for BlankStorage {
    type Error = ();

    fn get_resource(
        &self,
        _address: &AccountAddress,
        _tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
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

impl<'a, 'b, S: ModuleResolver> ModuleResolver for DeltaStorage<'a, 'b, S> {
    type Error = S::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Some(account_storage) = self.delta.accounts().get(module_id.address()) {
            if let Some(blob_opt) = account_storage.modules().get(module_id.name()) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_module(module_id)
    }
}

impl<'a, 'b, S: ResourceResolver> ResourceResolver for DeltaStorage<'a, 'b, S> {
    type Error = S::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, S::Error> {
        if let Some(account_storage) = self.delta.accounts().get(address) {
            if let Some(blob_opt) = account_storage.resources().get(tag) {
                return Ok(blob_opt.clone());
            }
        }

        self.base.get_resource(address, tag)
    }
}

impl<'a, 'b, S: MoveResolver> DeltaStorage<'a, 'b, S> {
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
        let (modules, resources) = account_changeset.into_inner();
        apply_changes(&mut self.modules, modules, |module_name| {
            format_err!(
                "Failed to delete module {}: module does not exist.",
                module_name
            )
        })?;

        apply_changes(&mut self.resources, resources, |struct_tag| {
            format_err!(
                "Failed to delete resource {}: resource does not exist.",
                struct_tag
            )
        })?;

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
        for (addr, account_changeset) in changeset.into_inner() {
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

impl ModuleResolver for InMemoryStorage {
    type Error = ();

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Some(account_storage) = self.accounts.get(module_id.address()) {
            return Ok(account_storage.modules.get(module_id.name()).cloned());
        }
        Ok(None)
    }
}

impl ResourceResolver for InMemoryStorage {
    type Error = ();

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Some(account_storage) = self.accounts.get(address) {
            return Ok(account_storage.resources.get(tag).cloned());
        }
        Ok(None)
    }
}
