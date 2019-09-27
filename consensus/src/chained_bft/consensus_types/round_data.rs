// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::common::{Author, Round};
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer, SimpleSerializer};
use crypto::{
    hash::{CryptoHash, CryptoHasher, RoundDataHasher},
    HashValue,
};
use failure::{Result as ProtoResult, ResultExt};
use network::proto::RoundData as ProtoRoundData;
use proto_conv::{FromProto, IntoProto};
use serde::{Deserialize, Serialize};
use std::{
    convert::TryFrom,
    fmt::{Display, Formatter},
};
use types::crypto_proxies::{Signature, ValidatorSigner, ValidatorVerifier};

// Internal use only, contains the fields of RoundData that contribute to its hash.
struct RoundDataSerializer {
    round: Round,
}

impl CanonicalSerialize for RoundDataSerializer {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> failure::Result<()> {
        serializer.encode_u64(self.round)?;
        Ok(())
    }
}

impl CryptoHash for RoundDataSerializer {
    type Hasher = RoundDataHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            SimpleSerializer::<Vec<u8>>::serialize(self)
                .expect("Should serialize.")
                .as_ref(),
        );
        state.finish()
    }
}

/// RoundData keeps the information about the round a validator has voted in.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct RoundData {
    round: Round,
    author: Author,
    signature: Signature, // covers round and author
}

impl Display for RoundData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "RoundData: [round: {:02}]", self.round,)
    }
}

impl RoundData {
    pub fn new(round: Round, author: Author, validator_signer: &ValidatorSigner) -> Self {
        let signature = validator_signer
            .sign_message(Self::digest(round))
            .expect("Failed to sign LedgerInfo")
            .into();
        Self {
            round,
            author,
            signature,
        }
    }

    pub fn round(&self) -> Round {
        self.round
    }

    pub fn author(&self) -> Author {
        self.author
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn hash(&self) -> HashValue {
        Self::digest(self.round)
    }

    pub fn digest(round: Round) -> HashValue {
        RoundDataSerializer { round }.hash()
    }

    /// Verifies the signature on the round data.
    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        self.signature()
            .verify(validator, self.author(), self.hash())
            .with_context(|e| format!("Fail to verify RoundData: {:?}", e))?;
        Ok(())
    }
}

impl IntoProto for RoundData {
    type ProtoType = ProtoRoundData;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_round(self.round);
        proto.set_author(self.author.into());
        proto.set_signature(bytes::Bytes::from(self.signature.to_bytes()));
        proto
    }
}

impl FromProto for RoundData {
    type ProtoType = ProtoRoundData;

    fn from_proto(mut object: Self::ProtoType) -> ProtoResult<Self> {
        let round = object.get_round();
        let author = Author::try_from(object.take_author())?;
        let signature = Signature::try_from(object.get_signature())?;
        Ok(RoundData {
            round,
            author,
            signature,
        })
    }
}
