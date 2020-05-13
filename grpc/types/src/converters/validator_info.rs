// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Error, Result};
use libra_crypto::{ed25519::Ed25519PublicKey, traits::ValidCryptoMaterial, x25519};
use libra_network_address::RawNetworkAddress;
use libra_types::{
    account_address::AccountAddress, validator_config::ValidatorConfig,
    validator_info::ValidatorInfo,
};
use std::convert::TryFrom;

impl TryFrom<crate::proto::types::ValidatorInfo> for ValidatorInfo {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorInfo) -> Result<Self> {
        let account_address = AccountAddress::try_from(proto.account_address)?;
        let consensus_public_key = Ed25519PublicKey::try_from(&proto.consensus_public_key[..])?;
        let consensus_voting_power = proto.consensus_voting_power;

        let validator_network_signing_public_key =
            Ed25519PublicKey::try_from(&proto.validator_network_signing_public_key[..])?;
        let validator_network_identity_public_key =
            x25519::PublicKey::try_from(&proto.validator_network_identity_public_key[..])?;
        let validator_network_address = RawNetworkAddress::new(proto.validator_network_address);
        let full_node_network_identity_public_key =
            x25519::PublicKey::try_from(&proto.full_node_network_identity_public_key[..])?;
        let full_node_network_address = RawNetworkAddress::new(proto.full_node_network_address);

        let config = ValidatorConfig::new(
            consensus_public_key,
            validator_network_signing_public_key,
            validator_network_identity_public_key,
            validator_network_address,
            full_node_network_identity_public_key,
            full_node_network_address,
        );
        Ok(Self::new(account_address, consensus_voting_power, config))
    }
}

impl From<ValidatorInfo> for crate::proto::types::ValidatorInfo {
    fn from(keys: ValidatorInfo) -> Self {
        let config = &keys.config();
        Self {
            account_address: keys.account_address().to_vec(),
            consensus_voting_power: keys.consensus_voting_power(),
            consensus_public_key: keys.consensus_public_key().to_bytes().to_vec(),
            validator_network_signing_public_key: config
                .validator_network_signing_public_key
                .to_bytes()
                .to_vec(),
            validator_network_identity_public_key: config
                .validator_network_identity_public_key
                .to_bytes(),
            validator_network_address: config.validator_network_address.as_ref().to_vec(),
            full_node_network_identity_public_key: config
                .full_node_network_identity_public_key
                .to_bytes(),
            full_node_network_address: config.full_node_network_address.as_ref().to_vec(),
        }
    }
}
