// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_network_address::{encrypted::RawEncNetworkAddress, RawNetworkAddress};
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ValidatorConfigResource {
    pub validator_config: Option<ValidatorConfig>,
    pub delegated_account: Option<AccountAddress>,
}

impl MoveResource for ValidatorConfigResource {
    const MODULE_NAME: &'static str = "ValidatorConfig";
    const STRUCT_NAME: &'static str = "ValidatorConfig";
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorConfig {
    pub consensus_public_key: Ed25519PublicKey,
    pub validator_network_addresses: Vec<RawEncNetworkAddress>,
    pub full_node_network_addresses: Vec<RawNetworkAddress>,
}

impl ValidatorConfig {
    pub fn new(
        consensus_public_key: Ed25519PublicKey,
        validator_network_addresses: Vec<RawEncNetworkAddress>,
        full_node_network_addresses: Vec<RawNetworkAddress>,
    ) -> Self {
        ValidatorConfig {
            consensus_public_key,
            validator_network_addresses,
            full_node_network_addresses,
        }
    }
}
