// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_network_address::{NetworkAddress, RawNetworkAddress};
use move_core_types::move_resource::MoveResource;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

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
    /// This is an lcs serialized Vec<RawEncNetworkAddress>
    pub validator_network_addresses: Vec<u8>,
    /// This is an lcs serialized Vec<RawNetworkAddress>
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

    /// Returns a filtered list of correct NetworkAddresses within the fullnode_network_addresses
    pub fn fullnode_network_addresses<'a>(
        &self,
        err_callback: Option<Box<dyn Fn(String) + 'a>>,
    ) -> Result<Vec<NetworkAddress>, lcs::Error> {
        let addrs: Vec<RawNetworkAddress> = lcs::from_bytes(&self.fullnode_network_addresses)?;
        Ok(addrs
            .iter()
            .filter_map(|raw_addr| match NetworkAddress::try_from(raw_addr) {
                Ok(addr) => Some(addr),
                Err(err) => {
                    if let Some(cb) = &err_callback {
                        cb(err.to_string());
                    }
                    None
                }
            })
            .collect())
    }
}
