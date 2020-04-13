// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use anyhow::{Error, Result};
use libra_crypto::{ed25519, x25519, ValidKey};
#[cfg(any(test, feature = "fuzzing"))]
use libra_crypto::{TPrivateKey, Uniform};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

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
    // This key can validate messages sent from this validator
    consensus_public_key: ed25519::VerifyingKey,
    // Voting power of this validator
    consensus_voting_power: u64,
    // This key can validate signed messages at the network layer
    network_signing_public_key: ed25519::VerifyingKey,
    // This key establishes the corresponding PrivateKey holder's eligibility to join the p2p
    // network
    network_identity_public_key: x25519::PublicKey,
}

impl fmt::Display for ValidatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "account_address: {}", self.account_address.short_str())
    }
}

impl ValidatorInfo {
    pub fn new(
        account_address: AccountAddress,
        consensus_public_key: ed25519::VerifyingKey,
        consensus_voting_power: u64,
        network_signing_public_key: ed25519::VerifyingKey,
        network_identity_public_key: x25519::PublicKey,
    ) -> Self {
        ValidatorInfo {
            account_address,
            consensus_public_key,
            consensus_voting_power,
            network_signing_public_key,
            network_identity_public_key,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_with_test_network_keys(
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        account_address: AccountAddress,
        consensus_public_key: ed25519::VerifyingKey,
        consensus_voting_power: u64,
    ) -> Self {
        let network_signing_public_key = ed25519::SigningKey::generate_for_testing().public_key();
        let private_key = x25519::PrivateKey::for_test(rng);
        let network_identity_public_key: x25519::PublicKey = private_key.public_key();

        Self {
            account_address,
            consensus_public_key,
            consensus_voting_power,
            network_signing_public_key,
            network_identity_public_key,
        }
    }

    /// Returns the id of this validator (hash of the current public key of the
    /// validator associated account address)
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    /// Returns the key for validating signed messages from this validator
    pub fn consensus_public_key(&self) -> &ed25519::VerifyingKey {
        &self.consensus_public_key
    }

    /// Returns the voting power for this validator
    pub fn consensus_voting_power(&self) -> u64 {
        self.consensus_voting_power
    }

    /// Returns the key for validating signed messages at the network layers
    pub fn network_signing_public_key(&self) -> &ed25519::VerifyingKey {
        &self.network_signing_public_key
    }

    /// Returns the key that establishes a validator's identity in the p2p network
    pub fn network_identity_public_key(&self) -> x25519::PublicKey {
        self.network_identity_public_key
    }
}

impl TryFrom<crate::proto::types::ValidatorInfo> for ValidatorInfo {
    type Error = Error;

    fn try_from(proto: crate::proto::types::ValidatorInfo) -> Result<Self> {
        let account_address = AccountAddress::try_from(proto.account_address)?;
        let consensus_public_key =
            ed25519::VerifyingKey::try_from(&proto.consensus_public_key[..])?;
        let consensus_voting_power = proto.consensus_voting_power;
        let network_signing_public_key =
            ed25519::VerifyingKey::try_from(&proto.network_signing_public_key[..])?;
        let network_identity_public_key =
            x25519::PublicKey::try_from(&proto.network_identity_public_key[..])?;
        Ok(Self::new(
            account_address,
            consensus_public_key,
            consensus_voting_power,
            network_signing_public_key,
            network_identity_public_key,
        ))
    }
}

impl From<ValidatorInfo> for crate::proto::types::ValidatorInfo {
    fn from(keys: ValidatorInfo) -> Self {
        Self {
            account_address: keys.account_address.to_vec(),
            consensus_public_key: keys.consensus_public_key.to_bytes().to_vec(),
            consensus_voting_power: keys.consensus_voting_power,
            network_signing_public_key: keys.network_signing_public_key.to_bytes().to_vec(),
            network_identity_public_key: keys.network_identity_public_key.to_bytes(),
        }
    }
}
