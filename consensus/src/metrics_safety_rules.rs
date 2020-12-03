// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::persistent_liveness_storage::PersistentLivenessStorage;
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_metrics::monitor;
use diem_types::epoch_change::EpochChangeProof;
use safety_rules::{ConsensusState, Error, TSafetyRules};
use std::sync::Arc;

/// Wrap safety rules with counters.
pub struct MetricsSafetyRules {
    inner: Box<dyn TSafetyRules + Send + Sync>,
    storage: Arc<dyn PersistentLivenessStorage>,
}

impl MetricsSafetyRules {
    pub fn new(
        inner: Box<dyn TSafetyRules + Send + Sync>,
        storage: Arc<dyn PersistentLivenessStorage>,
    ) -> Self {
        Self { inner, storage }
    }

    pub fn perform_initialize(&mut self) -> Result<(), Error> {
        let consensus_state = self.consensus_state()?;
        let sr_waypoint = consensus_state.waypoint();
        let proofs = self
            .storage
            .retrieve_epoch_change_proof(sr_waypoint.version())
            .map_err(|e| {
                Error::InternalError(format!(
                    "Unable to retrieve Waypoint state from storage, encountered Error:{}",
                    e
                ))
            })?;
        self.initialize(&proofs)
    }
}

impl TSafetyRules for MetricsSafetyRules {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        monitor!("safety_rules", self.inner.consensus_state())
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        monitor!("safety_rules", self.inner.initialize(proof))
    }

    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let mut result = monitor!(
            "safety_rules",
            self.inner.construct_and_sign_vote(vote_proposal)
        );

        if let Err(Error::NotInitialized(_res)) = result {
            self.perform_initialize()?;
            result = monitor!(
                "safety_rules",
                self.inner.construct_and_sign_vote(vote_proposal)
            );
        }
        result
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let mut result = monitor!("safety_rules", self.inner.sign_proposal(block_data.clone()));
        if let Err(Error::NotInitialized(_res)) = result {
            self.perform_initialize()?;
            result = monitor!("safety_rules", self.inner.sign_proposal(block_data));
        }
        result
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let mut result = monitor!("safety_rules", self.inner.sign_timeout(timeout));
        if let Err(Error::NotInitialized(_res)) = result {
            self.perform_initialize()?;
            result = monitor!("safety_rules", self.inner.sign_timeout(timeout));
        }
        result
    }
}
