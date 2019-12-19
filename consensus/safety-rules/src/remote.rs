// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, common::Payload, quorum_cert::QuorumCert,
    timeout::Timeout, vote::Vote, vote_proposal::VoteProposal,
};
use libra_types::crypto_proxies::Signature;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
pub enum SafetyRulesInput<T> {
    ConsensusState,
    Update(Box<QuorumCert>),
    StartNewEpoch(Box<QuorumCert>),
    #[serde(bound = "T: Payload")]
    ConstructAndSignVote(Box<VoteProposal<T>>),
    #[serde(bound = "T: Payload")]
    SignProposal(Box<BlockData<T>>),
    SignTimeout(Box<Timeout>),
}

pub struct RemoteService<T> {
    internal: SafetyRules<T>,
}

impl<T: Payload> RemoteService<T> {
    pub fn new(internal: SafetyRules<T>) -> Self {
        Self { internal }
    }

    pub fn handle_message(&mut self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;

        let output = match input {
            SafetyRulesInput::ConsensusState => lcs::to_bytes(&self.internal.consensus_state()),
            SafetyRulesInput::Update(qc) => lcs::to_bytes(&self.internal.update(&qc)),
            SafetyRulesInput::StartNewEpoch(qc) => {
                lcs::to_bytes(&self.internal.start_new_epoch(&qc))
            }
            SafetyRulesInput::ConstructAndSignVote(vote_proposal) => {
                lcs::to_bytes(&self.internal.construct_and_sign_vote(&vote_proposal))
            }
            SafetyRulesInput::SignProposal(block_data) => {
                lcs::to_bytes(&self.internal.sign_proposal(*block_data))
            }
            SafetyRulesInput::SignTimeout(timeout) => {
                lcs::to_bytes(&self.internal.sign_timeout(&timeout))
            }
        };

        Ok(output?)
    }
}

pub struct RemoteClient<T> {
    service: Arc<RwLock<RemoteService<T>>>,
}

impl<T: Payload> RemoteClient<T> {
    pub fn new(service: Arc<RwLock<RemoteService<T>>>) -> Self {
        Self { service }
    }

    pub fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.service.write().unwrap().handle_message(input_message)
    }
}

impl<T: Payload> TSafetyRules<T> for RemoteClient<T> {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        lcs::from_bytes(&response)?
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Update(Box::new(qc.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn start_new_epoch(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::StartNewEpoch(Box::new(qc.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal<T>) -> Result<Vote, Error> {
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        lcs::from_bytes(&response)?
    }

    fn sign_proposal(&mut self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        let response = self.request(SafetyRulesInput::SignProposal(Box::new(block_data)))?;
        lcs::from_bytes(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Signature, Error> {
        let response = self.request(SafetyRulesInput::SignTimeout(Box::new(timeout.clone())))?;
        lcs::from_bytes(&response)?
    }
}
