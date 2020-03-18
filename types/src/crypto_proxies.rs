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
    validator_set::ValidatorSet as RawValidatorSet, validator_verifier::ValidatorVerifier,
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
