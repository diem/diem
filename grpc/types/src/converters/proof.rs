// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module has definition of various proofs.

//use crate::{
//    account_state_blob::AccountStateBlob,
//    ledger_info::LedgerInfo,
//    transaction::{TransactionInfo, Version},
//};
use anyhow::{bail, format_err, Error, Result};
use libra_crypto::{
    hash::{CryptoHasher, ACCUMULATOR_PLACEHOLDER_HASH, SPARSE_MERKLE_PLACEHOLDER_HASH},
    HashValue,
};
use std::convert::{TryFrom, TryInto};

use libra_types::proof::{
    definition::{
        AccountStateProof, AccumulatorConsistencyProof, AccumulatorProof, AccumulatorRangeProof,
        EventProof, SparseMerkleProof, SparseMerkleRangeProof, TransactionListProof,
        TransactionProof,
    },
    SparseMerkleLeafNode,
};

/// Converts sibling nodes from Protobuf format to Rust format, using the fact that empty byte
/// arrays represent placeholder hashes.
fn from_proto_siblings(siblings: Vec<Vec<u8>>, placeholder: HashValue) -> Result<Vec<HashValue>> {
    debug_assert!(
        placeholder == *ACCUMULATOR_PLACEHOLDER_HASH
            || placeholder == *SPARSE_MERKLE_PLACEHOLDER_HASH,
        "Placeholder can only be ACCUMULATOR_PLACEHOLDER_HASH or SPARSE_MERKLE_PLACEHOLDER_HASH.",
    );

    siblings
        .into_iter()
        .map(|hash_bytes| {
            if hash_bytes.is_empty() {
                Ok(placeholder)
            } else {
                HashValue::from_slice(&hash_bytes)
            }
        })
        .collect()
}

/// Converts sibling nodes from Rust format to Protobuf format. The placeholder hashes are
/// converted to empty byte arrays.
fn into_proto_siblings(siblings: Vec<HashValue>, placeholder: HashValue) -> Vec<Vec<u8>> {
    debug_assert!(
        placeholder == *ACCUMULATOR_PLACEHOLDER_HASH
            || placeholder == *SPARSE_MERKLE_PLACEHOLDER_HASH,
        "Placeholder can only be ACCUMULATOR_PLACEHOLDER_HASH or SPARSE_MERKLE_PLACEHOLDER_HASH.",
    );

    siblings
        .into_iter()
        .map(|sibling| {
            if sibling != placeholder {
                sibling.to_vec()
            } else {
                vec![]
            }
        })
        .collect()
}

impl<H> TryFrom<crate::proto::types::AccumulatorProof> for AccumulatorProof<H>
where
    H: CryptoHasher,
{
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccumulatorProof) -> Result<Self> {
        let siblings = from_proto_siblings(proto_proof.siblings, *ACCUMULATOR_PLACEHOLDER_HASH)?;
        Ok(AccumulatorProof::new(siblings))
    }
}

impl<H> From<&AccumulatorProof<H>> for crate::proto::types::AccumulatorProof
where
    H: CryptoHasher,
{
    fn from(proof: &AccumulatorProof<H>) -> Self {
        let mut proto_proof = Self::default();
        proto_proof.siblings =
            into_proto_siblings(proof.siblings().to_vec(), *ACCUMULATOR_PLACEHOLDER_HASH);
        proto_proof
    }
}
impl<H> From<AccumulatorProof<H>> for crate::proto::types::AccumulatorProof
where
    H: CryptoHasher,
{
    fn from(proof: AccumulatorProof<H>) -> Self {
        Self::from(&proof)
    }
}

