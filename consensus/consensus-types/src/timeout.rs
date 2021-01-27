// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::Round;
use crate::quorum_cert::QuorumCert;
#[cfg(any(test, feature = "fuzzing"))]
use diem_crypto::ed25519::Ed25519Signature;
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
#[cfg(any(test, feature = "fuzzing"))]
use diem_types::validator_signer::ValidatorSigner;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// This structure contains all the information necessary to construct a signature
/// on the equivalent of a timeout message
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Timeout {
    /// Epoch number corresponds to the set of validators that are active for this round.
    epoch: u64,
    /// The consensus protocol executes proposals (blocks) in rounds, which monotically increase per epoch.
    round: Round,
    /// Highest quorum cert,
    quorum_cert: QuorumCert,
}

/// Internal struct for signing, only use the round of the qc instead of the full qc.
#[derive(Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub struct TimeoutForSigning {
    pub epoch: u64,
    pub round: Round,
    pub hqc_round: Round,
}

impl TimeoutForSigning {
    pub fn new(epoch: u64, round: Round, hqc_round: Round) -> Self {
        Self {
            epoch,
            round,
            hqc_round,
        }
    }
}

impl Timeout {
    pub fn new(epoch: u64, round: Round, quorum_cert: QuorumCert) -> Self {
        Self {
            epoch,
            round,
            quorum_cert,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn round(&self) -> Round {
        self.round
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn sign(&self, signer: &ValidatorSigner) -> Ed25519Signature {
        signer.sign(&self.signed_repr())
    }

    pub fn signed_repr(&self) -> TimeoutForSigning {
        TimeoutForSigning {
            epoch: self.epoch,
            round: self.round,
            hqc_round: self.quorum_cert.certified_block().round(),
        }
    }

    pub fn hqc_round(&self) -> Round {
        self.quorum_cert.certified_block().round()
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        &self.quorum_cert
    }
}

impl Display for Timeout {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "Timeout: [epoch: {}, round: {}, hqc_round: {}]",
            self.epoch,
            self.round,
            self.quorum_cert.certified_block().round()
        )
    }
}
