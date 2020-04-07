// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod crypto_kv_storage;
mod crypto_storage;
mod error;
mod in_memory;
mod kv_storage;
mod on_disk;
mod policy;
mod storage;
mod value;
mod vault;

pub use crate::{
    crypto_kv_storage::CryptoKVStorage,
    crypto_storage::{CryptoStorage, PublicKeyResponse},
    error::Error,
    in_memory::{InMemoryStorage, InMemoryStorageInternal},
    kv_storage::{GetResponse, KVStorage},
    on_disk::{OnDiskStorage, OnDiskStorageInternal},
    policy::{Capability, Identity, Permission, Policy},
    storage::Storage,
    value::Value,
    vault::VaultStorage,
};

#[cfg(test)]
mod tests;
