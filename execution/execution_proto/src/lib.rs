// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod proto;

#[cfg(test)]
mod protobuf_conversion_test;

use crypto::HashValue;
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use types::{
    crypto_proxies::LedgerInfoWithSignatures,
    transaction::{SignedTransaction, TransactionListWithProof, TransactionStatus, Version},
    validator_set::ValidatorSet,
    vm_error::VMStatus,
};

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[ProtoType(crate::proto::execution::ExecuteBlockRequest)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct ExecuteBlockRequest {
    /// The list of transactions from consensus.
    pub transactions: Vec<SignedTransaction>,

    /// Id of parent block.
    pub parent_block_id: HashValue,

    /// Id of current block.
    pub block_id: HashValue,
}

impl ExecuteBlockRequest {
    pub fn new(
        transactions: Vec<SignedTransaction>,
        parent_block_id: HashValue,
        block_id: HashValue,
    ) -> Self {
        ExecuteBlockRequest {
            transactions,
            parent_block_id,
            block_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub struct ExecuteBlockResponse {
    /// Root hash of the transaction accumulator as if all transactions in this block are applied.
    root_hash: HashValue,

    /// Status code for each individual transaction in this block.
    status: Vec<TransactionStatus>,

    /// The corresponding ledger version when this block is committed.
    version: Version,

    /// If set, these are the set of validators that will be used to start the next epoch
    /// immediately after this state is committed.
    validators: Option<ValidatorSet>,
}

impl ExecuteBlockResponse {
    pub fn new(
        root_hash: HashValue,
        status: Vec<TransactionStatus>,
        version: Version,
        validators: Option<ValidatorSet>,
    ) -> Self {
        ExecuteBlockResponse {
            root_hash,
            status,
            version,
            validators,
        }
    }

    pub fn root_hash(&self) -> HashValue {
        self.root_hash
    }

    pub fn status(&self) -> &[TransactionStatus] {
        &self.status
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn validators(&self) -> &Option<ValidatorSet> {
        &self.validators
    }
}

impl FromProto for ExecuteBlockResponse {
    type ProtoType = crate::proto::execution::ExecuteBlockResponse;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        Ok(ExecuteBlockResponse {
            root_hash: HashValue::from_slice(object.get_root_hash())?,
            status: object
                .take_status()
                .into_iter()
                .map(|proto_vm_status| {
                    let vm_status = VMStatus::from_proto(proto_vm_status)?;
                    Ok(vm_status.into())
                })
                .collect::<Result<Vec<_>>>()?,
            version: object.get_version(),
            validators: object
                .validators
                .take()
                .map(ValidatorSet::from_proto)
                .transpose()?,
        })
    }
}

impl IntoProto for ExecuteBlockResponse {
    type ProtoType = crate::proto::execution::ExecuteBlockResponse;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        out.set_root_hash(self.root_hash.to_vec());
        out.set_status(
            self.status
                .into_iter()
                .map(|transaction_status| {
                    let vm_status = match transaction_status {
                        TransactionStatus::Keep(status) => status,
                        TransactionStatus::Discard(status) => status,
                    };
                    vm_status.into_proto()
                })
                .collect(),
        );
        out.set_version(self.version);
        if let Some(validators) = self.validators {
            out.set_validators(validators.into_proto());
        }
        out
    }
}

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::execution::CommitBlockRequest)]
pub struct CommitBlockRequest {
    pub ledger_info_with_sigs: LedgerInfoWithSignatures,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
pub enum CommitBlockResponse {
    Succeeded,
    Failed,
}

impl FromProto for CommitBlockResponse {
    type ProtoType = crate::proto::execution::CommitBlockResponse;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        use crate::proto::execution::CommitBlockStatus;
        Ok(match object.get_status() {
            CommitBlockStatus::SUCCEEDED => CommitBlockResponse::Succeeded,
            CommitBlockStatus::FAILED => CommitBlockResponse::Failed,
        })
    }
}

impl IntoProto for CommitBlockResponse {
    type ProtoType = crate::proto::execution::CommitBlockResponse;

    fn into_proto(self) -> Self::ProtoType {
        use crate::proto::execution::CommitBlockStatus;
        let mut out = Self::ProtoType::new();
        out.set_status(match self {
            CommitBlockResponse::Succeeded => CommitBlockStatus::SUCCEEDED,
            CommitBlockResponse::Failed => CommitBlockStatus::FAILED,
        });
        out
    }
}

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::execution::ExecuteChunkRequest)]
pub struct ExecuteChunkRequest {
    pub txn_list_with_proof: TransactionListWithProof,
    pub ledger_info_with_sigs: LedgerInfoWithSignatures,
}

#[derive(Clone, Debug, Eq, PartialEq, FromProto, IntoProto)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
#[ProtoType(crate::proto::execution::ExecuteChunkResponse)]
pub struct ExecuteChunkResponse {}
