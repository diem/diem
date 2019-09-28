// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::common::{self, Author, Round};
use failure::prelude::*;
use network;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use types::{
    account_address::AccountAddress,
    crypto_proxies::{Signature, ValidatorVerifier},
};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// TimeoutCertificate is a proof that 2f+1 participants have voted in round r and we can now move
/// to round r+1.
pub struct TimeoutCertificate {
    round: Round,
    signatures: HashMap<Author, Signature>,
}

impl fmt::Display for TimeoutCertificate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimeoutCertificate[round: {}]", self.round)
    }
}

#[allow(dead_code)]
impl TimeoutCertificate {
    /// Creates new TimeoutCertificate
    pub fn new(round: Round, signatures: HashMap<Author, Signature>) -> Self {
        Self { round, signatures }
    }

    /// Verifies the signatures for the round
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        validator.check_voting_power(self.signatures().keys())?;
        let round_digest = common::round_hash(self.round());
        for (author, signature) in self.signatures() {
            signature
                .verify(validator, *author, round_digest)
                .with_context(|e| format!("Fail to verify TimeoutCertificate: {:?}", e))?;
        }
        Ok(())
    }

    /// Returns the round of the timeout certificate
    pub fn round(&self) -> Round {
        self.round
    }

    /// Returns the signatures certifying the round
    pub fn signatures(&self) -> &HashMap<Author, Signature> {
        &self.signatures
    }
}

impl IntoProto for TimeoutCertificate {
    type ProtoType = network::proto::TimeoutCertificate;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_round(self.round);
        self.signatures.into_iter().for_each(|(author, signature)| {
            let mut validator_signature = types::proto::ledger_info::ValidatorSignature::new();
            validator_signature.set_validator_id(author.into_proto());
            validator_signature.set_signature(signature.to_bytes().to_vec());
            proto.mut_signatures().push(validator_signature)
        });
        proto
    }
}

impl FromProto for TimeoutCertificate {
    type ProtoType = network::proto::TimeoutCertificate;

    fn from_proto(mut object: Self::ProtoType) -> failure::Result<Self> {
        let round = object.get_round();
        let signatures_proto = object.take_signatures();
        let signatures = signatures_proto
            .into_iter()
            .map(|proto| {
                let author = AccountAddress::from_proto(proto.get_validator_id().to_vec())?;
                let signature = Signature::try_from(proto.get_signature())?;
                Ok((author, signature))
            })
            .collect::<Result<HashMap<_, _>>>()?;
        Ok(TimeoutCertificate::new(round, signatures))
    }
}
