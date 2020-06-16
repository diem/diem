// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block_data::BlockData, quorum_cert::QuorumCert, timeout::Timeout, vote_proposal::VoteProposal,
};
use libra_types::epoch_change::EpochChangeProof;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum SafetyRulesInput {
    ConsensusState,
    Initialize(Box<EpochChangeProof>),
    Update(Box<QuorumCert>),
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
            SafetyRulesInput::ConsensusState => lcs::to_bytes(&self.internal.consensus_state()),
            SafetyRulesInput::Initialize(li) => lcs::to_bytes(&self.internal.initialize(&li)),
            SafetyRulesInput::Update(qc) => lcs::to_bytes(&self.internal.update(&qc)),
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
