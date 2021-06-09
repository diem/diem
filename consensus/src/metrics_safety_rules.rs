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

    fn retry<T, F: FnMut(&mut Box<dyn TSafetyRules + Send + Sync>) -> Result<T, Error>>(
        &mut self,
        mut f: F,
    ) -> Result<T, Error> {
        let result = f(&mut self.inner);
        match result {
            Err(Error::NotInitialized(_)) | Err(Error::IncorrectEpoch(_, _)) => {
                self.perform_initialize()?;
                f(&mut self.inner)
            }
            _ => result,
        }
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
        self.retry(|inner| monitor!("safety_rules", inner.construct_and_sign_vote(vote_proposal)))
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        self.retry(|inner| monitor!("safety_rules", inner.sign_proposal(block_data.clone())))
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.retry(|inner| monitor!("safety_rules", inner.sign_timeout(timeout)))
    }
}