impl TryFrom<crate::proto::types::SparseMerkleProof> for SparseMerkleProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::SparseMerkleProof) -> Result<Self> {
        let proto_leaf = proto_proof.leaf;
        let leaf = if proto_leaf.is_empty() {
            None
        } else if proto_leaf.len() == HashValue::LENGTH * 2 {
            let key = HashValue::from_slice(&proto_leaf[0..HashValue::LENGTH])?;
            let value_hash = HashValue::from_slice(&proto_leaf[HashValue::LENGTH..])?;
            Some(SparseMerkleLeafNode::new(key, value_hash))
        } else {
            bail!(
                "Mailformed proof. Leaf has {} bytes. Expect 0 or {} bytes.",
                proto_leaf.len(),
                HashValue::LENGTH * 2
            );
        };

        let siblings = from_proto_siblings(proto_proof.siblings, *SPARSE_MERKLE_PLACEHOLDER_HASH)?;

        Ok(SparseMerkleProof::new(leaf, siblings))
    }
}

impl From<&SparseMerkleProof> for crate::proto::types::SparseMerkleProof {
    fn from(proof: &SparseMerkleProof) -> Self {
        let mut proto_proof = Self::default();
        // If a leaf is present, we write the key and value hash as a single byte array of 64
        // bytes. Otherwise we write an empty byte array.
        if let Some(leaf) = proof.leaf() {
            proto_proof.leaf.extend_from_slice(leaf.key().as_ref());
            proto_proof
                .leaf
                .extend_from_slice(leaf.value_hash().as_ref());
        }
        proto_proof.siblings =
            into_proto_siblings(proof.siblings().to_vec(), *SPARSE_MERKLE_PLACEHOLDER_HASH);
        proto_proof
    }
}

impl From<SparseMerkleProof> for crate::proto::types::SparseMerkleProof {
    fn from(proof: SparseMerkleProof) -> Self {
        Self::from(&proof)
    }
}

impl TryFrom<crate::proto::types::AccumulatorConsistencyProof> for AccumulatorConsistencyProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccumulatorConsistencyProof) -> Result<Self> {
        let subtrees = proto_proof
            .subtrees
            .into_iter()
            .map(|hash_bytes| HashValue::from_slice(&hash_bytes))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(subtrees))
    }
}

impl From<AccumulatorConsistencyProof> for crate::proto::types::AccumulatorConsistencyProof {
    fn from(proof: AccumulatorConsistencyProof) -> Self {
        Self {
            subtrees: proof.subtrees().iter().map(HashValue::to_vec).collect(),
        }
    }
}

impl<H> TryFrom<crate::proto::types::AccumulatorRangeProof> for AccumulatorRangeProof<H>
where
    H: CryptoHasher,
{
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccumulatorRangeProof) -> Result<Self> {
        let left_siblings =
            from_proto_siblings(proto_proof.left_siblings, *ACCUMULATOR_PLACEHOLDER_HASH)?;
        let right_siblings =
            from_proto_siblings(proto_proof.right_siblings, *ACCUMULATOR_PLACEHOLDER_HASH)?;

        Ok(Self::new(left_siblings, right_siblings))
    }
}

impl<H> From<&AccumulatorRangeProof<H>> for crate::proto::types::AccumulatorRangeProof
where
    H: CryptoHasher,
{
    fn from(proof: &AccumulatorRangeProof<H>) -> Self {
        let mut proto_proof = Self::default();
        proto_proof.left_siblings = into_proto_siblings(
            proof.left_siblings().to_vec(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
        );
        proto_proof.right_siblings = into_proto_siblings(
            proof.right_siblings().to_vec(),
            *ACCUMULATOR_PLACEHOLDER_HASH,
        );
        proto_proof
    }
}

impl<H> From<AccumulatorRangeProof<H>> for crate::proto::types::AccumulatorRangeProof
where
    H: CryptoHasher,
{
    fn from(proof: AccumulatorRangeProof<H>) -> Self {
        Self::from(&proof)
    }
}

impl TryFrom<crate::proto::types::SparseMerkleRangeProof> for SparseMerkleRangeProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::SparseMerkleRangeProof) -> Result<Self> {
        let right_siblings =
            from_proto_siblings(proto_proof.right_siblings, *SPARSE_MERKLE_PLACEHOLDER_HASH)?;
        Ok(Self::new(right_siblings))
    }
}

