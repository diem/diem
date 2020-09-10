// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use std::collections::btree_map::BTreeMap;

#[derive(Debug, Clone)]
pub enum Op {
    Write(Vec<u8>),
    Deletion,
}

/// A collection of changes to modules and resources under a Move account.
#[derive(Debug, Clone)]
pub struct AccountChangeSet {
    pub modules: BTreeMap<Identifier, Op>,
    pub resources: BTreeMap<StructTag, Op>,
}

/// A collection of changes to a Move state.
#[derive(Debug, Clone)]
pub struct ChangeSet {
    pub accounts: BTreeMap<AccountAddress, AccountChangeSet>,
}

pub type Event = (Vec<u8>, u64, TypeTag, Vec<u8>);
