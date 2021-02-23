// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(any(test, feature = "fuzzing"))]
use crate::network_address::{
    encrypted::{TEST_SHARED_VAL_NETADDR_KEY, TEST_SHARED_VAL_NETADDR_KEY_VERSION},
    NetworkAddress,
};
use crate::{account_address::AccountAddress, validator_config::ValidatorConfig};
use diem_crypto::ed25519::Ed25519PublicKey;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::fmt;

/// After executing a special transaction indicates a change to the next epoch, consensus
/// and networking get the new list of validators, their keys, and their voting power.  Consensus
/// has a public key to validate signed messages and networking will has public identity
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
    // The time of last recofiguration invoked by this validator
    // in microseconds
    last_config_update_time: u64,
}

impl fmt::Display for ValidatorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "account_address: {}",
            self.account_address.short_str_lossless()
        )
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
            last_config_update_time: 0,
        }
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_with_test_network_keys(
        account_address: AccountAddress,
        consensus_public_key: Ed25519PublicKey,
        consensus_voting_power: u64,
    ) -> Self {
        let addr = NetworkAddress::mock();
        let enc_addr = addr.clone().encrypt(
            &TEST_SHARED_VAL_NETADDR_KEY,
            TEST_SHARED_VAL_NETADDR_KEY_VERSION,
            &account_address,
            0,
            0,
        );
        let config = ValidatorConfig::new(
            consensus_public_key,
            bcs::to_bytes(&vec![enc_addr.unwrap()]).unwrap(),
            bcs::to_bytes(&vec![addr]).unwrap(),
        );

        Self {
            account_address,
            consensus_voting_power,
            config,
            last_config_update_time: 0,
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

    /// Returns the validator's config
    pub fn config(&self) -> &ValidatorConfig {
        &self.config
    }

    /// Returns the validator's config, consuming self
    pub fn into_config(self) -> ValidatorConfig {
        self.config
    }
}
