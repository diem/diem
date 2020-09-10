use crate::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use anyhow::Result;
use std::collections::btree_map::{self, BTreeMap};

/// A collection of changes to modules and resources under a Move account.
#[derive(Clone)]
pub struct AccountChangeSet {
    pub modules: BTreeMap<Identifier, Option<Vec<u8>>>,
    pub resources: BTreeMap<StructTag, Option<Vec<u8>>>,
}

impl AccountChangeSet {
    pub fn squash(&mut self, other: Self) -> Result<()> {
        self.modules.extend(other.modules);
        self.resources.extend(other.resources);
        Ok(())
    }
}

/// A collection of changes to a Move state.
#[derive(Clone)]
pub struct ChangeSet {
    pub accounts: BTreeMap<AccountAddress, AccountChangeSet>,
}

impl ChangeSet {
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
