// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{accumulator_extension_proof::AccumulatorExtensionProof, block::Block};
use libra_crypto::hash::TransactionAccumulatorHasher;
use libra_types::epoch_info::EpochInfo;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// This structure contains all the information needed by safety rules to
/// evaluate a proposal / block for correctness / safety and to produce a Vote.
#[derive(Clone, Deserialize, Serialize)]
pub struct VoteProposal<T> {
    /// Contains the data necessary to construct the parent's execution output state
    /// and the childs in a verifiable way
    accumulator_extension_proof: AccumulatorExtensionProof<TransactionAccumulatorHasher>,
    /// The block / proposal to evaluate
    #[serde(bound(deserialize = "Block<T>: Deserialize<'de>"))]
    block: Block<T>,
    /// An optional field containing the next epoch info.
    next_epoch_info: Option<EpochInfo>,
}

impl<T> VoteProposal<T> {
    pub fn new(
        accumulator_extension_proof: AccumulatorExtensionProof<TransactionAccumulatorHasher>,
        block: Block<T>,
        next_epoch_info: Option<EpochInfo>,
    ) -> Self {
        Self {
            accumulator_extension_proof,
            block,
            next_epoch_info,
        }
    }

    pub fn accumulator_extension_proof(
        &self,
    ) -> &AccumulatorExtensionProof<TransactionAccumulatorHasher> {
        &self.accumulator_extension_proof
    }

    pub fn block(&self) -> &Block<T> {
        &self.block
    }

    pub fn next_epoch_info(&self) -> Option<&EpochInfo> {
        self.next_epoch_info.as_ref()
    }
}

impl<T: PartialEq> Display for VoteProposal<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "VoteProposal[block: {}]", self.block,)
    }
}
