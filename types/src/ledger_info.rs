// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    block_info::{BlockInfo, Round},
    crypto_proxies::ValidatorSet,
    transaction::Version,
    validator_verifier::{ValidatorVerifier, VerifyError},
};
use anyhow::{ensure, format_err, Error, Result};
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue, *,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    convert::{TryFrom, TryInto},
    fmt::{Display, Formatter},
};

/// This structure serves a dual purpose.
///
/// First, if this structure is signed by 2f+1 validators it signifies the state of the ledger at
/// version `version` -- it contains the transaction accumulator at that version which commits to
/// all historical transactions. This structure may be expanded to include other information that
/// is derived from that accumulator (e.g. the current time according to the time contract) to
/// reduce the number of proofs a client must get.
///
/// Second, the structure contains a `consensus_data_hash` value. This is the hash of an internal
/// data structure that represents a block that is voted on in HotStuff. If 2f+1 signatures are
/// gathered on the same ledger info that represents a Quorum Certificate (QC) on the consensus
/// data.
///
/// Combining these two concepts, when a validator votes on a block, B it votes for a
/// LedgerInfo with the `version` being the latest version that will be committed if B gets 2f+1
/// votes. It sets `consensus_data_hash` to represent B so that if those 2f+1 votes are gathered a
/// QC is formed on B.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct LedgerInfo {
    commit_info: BlockInfo,

    /// Hash of consensus specific data that is opaque to all parts of the system other than
    /// consensus.
    consensus_data_hash: HashValue,
}

impl Display for LedgerInfo {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "LedgerInfo: [commit_info: {}]", self.commit_info())
    }
}

impl LedgerInfo {
    /// Constructs a `LedgerInfo` object based on the given commit info and vote data hash.
    pub fn new(commit_info: BlockInfo, consensus_data_hash: HashValue) -> Self {
        Self {
            commit_info,
            consensus_data_hash,
        }
    }

    /// Create a new LedgerInfo at genesis with the given genesis state and
    /// initial validator set.
    pub fn genesis(genesis_state_root_hash: HashValue, validator_set: ValidatorSet) -> Self {
        Self::new(
            BlockInfo::genesis(genesis_state_root_hash, validator_set),
            HashValue::zero(),
        )
    }

    #[cfg(any(test, feature = "fuzzing"))]
    pub fn mock_genesis() -> Self {
        Self::new(BlockInfo::mock_genesis(), HashValue::zero())
    }

    /// The `BlockInfo` of a committed block.
    pub fn commit_info(&self) -> &BlockInfo {
        &self.commit_info
    }

    /// A series of wrapper functions for the data stored in the commit info. For the detailed
    /// information, please refer to `BlockInfo`
    pub fn epoch(&self) -> u64 {
        self.commit_info.epoch()
    }

    pub fn round(&self) -> Round {
        self.commit_info.round()
    }

    pub fn consensus_block_id(&self) -> HashValue {
        self.commit_info.id()
    }

    pub fn transaction_accumulator_hash(&self) -> HashValue {
        self.commit_info.executed_state_id()
    }

    pub fn version(&self) -> Version {
        self.commit_info.version()
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.commit_info.timestamp_usecs()
    }

    pub fn next_validator_set(&self) -> Option<&ValidatorSet> {
        self.commit_info.next_validator_set()
    }

    /// Returns hash of consensus voting data in this `LedgerInfo`.
    pub fn consensus_data_hash(&self) -> HashValue {
        self.consensus_data_hash
    }

    pub fn set_consensus_data_hash(&mut self, consensus_data_hash: HashValue) {
        self.consensus_data_hash = consensus_data_hash;
    }
}

impl TryFrom<crate::proto::types::LedgerInfo> for LedgerInfo {
    type Error = Error;

    fn try_from(proto: crate::proto::types::LedgerInfo) -> Result<Self> {
        let version = proto.version;
        let transaction_accumulator_hash =
            HashValue::from_slice(&proto.transaction_accumulator_hash)?;
        let consensus_data_hash = HashValue::from_slice(&proto.consensus_data_hash)?;
        let consensus_block_id = HashValue::from_slice(&proto.consensus_block_id)?;
        let epoch = proto.epoch;
        let round = proto.round;
        let timestamp_usecs = proto.timestamp_usecs;

        let next_validator_set = proto
            .next_validator_set
            .map(ValidatorSet::try_from)
            .transpose()?;
        Ok(LedgerInfo::new(
            BlockInfo::new(
                epoch,
                round,
                consensus_block_id,
                transaction_accumulator_hash,
                version,
                timestamp_usecs,
                next_validator_set,
            ),
            consensus_data_hash,
        ))
    }
}

impl From<LedgerInfo> for crate::proto::types::LedgerInfo {
    fn from(ledger_info: LedgerInfo) -> Self {
        Self {
            version: ledger_info.version(),
            transaction_accumulator_hash: ledger_info.transaction_accumulator_hash().to_vec(),
            consensus_data_hash: ledger_info.consensus_data_hash().to_vec(),
            consensus_block_id: ledger_info.consensus_block_id().to_vec(),
            epoch: ledger_info.epoch(),
            round: ledger_info.round(),
            timestamp_usecs: ledger_info.timestamp_usecs(),
            next_validator_set: ledger_info.next_validator_set().cloned().map(Into::into),
        }
    }
}

impl CryptoHash for LedgerInfo {
    type Hasher = LedgerInfoHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(&lcs::to_bytes(self).expect("Serialization should work."));
        state.finish()
    }
}

