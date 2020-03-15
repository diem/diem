// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::{AccessPath, Accesses},
    account_config,
    language_storage::StructTag,
};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519::X25519StaticPublicKey};
use move_core_types::identifier::{IdentStr, Identifier};
use once_cell::sync::Lazy;
use parity_multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

static VALIDATOR_CONFIG_MODULE_NAME: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("ValidatorConfig").unwrap());
static VALIDATOR_CONFIG_STRUCT_NAME: Lazy<Identifier> = Lazy::new(|| Identifier::new("T").unwrap());

pub fn validator_config_module_name() -> &'static IdentStr {
    &*VALIDATOR_CONFIG_MODULE_NAME
}

pub fn validator_config_struct_name() -> &'static IdentStr {
    &*VALIDATOR_CONFIG_STRUCT_NAME
}

pub fn validator_config_tag() -> StructTag {
    StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        name: validator_config_struct_name().to_owned(),
        module: validator_config_module_name().to_owned(),
        type_params: vec![],
    }
}

/// The access path where the Validator Set resource is stored.
pub static VALIDATOR_CONFIG_RESOURCE_PATH: Lazy<Vec<u8>> =
    Lazy::new(|| AccessPath::resource_access_vec(&validator_config_tag(), &Accesses::empty()));

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorConfigResource {
    pub validator_config: ValidatorConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ValidatorConfig {
    pub consensus_pubkey: Ed25519PublicKey,
    pub validator_network_signing_pubkey: Ed25519PublicKey,
    pub validator_network_identity_pubkey: X25519StaticPublicKey,
    pub validator_network_address: Multiaddr,
    pub fullnodes_network_identity_pubkey: X25519StaticPublicKey,
    pub fullnodes_network_address: Multiaddr,
}
