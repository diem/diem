// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::{self, Author, Round};
use failure::prelude::*;
use libra_types::{
    account_address::AccountAddress,
    crypto_proxies::{Signature, ValidatorVerifier},
};
use network;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// TimeoutCertificate is a proof that 2f+1 participants in epoch i
/// have voted in round r and we can now move to round r+1.
pub struct TimeoutCertificate {
    epoch: u64,
    round: Round,
    signatures: HashMap<Author, Signature>,
}

impl fmt::Display for TimeoutCertificate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "TimeoutCertificate[epoch: {}, round: {}]",
            self.epoch, self.round
        )
    }
}

impl TimeoutCertificate {
    /// Creates new TimeoutCertificate
    pub fn new(epoch: u64, round: Round, signatures: HashMap<Author, Signature>) -> Self {
        Self {
            epoch,
            round,
            signatures,
        }
    }

    /// Verifies the signatures for the round
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        validator.check_voting_power(self.signatures().keys())?;
        let round_digest = common::timeout_hash(self.round(), self.epoch());
        for (author, signature) in self.signatures() {
            signature
                .verify(validator, *author, round_digest)
                .with_context(|e| format!("Fail to verify TimeoutCertificate: {:?}", e))?;
        }
        Ok(())
    }

    /// Returns the epoch of the timeout certificate
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns the round of the timeout certificate
    pub fn round(&self) -> Round {
        self.round
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

impl TryFrom<network::proto::TimeoutCertificate> for TimeoutCertificate {
    type Error = failure::Error;

    fn try_from(proto: network::proto::TimeoutCertificate) -> failure::Result<Self> {
        let epoch = proto.epoch;
        let round = proto.round;
        let signatures = proto
            .signatures
            .into_iter()
            .map(|proto| {
                let author = AccountAddress::try_from(proto.validator_id)?;
                let signature = Signature::try_from(&proto.signature)?;
                Ok((author, signature))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(TimeoutCertificate::new(epoch, round, signatures))
    }
}

impl From<TimeoutCertificate> for network::proto::TimeoutCertificate {
    fn from(cert: TimeoutCertificate) -> Self {
        let signatures = cert
            .signatures
            .into_iter()
            .map(
                |(validator_id, signature)| libra_types::proto::types::ValidatorSignature {
                    validator_id: validator_id.to_vec(),
                    signature: signature.to_bytes().to_vec(),
                },
            )
            .collect();

        Self {
            epoch: cert.epoch,
            round: cert.round,
            signatures,
        }
    }
}
