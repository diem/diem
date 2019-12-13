// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{block::Block, common::Round, quorum_cert::QuorumCert};
use executor::{ExecutedTrees, ProcessedVMOutput, StateComputeResult};
use libra_crypto::hash::HashValue;
use libra_types::block_info::BlockInfo;
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};

/// ExecutedBlocks are managed in a speculative tree, the committed blocks form a chain. Besides
/// block data, each executed block also has other derived meta data which could be regenerated from
/// blocks.
#[derive(Clone, Debug)]
pub struct ExecutedBlock<T> {
    /// Block data that cannot be regenerated.
    block: Block<T>,
    /// The execution output is calculated for all the pending blocks prior to insertion to
    /// the tree. The execution results are not persisted: they're recalculated again for the
    /// pending blocks upon restart.
    output: Arc<ProcessedVMOutput>,
}

impl<T: PartialEq> PartialEq for ExecutedBlock<T> {
    fn eq(&self, other: &Self) -> bool {
        self.block == other.block && self.compute_result() == other.compute_result()
    }
}

impl<T: Eq> Eq for ExecutedBlock<T> where T: PartialEq {}

impl<T: PartialEq> Display for ExecutedBlock<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.block().fmt(f)
    }
}

impl<T> ExecutedBlock<T> {
    pub fn new(block: Block<T>, output: ProcessedVMOutput) -> Self {
        Self {
            block,
            output: Arc::new(output),
        }
    }

    pub fn block(&self) -> &Block<T> {
        &self.block
    }

    pub fn compute_result(&self) -> StateComputeResult {
        self.output().state_compute_result()
    }

    pub fn epoch(&self) -> u64 {
        self.block().epoch()
    }

    pub fn executed_trees(&self) -> &ExecutedTrees {
        self.output.executed_trees()
    }

    pub fn id(&self) -> HashValue {
        self.block().id()
    }

    pub fn output(&self) -> &Arc<ProcessedVMOutput> {
        &self.output
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

    pub fn transaction_info_hashes(&self) -> Vec<HashValue> {
        self.output
            .transaction_data()
            .iter()
            .filter_map(|x| x.txn_info_hash())
            .collect()
    }

    pub fn block_info(&self) -> BlockInfo {
        let executed_state = self.compute_result().executed_state;
        self.block().gen_block_info(
            executed_state.state_id,
            executed_state.version,
            executed_state.validators,
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
