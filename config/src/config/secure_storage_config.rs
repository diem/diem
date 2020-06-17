// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::config::{Identity, NetworkConfig, SecureBackend, WaypointConfig};
use libra_secure_storage::{
    GitHubStorage, InMemoryStorage, KVStorage, NamespacedStorage, OnDiskStorage, Storage,
    VaultStorage,
};
use libra_types::{waypoint::Waypoint, PeerId};
use std::{convert::TryInto, str::FromStr};

pub fn peer_id(config: &NetworkConfig) -> PeerId {
    let key = match &config.identity {
        Identity::FromConfig(config) => Some(config.peer_id),
        Identity::FromStorage(config) => {
            let storage: Storage = (&config.backend).into();
            let peer_id = storage
                .get(&config.peer_id_name)
                .expect("Unable to read peer id")
                .value
                .string()
                .expect("Expected string for peer id");
            Some(peer_id.try_into().expect("Unable to parse peer id"))
        }
        Identity::None => None,
    };
    key.expect("peer id should be present")
}

pub fn waypoint(config: &WaypointConfig) -> Waypoint {
    let waypoint = match &config {
        WaypointConfig::FromConfig(waypoint) => Some(*waypoint),
        WaypointConfig::FromStorage(backend) => {
            let storage: Storage = backend.into();
            let waypoint = storage
                .get(libra_global_constants::WAYPOINT)
                .expect("Unable to read waypoint")
                .value
                .string()
                .expect("Expected string for waypoint");
            Some(Waypoint::from_str(&waypoint).expect("Unable to parse waypoint"))
        }
        WaypointConfig::None => None,
    };
    waypoint.expect("waypoint should be present")
}

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
