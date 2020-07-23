// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{Author, Round},
    timeout::Timeout,
};
use anyhow::Context;
use libra_crypto::ed25519::Ed25519Signature;
use libra_types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[cfg(any(test, feature = "fuzzing"))]
use proptest::prelude::*;

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// TimeoutCertificate is a proof that 2f+1 participants in epoch i
/// have voted in round r and we can now move to round r+1.
pub struct TimeoutCertificate {
    timeout: Timeout,
    signatures: BTreeMap<Author, Ed25519Signature>,
}

impl fmt::Display for TimeoutCertificate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeoutCertificate[epoch: {}, round: {}]",
            self.timeout.epoch(),
            self.timeout.round(),
        )
    }
}

impl TimeoutCertificate {
    /// Creates new TimeoutCertificate
    pub fn new(timeout: Timeout) -> Self {
        Self {
            timeout,
            signatures: BTreeMap::new(),
        }
    }

    /// Verifies the signatures for the round
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        validator
            .verify_aggregated_struct_signature(&self.timeout, &self.signatures)
            .context("Failed to verify TimeoutCertificate")?;
        Ok(())
    }

    /// Returns the epoch of the timeout certificate
    pub fn epoch(&self) -> u64 {
        self.timeout.epoch()
    }

    /// Returns the round of the timeout certificate
    pub fn round(&self) -> Round {
        self.timeout.round()
    }

    /// Returns the signatures certifying the round
    pub fn signatures(&self) -> &BTreeMap<Author, Ed25519Signature> {
        &self.signatures
    }

    pub fn add_signature(&mut self, author: Author, signature: Ed25519Signature) {
        self.signatures.entry(author).or_insert(signature);
    }

    pub fn remove_signature(&mut self, author: Author) {
        self.signatures.remove(&author);
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for TimeoutCertificate {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            any::<Timeout>(),
            prop::collection::vec(any::<Author>(), 0..10),
        )
            .prop_map(|(timeout, authors)| {
                let dummy_signature = Ed25519Signature::for_testing();
                let mut signatures = BTreeMap::new();
                for author in authors {
                    signatures.insert(author, dummy_signature.clone());
                }
                Self {
                    timeout,
                    signatures,
                }
            })
            .boxed()
    }
}
