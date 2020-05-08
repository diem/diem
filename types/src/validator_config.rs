// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
use libra_network_address::RawNetworkAddress;
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorConfigResource {
    pub validator_config: ValidatorConfig,
    pub delegated_account: Vec<AccountAddress>,
}

impl MoveResource for ValidatorConfigResource {
    const MODULE_NAME: &'static str = "ValidatorConfig";
    const STRUCT_NAME: &'static str = "T";
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorConfig {
    pub consensus_public_key: Ed25519PublicKey,
    pub validator_network_signing_public_key: Ed25519PublicKey,
    pub validator_network_identity_public_key: x25519::PublicKey,
    pub validator_network_address: RawNetworkAddress,
    pub full_node_network_identity_public_key: x25519::PublicKey,
    pub full_node_network_address: RawNetworkAddress,
}

impl ValidatorConfig {
    pub fn new(
        consensus_public_key: Ed25519PublicKey,
        validator_network_signing_public_key: Ed25519PublicKey,
        validator_network_identity_public_key: x25519::PublicKey,
        validator_network_address: RawNetworkAddress,
        full_node_network_identity_public_key: x25519::PublicKey,
        full_node_network_address: RawNetworkAddress,
    ) -> Self {
        ValidatorConfig {
            consensus_public_key,
            validator_network_signing_public_key,
            validator_network_identity_public_key,
            validator_network_address,
            full_node_network_identity_public_key,
            full_node_network_address,
        }
    }
}
