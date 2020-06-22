// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote, vote_proposal::VoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_metrics::monitor;
use libra_types::epoch_change::EpochChangeProof;
use safety_rules::{ConsensusState, Error, TSafetyRules};

/// Wrap safety rules with counters.
pub struct MetricsSafetyRules {
    inner: Box<dyn TSafetyRules + Send + Sync>,
}

impl MetricsSafetyRules {
    pub fn new(inner: Box<dyn TSafetyRules + Send + Sync>) -> Self {
        Self { inner }
    }
}

impl TSafetyRules for MetricsSafetyRules {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        monitor!("safety_rules", self.inner.consensus_state())
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        monitor!("safety_rules", self.inner.initialize(proof))
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal) -> Result<Vote, Error> {
        monitor!(
            "safety_rules",
            self.inner.construct_and_sign_vote(vote_proposal)
        )
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        monitor!("safety_rules", self.inner.sign_proposal(block_data))
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        monitor!("safety_rules", self.inner.sign_timeout(timeout))
    }
}
