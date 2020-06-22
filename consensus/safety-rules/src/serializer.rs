// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote, vote_proposal::VoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_logger::warn;
use libra_types::epoch_change::EpochChangeProof;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
pub enum SafetyRulesInput {
    ConsensusState,
    Initialize(Box<EpochChangeProof>),
    ConstructAndSignVote(Box<VoteProposal>),
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
            SafetyRulesInput::ConsensusState => {
                log_and_serialize(self.internal.consensus_state(), "ConsensusState")
            }
            SafetyRulesInput::Initialize(li) => {
                log_and_serialize(self.internal.initialize(&li), "Initialize")
            }
            SafetyRulesInput::ConstructAndSignVote(vote_proposal) => log_and_serialize(
                self.internal.construct_and_sign_vote(&vote_proposal),
                "ConstructAndSignVote",
            ),
            SafetyRulesInput::SignProposal(block_data) => {
                log_and_serialize(self.internal.sign_proposal(*block_data), "SignProposal")
            }
            SafetyRulesInput::SignTimeout(timeout) => {
                log_and_serialize(self.internal.sign_timeout(&timeout), "SignTimeout")
            }
        };

        Ok(output?)
    }
}

fn log_and_serialize<T: Serialize>(
    response: Result<T, Error>,
    from: &'static str,
) -> Result<Vec<u8>, lcs::Error> {
    if let Err(e) = &response {
        warn!("[SafetyRules] {} failed: {}", from, e);
    }
    lcs::to_bytes(&response)
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
        let response = self.request(SafetyRulesInput::ConsensusState)?;
        lcs::from_bytes(&response)?
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let response = self.request(SafetyRulesInput::Initialize(Box::new(proof.clone())))?;
        lcs::from_bytes(&response)?
    }

    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal) -> Result<Vote, Error> {
        let response = self.request(SafetyRulesInput::ConstructAndSignVote(Box::new(
            vote_proposal.clone(),
        )))?;
        lcs::from_bytes(&response)?
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let response = self.request(SafetyRulesInput::SignProposal(Box::new(block_data)))?;
        lcs::from_bytes(&response)?
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
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
            .unwrap()
            .handle_message(input_message)
    }
}
