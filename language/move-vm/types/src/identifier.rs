// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A bunch of helper functions to fetch the storage key for move resources and values.

use libra_types::{
    access_path::{AccessPath, Accesses},
    account_address::AccountAddress,
    language_storage::{ResourceKey, StructTag},
};

/// Get the AccessPath to a resource stored under `address` with type name `tag`
pub fn create_access_path(address: AccountAddress, tag: StructTag) -> AccessPath {
    let resource_tag = ResourceKey::new(address, tag);
    AccessPath::resource_access_path(&resource_tag, &Accesses::empty())
}