/// The validator node returns this structure which includes signatures
/// from validators that confirm the state.  The client needs to only pass back
/// the LedgerInfo element since the validator node doesn't need to know the signatures
/// again when the client performs a query, those are only there for the client
/// to be able to verify the state
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LedgerInfoWithSignatures<Sig> {
    ledger_info: LedgerInfo,
    /// The validator is identified by its account address: in order to verify a signature
    /// one needs to retrieve the public key of the validator for the given epoch.
    signatures: BTreeMap<AccountAddress, Sig>,
}

impl<Sig> Display for LedgerInfoWithSignatures<Sig> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.ledger_info)
    }
}

impl<Sig: Signature> LedgerInfoWithSignatures<Sig> {
    pub fn new(ledger_info: LedgerInfo, signatures: BTreeMap<AccountAddress, Sig>) -> Self {
        LedgerInfoWithSignatures {
            ledger_info,
            signatures,
        }
    }

    /// Create a new `LedgerInfoWithSignatures` at genesis with the given genesis
    /// state and initial validator set.
    ///
    /// Note that the genesis `LedgerInfoWithSignatures` is unsigned. Validators
    /// and FullNodes are configured with the same genesis transaction and generate
    /// an identical genesis `LedgerInfoWithSignatures` independently. In contrast,
    /// Clients will likely use a waypoint generated from the genesis `LedgerInfo`.
    pub fn genesis(genesis_state_root_hash: HashValue, validator_set: ValidatorSet) -> Self {
        Self::new(
            LedgerInfo::genesis(genesis_state_root_hash, validator_set),
            BTreeMap::new(),
        )
    }

    pub fn ledger_info(&self) -> &LedgerInfo {
        &self.ledger_info
    }

    pub fn add_signature(&mut self, validator: AccountAddress, signature: Sig) {
        self.signatures.entry(validator).or_insert(signature);
    }

    pub fn remove_signature(&mut self, validator: AccountAddress) {
        self.signatures.remove(&validator);
    }

    pub fn signatures(&self) -> &BTreeMap<AccountAddress, Sig> {
        &self.signatures
    }

    pub fn verify_signatures(
        &self,
        validator: &ValidatorVerifier<Sig::VerifyingKeyMaterial>,
    ) -> ::std::result::Result<(), VerifyError> {
        let ledger_hash = self.ledger_info().hash();
        validator.batch_verify_aggregated_signature(ledger_hash, self.signatures())
    }
}

impl<Sig: Signature> TryFrom<crate::proto::types::LedgerInfoWithSignatures>
    for LedgerInfoWithSignatures<Sig>
{
    type Error = Error;

    fn try_from(proto: crate::proto::types::LedgerInfoWithSignatures) -> Result<Self> {
        let ledger_info = proto
            .ledger_info
            .ok_or_else(|| format_err!("Missing ledger_info"))?
            .try_into()?;

        let signatures_proto = proto.signatures;
        let num_signatures = signatures_proto.len();
        let signatures = signatures_proto
            .into_iter()
            .map(|proto| {
                let validator_id = AccountAddress::try_from(proto.validator_id)?;
                let signature_bytes: &[u8] = proto.signature.as_ref();
                let signature = Sig::try_from(signature_bytes)?;
                Ok((validator_id, signature))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        ensure!(
            signatures.len() == num_signatures,
            "Signatures should be from different validators."
        );

        Ok(LedgerInfoWithSignatures {
            ledger_info,
            signatures,
        })
    }
}

impl<Sig: Signature> From<LedgerInfoWithSignatures<Sig>>
    for crate::proto::types::LedgerInfoWithSignatures
{
    fn from(ledger_info_with_sigs: LedgerInfoWithSignatures<Sig>) -> Self {
        let ledger_info = Some(ledger_info_with_sigs.ledger_info.into());
        let signatures = ledger_info_with_sigs
            .signatures
            .into_iter()
            .map(
                |(validator_id, signature)| crate::proto::types::ValidatorSignature {
                    validator_id: validator_id.to_vec(),
                    signature: signature.to_bytes().to_vec(),
                },
            )
            .collect();

        Self {
            signatures,
            ledger_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block_info::BlockInfo,
        ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
        validator_signer::ValidatorSigner,
    };
    use libra_crypto::{ed25519::*, HashValue};
    use std::collections::BTreeMap;

    #[test]
    fn test_signatures_hash() {
        let ledger_info = LedgerInfo::new(BlockInfo::empty(), HashValue::zero());

        let random_hash = HashValue::random();
        const NUM_SIGNERS: u8 = 7;
        // Generate NUM_SIGNERS random signers.
        let validator_signers: Vec<ValidatorSigner<Ed25519PrivateKey>> = (0..NUM_SIGNERS)
            .map(|i| ValidatorSigner::random([i; 32]))
            .collect();
        let mut author_to_signature_map = BTreeMap::new();
        for validator in validator_signers.iter() {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }

        let ledger_info_with_signatures =
            LedgerInfoWithSignatures::new(ledger_info.clone(), author_to_signature_map);

        // Add the signatures in reverse order and ensure the serialization matches
        let mut author_to_signature_map = BTreeMap::new();
        for validator in validator_signers.iter().rev() {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }

        let ledger_info_with_signatures_reversed =
            LedgerInfoWithSignatures::new(ledger_info, author_to_signature_map);

        let ledger_info_with_signatures_bytes =
            lcs::to_bytes(&ledger_info_with_signatures).expect("block serialization failed");
        let ledger_info_with_signatures_reversed_bytes =
            lcs::to_bytes(&ledger_info_with_signatures_reversed)
                .expect("block serialization failed");

        assert_eq!(
            ledger_info_with_signatures_bytes,
            ledger_info_with_signatures_reversed_bytes
        );
    }
}
