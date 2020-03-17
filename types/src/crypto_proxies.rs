// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines a data structure made to contain a cryptographic
//! signature, in the sense of an implementation of
//! libra_crypto::traits::Signature. The container is an opaque NewType that
//! intentionally does not allow access to the inner impl.
//!
//! It also proxies the four methods used in consensus on structures that
//! expect to receive one such implementation directly.
//!
//! The goal of this structure is two-fold:
//! - help make sure that any consensus data that uses cryptographic material is defined generically
//!   in terms of libra_crypto::traits, rather than accessing the implementation details of a particular
//!   scheme (a.k.a. encapsulation),
//! - contribute to a single-location place for instantiations of these polymorphic types
//!   (`crate::chained_bft::consensus_types`)
//! - help the consensus crate avoid generic parameters within the code, by avoiding propagating
//!   generic signature type parameters throughout the crate's code (as prototyped in https://github.com/libra/libra/pull/522),
//!   as generic types are seen as hindering readability.

use crate::{
    validator_info::ValidatorInfo as RawValidatorInfo,
    validator_set::ValidatorSet as RawValidatorSet,
    validator_verifier::ValidatorVerifier as RawValidatorVerifier,
};

// This sets the types containing cryptographic materials used in the
// consensus crate. It is intended as a one-stop shop for changing the
// signing scheme of the consensus. If consensus uses a type that
// contains cryptographic material, its polymorphic parameter should be
// instantiated here and re-exported.
//
// Imports specific to any signing scheme (e.g. ed25519::*) should not be
// present anywhere in the consensus crate. Use of raw cryptographic
// types that do not go through the instantiated polymorphic structures
// below is banned.

use libra_crypto::ed25519::Ed25519PublicKey;
use std::{collections::BTreeMap, fmt};

pub type ValidatorInfo = RawValidatorInfo<Ed25519PublicKey>;
pub type ValidatorSet = RawValidatorSet<Ed25519PublicKey>;
pub type ValidatorVerifier = RawValidatorVerifier<Ed25519PublicKey>;
pub use crate::validator_change::ValidatorChangeProof;
use std::sync::Arc;

#[derive(Clone, Debug)]
/// EpochInfo represents a trusted validator set to validate messages from the specific epoch,
/// it could be updated with ValidatorChangeProof.
pub struct EpochInfo {
    pub epoch: u64,
    pub verifier: Arc<ValidatorVerifier>,
}

impl EpochInfo {
    pub fn empty() -> Self {
        Self {
            epoch: 0,
            verifier: Arc::new(ValidatorVerifier::new(BTreeMap::new())),
        }
    }
}

impl fmt::Display for EpochInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EpochInfo [epoch: {}, validator: {}]",
            self.epoch, self.verifier
        )
    }
}

#[cfg(any(test, feature = "fuzzing"))]
use crate::validator_signer::ValidatorSigner;

/// Helper function to get random validator signers and a corresponding validator verifier for
/// testing.  If custom_voting_power_quorum is not None, set a custom voting power quorum amount.
/// With pseudo_random_account_address enabled, logs show 0 -> [0000], 1 -> [1000]
#[cfg(any(test, feature = "fuzzing"))]
pub fn random_validator_verifier(
    count: usize,
    custom_voting_power_quorum: Option<u64>,
    pseudo_random_account_address: bool,
) -> (Vec<ValidatorSigner>, ValidatorVerifier) {
    let mut signers = Vec::new();
    let mut account_address_to_validator_info = BTreeMap::new();
    for i in 0..count {
        let random_signer = if pseudo_random_account_address {
            ValidatorSigner::from_int(i as u8)
        } else {
            ValidatorSigner::random([i as u8; 32])
        };
        account_address_to_validator_info.insert(
            random_signer.author(),
            crate::validator_verifier::ValidatorConsensusInfo::new(random_signer.public_key(), 1),
        );
        signers.push(random_signer);
    }
    (
        signers,
        match custom_voting_power_quorum {
            Some(custom_voting_power_quorum) => ValidatorVerifier::new_with_quorum_voting_power(
                account_address_to_validator_info,
                custom_voting_power_quorum,
            )
            .expect("Unable to create testing validator verifier"),
            None => ValidatorVerifier::new(account_address_to_validator_info),
        },
    )
}
