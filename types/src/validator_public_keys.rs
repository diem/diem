// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crypto::{ed25519::*, traits::ValidKey, x25519::X25519StaticPublicKey};
use failure::Result;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};

/// After executing a special transaction indicates a change to the next epoch, consensus
/// and networking get the new list of validators, their keys, and their voting power.  Consensus
/// has a public key to validate signed messages and networking will has public signing and identity
/// keys for creating secure channels of communication between validators.  The validators and
/// their public keys and voting power may or may not change between epochs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct ValidatorPublicKeys {
    // Hash value of the current public key of the account address
    account_address: AccountAddress,
    // This key can validate messages sent from this validator
    consensus_public_key: Ed25519PublicKey,
    // Voting power of this validator
    consensus_voting_power: u64,
    // This key can validate signed messages at the network layer
    network_signing_public_key: Ed25519PublicKey,
    // This key establishes the corresponding PrivateKey holder's eligibility to join the p2p
    // network
    network_identity_public_key: X25519StaticPublicKey,
}

impl fmt::Display for ValidatorPublicKeys {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "account_address: {}", self.account_address.short_str())
    }
}

impl ValidatorPublicKeys {
    pub fn new(
        account_address: AccountAddress,
        consensus_public_key: Ed25519PublicKey,
        consensus_voting_power: u64,
        network_signing_public_key: Ed25519PublicKey,
        network_identity_public_key: X25519StaticPublicKey,
    ) -> Self {
        ValidatorPublicKeys {
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
    pub fn consensus_public_key(&self) -> &Ed25519PublicKey {
        &self.consensus_public_key
    }

    /// Returns the key for validating signed messages at the network layers
    pub fn network_signing_public_key(&self) -> &Ed25519PublicKey {
        &self.network_signing_public_key
    }

    /// Returns the key that establishes a validator's identity in the p2p network
    pub fn network_identity_public_key(&self) -> &X25519StaticPublicKey {
        &self.network_identity_public_key
    }
}

impl TryFrom<crate::proto::types::ValidatorPublicKeys> for ValidatorPublicKeys {
    type Error = failure::Error;

    fn try_from(proto: crate::proto::types::ValidatorPublicKeys) -> Result<Self> {
        let account_address = AccountAddress::try_from(proto.account_address)?;
        let consensus_public_key = Ed25519PublicKey::try_from(&proto.consensus_public_key[..])?;
        let consensus_voting_power = proto.consensus_voting_power;
        let network_signing_public_key =
            Ed25519PublicKey::try_from(&proto.network_signing_public_key[..])?;
        let network_identity_public_key =
            X25519StaticPublicKey::try_from(&proto.network_identity_public_key[..])?;
        Ok(Self::new(
            account_address,
            consensus_public_key,
            consensus_voting_power,
            network_signing_public_key,
            network_identity_public_key,
        ))
    }
}

impl From<ValidatorPublicKeys> for crate::proto::types::ValidatorPublicKeys {
    fn from(keys: ValidatorPublicKeys) -> Self {
        Self {
            account_address: keys.account_address.to_vec(),
            consensus_public_key: keys.consensus_public_key.to_bytes().to_vec(),
            consensus_voting_power: keys.consensus_voting_power,
            network_signing_public_key: keys.network_signing_public_key.to_bytes().to_vec(),
            network_identity_public_key: keys.network_identity_public_key.to_bytes().to_vec(),
        }
    }
}
