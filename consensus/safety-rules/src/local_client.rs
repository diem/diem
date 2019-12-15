// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, common::Payload, quorum_cert::QuorumCert,
    timeout::Timeout, vote::Vote, vote_proposal::VoteProposal,
};
use libra_types::crypto_proxies::Signature;
use std::sync::{Arc, RwLock};

/// A local interface into SafetyRules. Constructed in such a way that the container / caller
/// cannot distinguish this API from an actual client/server process without being exposed to
/// the actual container instead the caller can access a Box<dyn TSafetyRules>.
pub struct LocalClient<T> {
    internal: Arc<RwLock<SafetyRules<T>>>,
}

impl<T: Payload> LocalClient<T> {
    pub fn new(internal: Arc<RwLock<SafetyRules<T>>>) -> Self {
        Self { internal }
    }
}

impl<T: Payload> TSafetyRules<T> for LocalClient<T> {
    fn consensus_state(&self) -> ConsensusState {
        self.internal.read().unwrap().consensus_state()
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.internal.write().unwrap().update(qc)
    }

    fn start_new_epoch(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        self.internal.write().unwrap().start_new_epoch(qc)
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal<T>) -> Result<Vote, Error> {
        self.internal
            .write()
            .unwrap()
            .construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        self.internal.write().unwrap().sign_proposal(block_data)
    }

    fn sign_timeout(&self, timeout: &Timeout) -> Result<Signature, Error> {
        self.internal.write().unwrap().sign_timeout(timeout)
    }
}
