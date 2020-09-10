// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_network_address::{encrypted::EncNetworkAddress, NetworkAddress};
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
    /// This is an lcs serialized Vec<EncNetworkAddress>
    pub validator_network_addresses: Vec<u8>,
    /// This is an lcs serialized Vec<NetworkAddress>
    pub fullnode_network_addresses: Vec<u8>,
}

impl ValidatorConfig {
    pub fn new(
        consensus_public_key: Ed25519PublicKey,
        validator_network_addresses: Vec<u8>,
        fullnode_network_addresses: Vec<u8>,
    ) -> Self {
        ValidatorConfig {
            consensus_public_key,
            validator_network_addresses,
            fullnode_network_addresses,
        }
    }

    pub fn fullnode_network_addresses(&self) -> Result<Vec<NetworkAddress>, lcs::Error> {
        lcs::from_bytes(&self.fullnode_network_addresses)
    }

    pub fn validator_network_addresses(&self) -> Result<Vec<EncNetworkAddress>, lcs::Error> {
        lcs::from_bytes(&self.validator_network_addresses)
    }
}
