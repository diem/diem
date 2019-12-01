// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{Author, Round},
    timeout::Timeout,
};
use anyhow::Context;
use libra_crypto::hash::CryptoHash;
use libra_types::crypto_proxies::{Signature, ValidatorVerifier};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// TimeoutCertificate is a proof that 2f+1 participants in epoch i
/// have voted in round r and we can now move to round r+1.
pub struct TimeoutCertificate {
    timeout: Timeout,
    signatures: HashMap<Author, Signature>,
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
    pub fn new(timeout: Timeout, signatures: HashMap<Author, Signature>) -> Self {
        Self {
            timeout,
            signatures,
        }
    }

    /// Verifies the signatures for the round
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        validator.check_voting_power(self.signatures().keys())?;
        let timeout_hash = self.timeout.hash();
        for (author, signature) in self.signatures() {
            signature
                .verify(validator, *author, timeout_hash)
                .context("Fail to verify TimeoutCertificate")?;
        }
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
    pub fn signatures(&self) -> &HashMap<Author, Signature> {
        &self.signatures
    }

    pub fn add_signature(&mut self, author: Author, signature: Signature) {
        self.signatures.entry(author).or_insert(signature);
    }

    pub fn remove_signature(&mut self, author: Author) {
        self.signatures.remove(&author);
    }
}
