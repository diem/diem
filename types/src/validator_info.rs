// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account_address::AccountAddress, validator_config::ValidatorConfig};
#[cfg(any(test, feature = "fuzzing"))]
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_crypto::{ed25519::Ed25519PublicKey, x25519};
#[cfg(any(test, feature = "fuzzing"))]
use libra_network_address::NetworkAddress;
#[cfg(any(test, feature = "fuzzing"))]
use libra_network_address::RawNetworkAddress;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(any(test, feature = "fuzzing"))]
use std::{convert::TryFrom, str::FromStr};

/// After executing a special transaction indicates a change to the next epoch, consensus
/// and networking get the new list of validators, their keys, and their voting power.  Consensus
/// has a public key to validate signed messages and networking will has public signing and identity
/// keys for creating secure channels of communication between validators.  The validators and
/// their public keys and voting power may or may not change between epochs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct ValidatorInfo {
    // The validator's account address. AccountAddresses are initially derived from the account
    // auth pubkey; however, the auth key can be rotated, so one should not rely on this
    // initial property.
    account_address: AccountAddress,
    // Voting power of this validator
    consensus_voting_power: u64,
    // Validator config
    config: ValidatorConfig,
}

impl fmt::Display for ValidatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "account_address: {}", self.account_address.short_str())
    }
}

impl ValidatorInfo {
    pub fn new(
        account_address: AccountAddress,
        consensus_voting_power: u64,
        config: ValidatorConfig,
    ) -> Self {
        ValidatorInfo {
            account_address,
            consensus_voting_power,
            config,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_with_test_network_keys(
        account_address: AccountAddress,
        consensus_public_key: Ed25519PublicKey,
        consensus_voting_power: u64,
    ) -> Self {
        let validator_network_signing_public_key =
            Ed25519PrivateKey::generate_for_testing().public_key();
        let private_key = x25519::PrivateKey::generate_for_testing();
        let validator_network_identity_public_key = private_key.public_key();
        let network_address = NetworkAddress::from_str("/ip4/127.0.0.1/tcp/1234").unwrap();
        let validator_network_address = RawNetworkAddress::try_from(&network_address).unwrap();

        let private_key = x25519::PrivateKey::generate_for_testing();
        let full_node_network_identity_public_key = private_key.public_key();
        let full_node_network_address = RawNetworkAddress::try_from(&network_address).unwrap();
        let config = ValidatorConfig::new(
            consensus_public_key,
            validator_network_signing_public_key,
            validator_network_identity_public_key,
            validator_network_address,
            full_node_network_identity_public_key,
            full_node_network_address,
        );

        Self {
            account_address,
            consensus_voting_power,
            config,
        }
    }

    /// Returns the id of this validator (hash of the current public key of the
    /// validator associated account address)
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    /// Returns the key for validating signed messages from this validator
    pub fn consensus_public_key(&self) -> &Ed25519PublicKey {
        &self.config.consensus_public_key
    }

    /// Returns the voting power for this validator
    pub fn consensus_voting_power(&self) -> u64 {
        self.consensus_voting_power
    }

    /// Returns the key for validating signed messages at the network layers
    pub fn network_signing_public_key(&self) -> &Ed25519PublicKey {
        &self.config.validator_network_signing_public_key
    }

    /// Returns the key that establishes a validator's identity in the p2p network
    pub fn network_identity_public_key(&self) -> x25519::PublicKey {
        self.config.validator_network_identity_public_key
    }

    /// Returns the validator's config
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }
}
