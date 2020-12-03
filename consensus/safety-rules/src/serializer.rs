// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{counters, logging::LogEntry, ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_infallible::RwLock;
use diem_types::epoch_change::EpochChangeProof;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SafetyRulesInput {
    ConsensusState,
    Initialize(Box<EpochChangeProof>),
    ConstructAndSignVote(Box<MaybeSignedVoteProposal>),
    SignProposal(Box<BlockData>),
    SignTimeout(Box<Timeout>),
}

pub struct SerializerService {
    internal: SafetyRules,
}

impl SerializerService {
    pub fn new(internal: SafetyRules) -> Self {
        Self { internal }
    }

    pub fn handle_message(&mut self, input_message: Vec<u8>) -> Result<Vec<u8>, Error> {
        let input = lcs::from_bytes(&input_message)?;

        let output = match input {
            SafetyRulesInput::ConsensusState => lcs::to_bytes(&self.internal.consensus_state()),
            SafetyRulesInput::Initialize(li) => lcs::to_bytes(&self.internal.initialize(&li)),
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

pub struct SerializerClient {
    service: Box<dyn TSerializerClient>,
}

impl SerializerClient {
    pub fn new(serializer_service: Arc<RwLock<SerializerService>>) -> Self {
        let service = Box::new(LocalService { serializer_service });
        Self { service }
    }

    pub fn new_client(service: Box<dyn TSerializerClient>) -> Self {
        Self { service }
    }

    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        self.service.request(input)
    }
}

impl TSafetyRules for SerializerClient {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let _timer = counters::start_timer("external", LogEntry::ConsensusState.as_str());
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        lcs::from_bytes(&response)?
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let _timer = counters::start_timer("external", LogEntry::Initialize.as_str());
        let response = self.request(SafetyRulesInput::Initialize(Box::new(proof.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let _timer = counters::start_timer("external", LogEntry::ConstructAndSignVote.as_str());
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        lcs::from_bytes(&response)?
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignProposal.as_str());
        let response = self.request(SafetyRulesInput::SignProposal(Box::new(block_data)))?;
        lcs::from_bytes(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let _timer = counters::start_timer("external", LogEntry::SignTimeout.as_str());
        let response = self.request(SafetyRulesInput::SignTimeout(Box::new(timeout.clone())))?;
        lcs::from_bytes(&response)?
    }
}

pub trait TSerializerClient: Send + Sync {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error>;
}

struct LocalService {
    pub serializer_service: Arc<RwLock<SerializerService>>,
}

impl TSerializerClient for LocalService {
    fn request(&mut self, input: SafetyRulesInput) -> Result<Vec<u8>, Error> {
        let input_message = lcs::to_bytes(&input)?;
        self.serializer_service
            .write()
            .handle_message(input_message)
    }
}
