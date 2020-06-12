// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::SecureBackend;
use std::convert::From;

pub mod config;
mod crypto_kv_storage;
mod crypto_storage;
mod error;
mod github;
mod in_memory;
mod kv_storage;
mod namespaced_storage;
mod on_disk;
mod policy;
mod storage;
mod value;
mod vault;

pub use crate::{
    crypto_kv_storage::CryptoKVStorage,
    crypto_storage::{CryptoStorage, PublicKeyResponse},
    error::Error,
    github::GitHubStorage,
    in_memory::{InMemoryStorage, InMemoryStorageInternal},
    kv_storage::{GetResponse, KVStorage},
    namespaced_storage::NamespacedStorage,
    on_disk::{OnDiskStorage, OnDiskStorageInternal},
    policy::{Capability, Identity, Permission, Policy},
    storage::Storage,
    value::Value,
    vault::VaultStorage,
};

impl From<&SecureBackend> for Storage {
    fn from(backend: &SecureBackend) -> Self {
        match backend {
            SecureBackend::GitHub(config) => {
                let storage = GitHubStorage::new(
                    config.owner.clone(),
                    config.repository.clone(),
                    config.token.read_token().expect("Unable to read token"),
                );
                if let Some(namespace) = &config.namespace {
                    Storage::from(NamespacedStorage::new(Box::new(storage), namespace.clone()))
                } else {
                    Storage::from(storage)
                }
            }
            SecureBackend::InMemoryStorage => Storage::from(InMemoryStorage::new()),
            SecureBackend::OnDiskStorage(config) => {
                let storage = OnDiskStorage::new(config.path());
                if let Some(namespace) = &config.namespace {
                    Storage::from(NamespacedStorage::new(Box::new(storage), namespace.clone()))
                } else {
                    Storage::from(storage)
                }
            }
            SecureBackend::Vault(config) => Storage::from(VaultStorage::new(
                config.server.clone(),
                config.token.read_token().expect("Unable to read token"),
                config.namespace.clone(),
                config
                    .ca_certificate
                    .as_ref()
                    .map(|_| config.ca_certificate().unwrap()),
            )),
        }
    }
}

#[cfg(test)]
mod tests;
