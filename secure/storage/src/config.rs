// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{CryptoStorage, KVStorage, Storage};
use libra_config::config::{Identity, NetworkConfig, WaypointConfig};
use libra_crypto::x25519;
use libra_types::{waypoint::Waypoint, PeerId};
use std::{convert::TryInto, str::FromStr};

pub fn identity_key(config: &mut NetworkConfig) -> x25519::PrivateKey {
    let key = match &mut config.identity {
        Identity::FromConfig(config) => config.keypair.take_private(),
        Identity::FromStorage(config) => {
            let storage: Storage = (&config.backend).into();
            let key = storage
                .export_private_key(&config.key_name)
                .expect("Unable to read key");
            let key = x25519::PrivateKey::from_ed25519_private_bytes(&key.to_bytes())
                .expect("Unable to convert key");
            Some(key)
        }
        Identity::None => None,
    };
    key.expect("identity key should be present")
}

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
        WaypointConfig::FromConfig { waypoint } => Some(*waypoint),
        WaypointConfig::FromStorage { backend } => {
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
