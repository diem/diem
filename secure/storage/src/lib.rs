// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod error;
mod in_memory;
mod on_disk;
mod permissions;
mod storage;
mod value;

pub use crate::{
    error::Error,
    in_memory::InMemoryStorage,
    on_disk::OnDiskStorage,
    permissions::{Id, Permission, Permissions},
    storage::Storage,
    value::Value,
};

#[cfg(test)]
mod tests;
