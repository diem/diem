// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, move_resource::MoveResource};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
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
    // TODO(valerini): reflect the same move structure here
    pub consensus_pubkey: Ed25519PublicKey,
    pub network_signing_pubkey: Ed25519PublicKey,
    pub network_identity_pubkey: x25519::PublicKey,
    pub validator_network_identity_pubkey: x25519::PublicKey,
    pub validator_network_address: Multiaddr,
    pub fullnodes_network_identity_pubkey: x25519::PublicKey,
    pub fullnodes_network_address: Multiaddr,
    pub delegated_account: Vec<AccountAddress>,
}
