// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::move_resource::MoveResource;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519::X25519StaticPublicKey};
use parity_multiaddr::Multiaddr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorConfigResource {
    pub validator_config: ValidatorConfig,
}

impl MoveResource for ValidatorConfigResource {
    const MODULE_NAME: &'static str = "ValidatorConfig";
    const STRUCT_NAME: &'static str = "T";
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
