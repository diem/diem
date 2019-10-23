// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_data::{BlockData, BlockType},
    block_info::BlockInfo,
    common::{Author, Round},
    quorum_cert::QuorumCert,
    vote_data::VoteData,
};
use failure::{ensure, format_err};
use libra_crypto::hash::{CryptoHash, HashValue};
use libra_types::{
    crypto_proxies::{LedgerInfoWithSignatures, Signature, ValidatorSigner, ValidatorVerifier},
    ledger_info::LedgerInfo,
};
use mirai_annotations::debug_checked_verify_eq;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use std::{
    collections::BTreeMap,
    convert::TryFrom,
    fmt::{Display, Formatter},
};

#[path = "block_test_utils.rs"]
#[cfg(any(test, feature = "testing"))]
pub mod block_test_utils;

#[cfg(test)]
#[path = "block_test.rs"]
pub mod block_test;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
/// Block has the core data of a consensus block that should be persistent when necessary.
/// Each block must know the id of its parent and keep the QuorurmCertificate to that parent.
pub struct Block<T> {
    /// This block's id as a hash value, it is generated at call time
    #[serde(skip)]
    id: HashValue,
    /// The container for the actual block
    block_data: BlockData<T>,
    /// Signature that the hash of this block has been authored by the owner of the private key,
    /// this is only set within Proposal blocks
    signature: Option<Signature>,
}

impl<T: PartialEq> Display for Block<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let nil_marker = if self.is_nil_block() { " (NIL)" } else { "" };
        write!(
            f,
            "[id: {}{}, round: {:02}, parent_id: {}]",
            self.id,
            nil_marker,
            self.round(),
            self.quorum_cert().certified_block().id(),
        )
    }
}

impl<T> Block<T> {
    pub fn author(&self) -> Option<Author> {
        self.block_data.author()
    }

