// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;
mod in_memory;
mod on_disk;
mod permissions;
mod storage;
mod value;
mod vault;

pub use crate::{
    error::Error,
    in_memory::InMemoryStorage,
    on_disk::OnDiskStorage,
    permissions::{Id, Permission, Permissions},
    storage::Storage,
    value::Value,
    vault::VaultStorage,
};

#[cfg(test)]
mod tests;
