// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::{Author, Round},
    timeout::Timeout,
};
use anyhow::{ensure, Context};
use diem_crypto::ed25519::Ed25519Signature;
use diem_types::validator_verifier::ValidatorVerifier;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// TimeoutCertificate is a proof that 2f+1 participants in epoch i
/// have voted in round r and we can now move to round r+1.
/// It also contains the highest quorum cert round from 2f+1 participants as well as the highest quorum cert.
/// More details in safety rules.
/// Signatures are signed on TimeoutForSigning structure (see timeout.rs).
pub struct TimeoutCertificate {
    timeout: Timeout,
    signatures: BTreeMap<Author, (Round, Ed25519Signature)>,
}

impl fmt::Display for TimeoutCertificate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeoutCertificate[epoch: {}, round: {}, hqc_round: {}]",
            self.timeout.epoch(),
            self.timeout.round(),
            self.timeout.hqc_round(),
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

    /// Verifies the signatures for each validator, it signs on the timeout round and the highest qc round.
    pub fn verify(&self, validator: &ValidatorVerifier) -> anyhow::Result<()> {
        // Verify enough signatures
        validator.check_voting_power(self.signatures.keys())?;
        // Verify each node's timeout attestation.
        let mut highest_signed_round = 0;
        for (author, (hqc_round, signature)) in &self.signatures {
            let t = Timeout::new(self.timeout.epoch(), self.timeout.round(), *hqc_round);
            validator
                .verify(*author, &t, signature)
                .context("Failed to verify TimeoutCertificate")?;
            highest_signed_round = std::cmp::max(*hqc_round, highest_signed_round);
        }
        ensure!(
            highest_signed_round == self.timeout.hqc_round(),
            "Mismatched highest qc round"
        );
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

    /// The highest hqc round of the 2f+1 participants
    pub fn hqc_round(&self) -> Round {
        self.timeout.hqc_round()
    }

    /// Returns the signatures certifying the round
    pub fn signers(&self) -> impl Iterator<Item = &Author> {
        self.signatures.iter().map(|(k, _)| k)
    }

    /// Add a new timeout message from author
    pub fn add(&mut self, author: Author, timeout: Timeout, signature: Ed25519Signature) {
        assert_eq!(
            self.timeout.round(),
            timeout.round(),
            "Timeout should have the same round as TimeoutCert"
        );
        let hqc_round = timeout.hqc_round();
        if timeout.hqc_round() > self.timeout.hqc_round() {
            self.timeout = timeout;
        }
        self.signatures.insert(author, (hqc_round, signature));
    }
}
