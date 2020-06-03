// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Storage;
use libra_config::config::{Identity, NetworkConfig};
use libra_crypto::x25519;
use libra_types::PeerId;
use std::convert::TryInto;

pub fn identity_key(config: &mut NetworkConfig) -> x25519::PrivateKey {
    let key = match &mut config.identity {
        Identity::FromConfig(config) => config.keypair.take_private(),
        Identity::FromStorage(config) => {
            let storage: Box<dyn Storage> = (&config.backend).into();
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
            let storage: Box<dyn Storage> = (&config.backend).into();
            println!(
                "{} {:?}",
                config.peer_id_name,
                storage.get(&config.peer_id_name)
            );
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
