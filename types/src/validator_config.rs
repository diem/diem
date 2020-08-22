// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_network_address::{encrypted::RawEncNetworkAddress, RawNetworkAddress};
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Default)]
pub struct ValidatorConfigResource {
    pub validator_config: Option<ValidatorConfig>,
    pub delegated_account: Option<AccountAddress>,
    pub human_name: Vec<u8>,
}

impl MoveResource for ValidatorConfigResource {
    const MODULE_NAME: &'static str = "ValidatorConfig";
    const STRUCT_NAME: &'static str = "ValidatorConfig";
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Default)]
pub struct ValidatorOperatorConfigResource {
    pub human_name: Vec<u8>,
}

impl MoveResource for ValidatorOperatorConfigResource {
    const MODULE_NAME: &'static str = "ValidatorOperatorConfig";
    const STRUCT_NAME: &'static str = "ValidatorOperatorConfig";
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorConfig {
    pub consensus_public_key: Ed25519PublicKey,
    pub validator_network_address: RawEncNetworkAddress,
    pub full_node_network_address: RawNetworkAddress,
}

impl ValidatorConfig {
    pub fn new(
        consensus_public_key: Ed25519PublicKey,
        validator_network_address: RawEncNetworkAddress,
        full_node_network_address: RawNetworkAddress,
    ) -> Self {
        ValidatorConfig {
            consensus_public_key,
            validator_network_address,
            full_node_network_address,
        }
    }
}
