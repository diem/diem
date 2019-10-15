// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{common::Round, vote_data::VoteData};
use canonical_serialization::{CanonicalSerialize, CanonicalSerializer};
use crypto::{
    hash::{CryptoHash, ACCUMULATOR_PLACEHOLDER_HASH, GENESIS_BLOCK_ID},
    HashValue,
};
use failure::prelude::*;
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct QuorumCert {
    /// The vote information certified by the quorum.
    vote_data: VoteData,
    /// The signed LedgerInfo of a committed block that carries the data about the certified block.
    signed_ledger_info: LedgerInfoWithSignatures,
}

impl Display for QuorumCert {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "QuorumCert: [{}, {}]",
            self.vote_data, self.signed_ledger_info
        )
    }
}

impl CanonicalSerialize for QuorumCert {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.vote_data)?
            .encode_struct(&self.signed_ledger_info)?;
        Ok(())
    }
}

impl QuorumCert {
    pub fn new(vote_data: VoteData, signed_ledger_info: LedgerInfoWithSignatures) -> Self {
        QuorumCert {
            vote_data,
            signed_ledger_info,
        }
    }
    /// All the vote data getters are just proxies for retrieving the values from the VoteData
    pub fn certified_block_id(&self) -> HashValue {
        self.vote_data.block_id()
    }

    pub fn certified_state_id(&self) -> HashValue {
        self.vote_data.executed_state_id()
    }

    pub fn certified_block_round(&self) -> Round {
        self.vote_data.block_round()
    }

    pub fn parent_block_id(&self) -> HashValue {
        self.vote_data.parent_block_id()
    }

    pub fn parent_block_round(&self) -> Round {
        self.vote_data.parent_block_round()
    }

    pub fn ledger_info(&self) -> &LedgerInfoWithSignatures {
        &self.signed_ledger_info
    }

    pub fn committed_block_id(&self) -> Option<HashValue> {
        let id = self.ledger_info().ledger_info().consensus_block_id();
        if id.is_zero() {
            None
        } else {
            Some(id)
        }
    }

    #[cfg(any(test, feature = "testing"))]
    pub fn certificate_for_genesis() -> QuorumCert {
        Self::certificate_for_genesis_from_ledger_info(&LedgerInfo::genesis())
    }

    /// QuorumCert for the genesis block deterministically generated from end-epoch LedgerInfo:
    /// - the ID of the block is determined by the generated genesis block.
    /// - the accumulator root hash of the LedgerInfo is set to the last executed state of previous
    ///   epoch.
    /// - the map of signatures is empty because genesis block is implicitly agreed.
    pub fn certificate_for_genesis_from_ledger_info(ledger_info: &LedgerInfo) -> QuorumCert {
        let state_id = ledger_info.transaction_accumulator_hash();
        let vote_data = VoteData::new(*GENESIS_BLOCK_ID, state_id, 0, *GENESIS_BLOCK_ID, 0);
        let signer = ValidatorSigner::genesis();
        let li = LedgerInfo::new(
            ledger_info.version(),
            state_id,
            vote_data.hash(),
            *GENESIS_BLOCK_ID,
            ledger_info.epoch_num() + 1,
            ledger_info.timestamp_usecs(),
            None,
        );
        let signature = signer
            .sign_message(li.hash())
            .expect("Fail to sign genesis ledger info");
        let mut signatures = BTreeMap::new();
        signatures.insert(signer.author(), signature);
        QuorumCert::new(vote_data, LedgerInfoWithSignatures::new(li, signatures))
    }

    pub fn verify(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        let vote_hash = self.vote_data.hash();
        ensure!(
            self.ledger_info().ledger_info().consensus_data_hash() == vote_hash,
            "Quorum Cert's hash mismatch LedgerInfo"
        );
        // Genesis is implicitly agreed upon, it doesn't have real signatures.
        if self.vote_data.block_round() == 0
            && self.vote_data.block_id() == *GENESIS_BLOCK_ID
            && self.vote_data.executed_state_id() == *ACCUMULATOR_PLACEHOLDER_HASH
        {
            return Ok(());
        }
        self.ledger_info()
            .verify(validator)
            .with_context(|e| format!("Fail to verify QuorumCert: {:?}", e))?;
        Ok(())
    }
}

impl TryFrom<network::proto::QuorumCert> for QuorumCert {
    type Error = failure::Error;

    fn try_from(proto: network::proto::QuorumCert) -> failure::Result<Self> {
        let vote_data = proto
            .vote_data
            .ok_or_else(|| format_err!("Missing vote_data"))?
            .try_into()?;
        let signed_ledger_info = proto
            .signed_ledger_info
            .ok_or_else(|| format_err!("Missing signed_ledger_info"))?
            .try_into()?;

        Ok(QuorumCert {
            vote_data,
            signed_ledger_info,
        })
    }
}

impl From<QuorumCert> for network::proto::QuorumCert {
    fn from(cert: QuorumCert) -> Self {
        Self {
            vote_data: Some(cert.vote_data.into()),
            signed_ledger_info: Some(cert.signed_ledger_info.into()),
        }
    }
}
