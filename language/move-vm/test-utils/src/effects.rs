// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use std::collections::btree_map::{self, BTreeMap};

/// A collection of changes to modules and resources under a Move account.
#[derive(Debug, Clone)]
pub struct AccountChangeSet {
    pub modules: BTreeMap<Identifier, Option<Vec<u8>>>,
    pub resources: BTreeMap<StructTag, Option<Vec<u8>>>,
}

fn publish_checked<K, V, F>(map: &mut BTreeMap<K, Option<V>>, k: K, v: V, make_err: F) -> Result<()>
where
    K: Ord,
    F: FnOnce() -> Error,
{
    match map.entry(k) {
        btree_map::Entry::Occupied(entry) => {
            let r = entry.into_mut();
            match r {
                Some(_) => return Err(make_err()),
                None => *r = Some(v),
            }
        }
        btree_map::Entry::Vacant(entry) => {
            entry.insert(Some(v));
        }
    }
    Ok(())
}

fn unpublish_checked<K, V, F>(map: &mut BTreeMap<K, Option<V>>, k: K, make_err: F) -> Result<()>
where
    K: Ord,
    F: FnOnce() -> Error,
{
    match map.entry(k) {
        btree_map::Entry::Occupied(entry) => {
            let r = entry.into_mut();
            match r {
                Some(_) => *r = None,
                None => return Err(make_err()),
            }
        }
        btree_map::Entry::Vacant(entry) => {
            entry.insert(None);
        }
    }
    Ok(())
}

impl AccountChangeSet {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            resources: BTreeMap::new(),
        }
    }

    pub fn squash(&mut self, other: Self) -> Result<()> {
        for (name, blob_opt) in other.modules {
            match blob_opt {
                Some(blob) => self.publish_module(name, blob)?,
                None => self.unpublish_module(name)?,
            }
        }
        for (struct_tag, blob_opt) in other.resources {
            match blob_opt {
                Some(blob) => self.publish_resource(struct_tag, blob)?,
                None => self.unpublish_resource(struct_tag)?,
            }
        }
        Ok(())
    }

    pub fn publish_or_overwrite_module(&mut self, name: Identifier, blob: Vec<u8>) {
        self.modules.insert(name, Some(blob));
    }

    pub fn publish_or_overwrite_resource(&mut self, struct_tag: StructTag, blob: Vec<u8>) {
        self.resources.insert(struct_tag, Some(blob));
    }

    pub fn publish_module(&mut self, name: Identifier, blob: Vec<u8>) -> Result<()> {
        publish_checked(&mut self.modules, name, blob, || {
            format_err!("module already published")
        })
    }

    pub fn unpublish_module(&mut self, name: Identifier) -> Result<()> {
        unpublish_checked(&mut self.modules, name, || {
            format_err!("module already unpublished")
        })
    }

    pub fn publish_resource(&mut self, struct_tag: StructTag, blob: Vec<u8>) -> Result<()> {
        publish_checked(&mut self.resources, struct_tag, blob, || {
            format_err!("resource already published")
        })
    }

    pub fn unpublish_resource(&mut self, struct_tag: StructTag) -> Result<()> {
        unpublish_checked(&mut self.resources, struct_tag, || {
            format_err!("resource already unpublished")
        })
    }
}

/// A collection of changes to a Move state.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    pub accounts: BTreeMap<AccountAddress, AccountChangeSet>,
}

impl ChangeSet {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    fn get_or_insert_account_changeset(&mut self, addr: AccountAddress) -> &mut AccountChangeSet {
        match self.accounts.entry(addr) {
            btree_map::Entry::Occupied(entry) => entry.into_mut(),
            btree_map::Entry::Vacant(entry) => entry.insert(AccountChangeSet::new()),
        }
    }

    pub fn publish_or_overwrite_module(&mut self, module_id: ModuleId, blob: Vec<u8>) {
        let (addr, name) = module_id.into();
        let account_changeset = self.get_or_insert_account_changeset(addr);
        account_changeset.publish_or_overwrite_module(name, blob)
    }

    pub fn publish_module(&mut self, module_id: ModuleId, blob: Vec<u8>) -> Result<()> {
        let (addr, name) = module_id.into();
        let account_changeset = self.get_or_insert_account_changeset(addr);
        account_changeset.publish_module(name, blob)
    }

    pub fn unpublish_module(&mut self, module_id: ModuleId) -> Result<()> {
        let (addr, name) = module_id.into();
        let account_changeset = self.get_or_insert_account_changeset(addr);
        account_changeset.unpublish_module(name)
    }

    pub fn publish_or_overwrite_resource(
        &mut self,
        addr: AccountAddress,
        struct_tag: StructTag,
        blob: Vec<u8>,
    ) {
        self.get_or_insert_account_changeset(addr)
            .publish_or_overwrite_resource(struct_tag, blob)
    }

    pub fn publish_resource(
        &mut self,
        addr: AccountAddress,
        struct_tag: StructTag,
        blob: Vec<u8>,
    ) -> Result<()> {
        self.get_or_insert_account_changeset(addr)
            .publish_resource(struct_tag, blob)
    }

    pub fn unpublish_resource(
        &mut self,
        addr: AccountAddress,
        struct_tag: StructTag,
    ) -> Result<()> {
        self.get_or_insert_account_changeset(addr)
            .unpublish_resource(struct_tag)
    }

    pub fn squash(&mut self, other: Self) -> Result<()> {
        for (addr, other_account_changeset) in other.accounts {
            match self.accounts.entry(addr) {
                btree_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().squash(other_account_changeset)?;
                }
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(other_account_changeset);
                }
            }
        }
        Ok(())
    }
}

pub type Event = (Vec<u8>, u64, TypeTag, Vec<u8>);
