// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::Round;
use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, common::Payload, quorum_cert::QuorumCert,
    timeout::Timeout, vote::Vote, vote_proposal::VoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_types::epoch_change::EpochChangeProof;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
pub enum SafetyRulesInput<T> {
    ConsensusState,
    Initialize(Box<EpochChangeProof>),
    Update(Box<QuorumCert>),
    #[serde(bound = "T: Payload")]
    ConstructAndSignVote(Box<VoteProposal<T>>),
    #[serde(bound = "T: Payload")]
    ConstructAndSignStrongVote(Box<VoteProposal<T>>, Round),
    #[serde(bound = "T: Payload")]
    SignProposal(Box<BlockData<T>>),
    SignTimeout(Box<Timeout>),
}

pub struct SerializerService<T> {
    internal: SafetyRules<T>,
}

impl<T: Payload> SerializerService<T> {
    pub fn new(internal: SafetyRules<T>) -> Self {
        Self { internal }
    }

    pub fn handle_message(&mut self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;

        let output = match input {
            SafetyRulesInput::ConsensusState => lcs::to_bytes(&self.internal.consensus_state()),
            SafetyRulesInput::Initialize(li) => lcs::to_bytes(&self.internal.initialize(&li)),
            SafetyRulesInput::Update(qc) => lcs::to_bytes(&self.internal.update(&qc)),
            SafetyRulesInput::ConstructAndSignVote(vote_proposal) => {
                lcs::to_bytes(&self.internal.construct_and_sign_vote(&vote_proposal))
            }
            SafetyRulesInput::ConstructAndSignStrongVote(vote_proposal, marker) => {
                lcs::to_bytes(&self.internal.construct_and_sign_strong_vote(&vote_proposal, marker))
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

pub struct SerializerClient<T> {
    service: Box<dyn TSerializerClient<T>>,
}

impl<T: Payload> SerializerClient<T> {
    pub fn new(serializer_service: Arc<RwLock<SerializerService<T>>>) -> Self {
        let service = Box::new(LocalService { serializer_service });
        Self { service }
    }

    pub fn new_client(service: Box<dyn TSerializerClient<T>>) -> Self {
        Self { service }
    }

    fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl<T: Payload> TSafetyRules<T> for SerializerClient<T> {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        lcs::from_bytes(&response)?
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Initialize(Box::new(proof.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Update(Box::new(qc.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal<T>) -> Result<Vote, Error> {
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_strong_vote(&mut self, vote_proposal: &VoteProposal<T>, marker: Round) -> Result<Vote, Error> {
        let response = self.request(SafetyRulesInput::ConstructAndSignStrongVote(Box::new(
            vote_proposal.clone(),
        ), marker))?;
        lcs::from_bytes(&response)?
    }

    fn sign_proposal(&mut self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        let response = self.request(SafetyRulesInput::SignProposal(Box::new(block_data)))?;
        lcs::from_bytes(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let response = self.request(SafetyRulesInput::SignTimeout(Box::new(timeout.clone())))?;
        lcs::from_bytes(&response)?
    }
}

pub trait TSerializerClient<T>: Send + Sync {
    fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error>;
}

struct LocalService<T> {
    pub serializer_service: Arc<RwLock<SerializerService<T>>>,
}

impl<T: Payload> TSerializerClient<T> for LocalService<T> {
    fn request(&mut self, input: SafetyRulesInput<T>) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.serializer_service
            .write()
            .unwrap()
            .handle_message(input_message)
    }
}
