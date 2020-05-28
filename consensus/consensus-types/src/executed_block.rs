// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, common::Round, quorum_cert::QuorumCert};
use executor_types::StateComputeResult;
use libra_crypto::hash::HashValue;
use libra_types::block_info::BlockInfo;
use std::fmt::{Display, Formatter};

/// ExecutedBlocks are managed in a speculative tree, the committed blocks form a chain. Besides
/// block data, each executed block also has other derived meta data which could be regenerated from
/// blocks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutedBlock<T> {
    /// Block data that cannot be regenerated.
    block: Block<T>,
    /// The state_compute_result is calculated for all the pending blocks prior to insertion to
    /// the tree. The execution results are not persisted: they're recalculated again for the
    /// pending blocks upon restart.
    state_compute_result: StateComputeResult,
}

impl<T> Display for ExecutedBlock<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.block())
    }
}

impl<T> ExecutedBlock<T> {
    pub fn new(block: Block<T>, state_compute_result: StateComputeResult) -> Self {
        Self {
            block,
            state_compute_result,
        }
    }

    pub fn block(&self) -> &Block<T> {
        &self.block
    }

    pub fn root_hash(&self) -> HashValue {
        self.state_compute_result.root_hash()
    }

    pub fn epoch(&self) -> u64 {
        self.block().epoch()
    }

    pub fn id(&self) -> HashValue {
        self.block().id()
    }

    pub fn payload(&self) -> Option<&T> {
        self.block().payload()
    }

    pub fn parent_id(&self) -> HashValue {
        self.quorum_cert().certified_block().id()
    }

    pub fn quorum_cert(&self) -> &QuorumCert {
        self.block().quorum_cert()
    }

    pub fn round(&self) -> Round {
        self.block().round()
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.block().timestamp_usecs()
    }

    pub fn compute_result(&self) -> &StateComputeResult {
        &self.state_compute_result
    }

    pub fn block_info(&self) -> BlockInfo {
        self.block().gen_block_info(
            self.compute_result().root_hash(),
            self.compute_result().version(),
            self.compute_result().epoch_state().clone(),
        )
    }
}

impl<T> ExecutedBlock<T>
where
    T: PartialEq,
{
    pub fn is_nil_block(&self) -> bool {
        self.block().is_nil_block()
    }
}