    pub fn epoch(&self) -> u64 {
        self.block_data.epoch()
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    // Is this block a parent of the parameter block?
    #[cfg(test)]
    pub fn is_parent_of(&self, block: &Self) -> bool {
        block.parent_id() == self.id
    }

    pub fn parent_id(&self) -> HashValue {
        self.block_data.quorum_cert().certified_block().id()
    }

    pub fn payload(&self) -> Option<&T> {
        self.block_data.payload()
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        self.block_data.quorum_cert()
    }

    pub fn round(&self) -> Round {
        self.block_data.round()
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.block_data.timestamp_usecs()
    }
}

impl<T> Block<T>
where
    T: PartialEq,
{
    pub fn is_genesis_block(&self) -> bool {
        self.block_data.is_genesis_block()
    }

    pub fn is_nil_block(&self) -> bool {
        self.block_data.is_nil_block()
    }
}

impl<T> Block<T>
where
    T: Default + PartialEq + Serialize,
{
    #[cfg(any(test, feature = "testing"))]
    pub fn make_genesis_block() -> Self {
        Self::make_genesis_block_from_ledger_info(&LedgerInfo::genesis())
    }

    /// Construct new genesis block for next epoch deterministically from the end-epoch LedgerInfo
    /// We carry over most fields except round and block id
    pub fn make_genesis_block_from_ledger_info(ledger_info: &LedgerInfo) -> Self {
        assert!(ledger_info.next_validator_set().is_some());
        let ancestor = BlockInfo::new(
            ledger_info.epoch(),
            0,
            HashValue::zero(),
            ledger_info.transaction_accumulator_hash(),
            ledger_info.version(),
            ledger_info.timestamp_usecs(),
            ledger_info.next_validator_set().cloned(),
        );

        // Genesis carries a placeholder quorum certificate to its parent id with LedgerInfo
        // carrying information about version from the last LedgerInfo of previous epoch.
        let genesis_quorum_cert = QuorumCert::new(
            VoteData::new(ancestor.clone(), ancestor),
            LedgerInfoWithSignatures::new(
                LedgerInfo::new(
                    ledger_info.version(),
                    ledger_info.transaction_accumulator_hash(),
                    HashValue::zero(),
                    HashValue::zero(),
                    ledger_info.epoch(),
                    ledger_info.timestamp_usecs(),
                    None,
                ),
                BTreeMap::new(),
            ),
        );

        let block_data = BlockData::new_genesis(ledger_info.timestamp_usecs(), genesis_quorum_cert);

        Block {
            id: block_data.hash(),
            block_data,
            signature: None,
        }
    }

    /// The NIL blocks are special: they're not carrying any real payload and are generated
    /// independently by different validators just to fill in the round with some QC.
    pub fn new_nil(round: Round, quorum_cert: QuorumCert) -> Self {
        let block_data = BlockData::new_nil(round, quorum_cert);

        Block {
            id: block_data.hash(),
            block_data,
            signature: None,
        }
    }

    pub fn new_proposal(
        payload: T,
        round: Round,
        timestamp_usecs: u64,
        quorum_cert: QuorumCert,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        let block_data = BlockData::new_proposal(
            payload,
            validator_signer.author(),
            round,
            timestamp_usecs,
            quorum_cert,
        );

        Self::new_proposal_from_block_data(block_data, validator_signer)
    }

    pub fn new_proposal_from_block_data(
        block_data: BlockData<T>,
        validator_signer: &ValidatorSigner,
    ) -> Self {
        let id = block_data.hash();
        let signature = validator_signer
            .sign_message(id)
            .expect("Failed to sign message");

        Block {
            id,
            block_data,
            signature: Some(signature.into()),
        }
    }

    /// Verifies that the proposal and the QC are correctly signed.
    /// If this is the genesis block, we skip these checks.
    pub fn validate_signatures(&self, validator: &ValidatorVerifier) -> failure::Result<()> {
        match self.block_data.block_type() {
            BlockType::Genesis => Ok(()),
            BlockType::NilBlock => self.quorum_cert().verify(validator),
            BlockType::Proposal { author, .. } => {
                let signature = self
                    .signature
                    .as_ref()
                    .ok_or_else(|| format_err!("Missing signature in Proposal"))?;
                signature.verify(validator, *author, self.id())?;
                self.quorum_cert().verify(validator)
            }
        }
    }

    /// Makes sure that the proposal makes sense, independently of the current state.
    /// If this is the genesis block, we skip these checks.
    pub fn verify_well_formed(&self) -> failure::Result<()> {
        if self.is_genesis_block() {
            return Ok(());
        }
        debug_checked_verify_eq!(
            self.id(),
            self.block_data.hash(),
            "Block id mismatch the hash"
        );
        ensure!(
            self.quorum_cert().certified_block().round() < self.round(),
            "Block has invalid round"
        );
        Ok(())
    }
}

impl<T> TryFrom<network::proto::Block> for Block<T>
where
    T: DeserializeOwned + Serialize,
{
    type Error = failure::Error;

    fn try_from(proto: network::proto::Block) -> failure::Result<Self> {
        Ok(lcs::from_bytes(&proto.bytes)?)
    }
}

impl<T> TryFrom<Block<T>> for network::proto::Block
where
    T: Serialize + Default + PartialEq,
{
    type Error = failure::Error;

    fn try_from(block: Block<T>) -> failure::Result<Self> {
        Ok(Self {
            bytes: lcs::to_bytes(&block)?,
        })
    }
}

impl<'de, T: DeserializeOwned + Serialize> Deserialize<'de> for Block<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BlockWithoutId<T> {
            #[serde(bound(deserialize = "BlockData<T>: Deserialize<'de>"))]
            block_data: BlockData<T>,
            signature: Option<Signature>,
        };

        let BlockWithoutId {
            block_data,
            signature,
        } = BlockWithoutId::deserialize(deserializer)?;

        Ok(Block {
            id: block_data.hash(),
            block_data,
            signature,
        })
    }
}
