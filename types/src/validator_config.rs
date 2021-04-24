// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    network_address::{encrypted::EncNetworkAddress, NetworkAddress},
};
use diem_crypto::ed25519::Ed25519PublicKey;
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    move_resource::{MoveResource, MoveStructType},
};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Default)]
pub struct ValidatorConfigResource {
    pub validator_config: Option<ValidatorConfig>,
    pub delegated_account: Option<AccountAddress>,
    pub human_name: Vec<u8>,
}

impl MoveStructType for ValidatorConfigResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ValidatorConfig");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ValidatorConfig");
}

impl MoveResource for ValidatorConfigResource {}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Default)]
pub struct ValidatorOperatorConfigResource {
    pub human_name: Vec<u8>,
}

impl MoveStructType for ValidatorOperatorConfigResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("ValidatorOperatorConfig");
    const STRUCT_NAME: &'static IdentStr = ident_str!("ValidatorOperatorConfig");
}

impl MoveResource for ValidatorOperatorConfigResource {}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorConfig {
    pub consensus_public_key: Ed25519PublicKey,
    /// This is an bcs serialized Vec<EncNetworkAddress>
    pub validator_network_addresses: Vec<u8>,
    /// This is an bcs serialized Vec<NetworkAddress>
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

    pub fn fullnode_network_addresses(&self) -> Result<Vec<NetworkAddress>, bcs::Error> {
        bcs::from_bytes(&self.fullnode_network_addresses)
    }

    pub fn validator_network_addresses(&self) -> Result<Vec<EncNetworkAddress>, bcs::Error> {
        bcs::from_bytes(&self.validator_network_addresses)
    }
}
