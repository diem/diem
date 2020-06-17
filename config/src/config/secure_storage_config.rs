// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::SecureBackend;
use libra_secure_storage::{
    GitHubStorage, InMemoryStorage, NamespacedStorage, OnDiskStorage, Storage, VaultStorage,
};

impl From<&SecureBackend> for Storage {
    fn from(backend: &SecureBackend) -> Self {
        match backend {
            SecureBackend::GitHub(config) => {
                let storage = GitHubStorage::new(
                    config.repository_owner.clone(),
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
