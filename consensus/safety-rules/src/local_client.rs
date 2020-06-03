// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, quorum_cert::QuorumCert, timeout::Timeout, vote::Vote,
    vote_proposal::VoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_types::epoch_change::EpochChangeProof;
use std::sync::{Arc, RwLock};

/// A local interface into SafetyRules. Constructed in such a way that the container / caller
/// cannot distinguish this API from an actual client/server process without being exposed to
/// the actual container instead the caller can access a Box<dyn TSafetyRules>.
pub struct LocalClient {
    internal: Arc<RwLock<SafetyRules>>,
}

impl LocalClient {
    pub fn new(internal: Arc<RwLock<SafetyRules>>) -> Self {
        Self { internal }
    }
}

impl TSafetyRules for LocalClient {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        self.internal.write().unwrap().consensus_state()
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        self.internal.write().unwrap().initialize(proof)
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.internal.write().unwrap().update(qc)
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal) -> Result<Vote, Error> {
        self.internal
            .write()
            .unwrap()
            .construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        self.internal.write().unwrap().sign_proposal(block_data)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.internal.write().unwrap().sign_timeout(timeout)
    }
}
