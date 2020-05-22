// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use libra_config::config::SecureBackend;
use std::convert::From;

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
    storage::{BoxStorage, Storage},
    value::Value,
    vault::VaultStorage,
};

impl From<&SecureBackend> for Box<dyn Storage> {
    fn from(backend: &SecureBackend) -> Self {
        match backend {
            SecureBackend::GitHub(config) => {
                let storage = GitHubStorage::new(
                    config.owner.clone(),
                    config.repository.clone(),
                    config.token.read_token().expect("Unable to read token"),
                );
                if let Some(namespace) = &config.namespace {
                    Box::new(NamespacedStorage::new(storage, namespace.clone()))
                } else {
                    Box::new(storage)
                }
            }
            SecureBackend::InMemoryStorage => Box::new(InMemoryStorage::new()),
            SecureBackend::OnDiskStorage(config) => {
                let storage = OnDiskStorage::new(config.path());
                if let Some(namespace) = &config.namespace {
                    Box::new(NamespacedStorage::new(storage, namespace.clone()))
                } else {
                    Box::new(storage)
                }
            }
            SecureBackend::Vault(config) => Box::new(VaultStorage::new(
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
