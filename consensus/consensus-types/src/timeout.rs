// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::Round;
use diem_crypto::ed25519::Ed25519Signature;
use diem_crypto_derive::{BCSCryptoHash, CryptoHasher};
use diem_types::validator_signer::ValidatorSigner;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// This structure contains all the information necessary to construct a signature
/// on the equivalent of a timeout message
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, CryptoHasher, BCSCryptoHash)]
pub struct Timeout {
    /// Epoch number corresponds to the set of validators that are active for this round.
    epoch: u64,
    /// The consensus protocol executes proposals (blocks) in rounds, which monotically increase per epoch.
    round: Round,
}

impl Timeout {
    pub fn new(epoch: u64, round: Round) -> Self {
        Self { epoch, round }
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn sign(&self, signer: &ValidatorSigner) -> Ed25519Signature {
        signer.sign(self)
    }
}

impl Display for Timeout {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Timeout: [epoch: {}, round: {}]", self.epoch, self.round,)
    }
}