impl From<SparseMerkleRangeProof> for crate::proto::types::SparseMerkleRangeProof {
    fn from(proof: SparseMerkleRangeProof) -> Self {
        let right_siblings = into_proto_siblings(
            proof.right_siblings().to_vec(),
            *SPARSE_MERKLE_PLACEHOLDER_HASH,
        );
        Self { right_siblings }
    }
}

impl TryFrom<crate::proto::types::TransactionProof> for TransactionProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::TransactionProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;

        Ok(TransactionProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
        ))
    }
}

impl From<TransactionProof> for crate::proto::types::TransactionProof {
    fn from(proof: TransactionProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof().into(),
            ),
            transaction_info: Some(proof.transaction_info().into()),
        }
    }
}

impl TryFrom<crate::proto::types::AccountStateProof> for AccountStateProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::AccountStateProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;
        let transaction_info_to_account_proof = proto_proof
            .transaction_info_to_account_proof
            .ok_or_else(|| format_err!("Missing transaction_info_to_account_proof"))?
            .try_into()?;

        Ok(AccountStateProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_account_proof,
        ))
    }
}

impl From<AccountStateProof> for crate::proto::types::AccountStateProof {
    fn from(proof: AccountStateProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof().into(),
            ),
            transaction_info: Some(proof.transaction_info().into()),
            transaction_info_to_account_proof: Some(
                proof.transaction_info_to_account_proof().into(),
            ),
        }
    }
}

impl TryFrom<crate::proto::types::EventProof> for EventProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::EventProof) -> Result<Self> {
        let ledger_info_to_transaction_info_proof = proto_proof
            .ledger_info_to_transaction_info_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_info_proof"))?
            .try_into()?;
        let transaction_info = proto_proof
            .transaction_info
            .ok_or_else(|| format_err!("Missing transaction_info"))?
            .try_into()?;
        let transaction_info_to_event_proof = proto_proof
            .transaction_info_to_event_proof
            .ok_or_else(|| format_err!("Missing transaction_info_to_account_proof"))?
            .try_into()?;

        Ok(EventProof::new(
            ledger_info_to_transaction_info_proof,
            transaction_info,
            transaction_info_to_event_proof,
        ))
    }
}

impl From<EventProof> for crate::proto::types::EventProof {
    fn from(proof: EventProof) -> Self {
        Self {
            ledger_info_to_transaction_info_proof: Some(
                proof.ledger_info_to_transaction_info_proof().into(),
            ),
            transaction_info: Some(proof.transaction_info().into()),
            transaction_info_to_event_proof: Some(proof.transaction_info_to_event_proof().into()),
        }
    }
}

impl TryFrom<crate::proto::types::TransactionListProof> for TransactionListProof {
    type Error = Error;

    fn try_from(proto_proof: crate::proto::types::TransactionListProof) -> Result<Self> {
        let ledger_info_to_transaction_infos_proof = proto_proof
            .ledger_info_to_transaction_infos_proof
            .ok_or_else(|| format_err!("Missing ledger_info_to_transaction_infos_proof"))?
            .try_into()?;
        let transaction_infos = proto_proof
            .transaction_infos
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()?;

        Ok(TransactionListProof::new(
            ledger_info_to_transaction_infos_proof,
            transaction_infos,
        ))
    }
}

impl From<TransactionListProof> for crate::proto::types::TransactionListProof {
    fn from(proof: TransactionListProof) -> Self {
        Self {
            ledger_info_to_transaction_infos_proof: Some(
                proof.ledger_info_to_transaction_infos_proof().into(),
            ),
            transaction_infos: proof.transaction_infos().iter().map(Into::into).collect(),
        }
    }
}
