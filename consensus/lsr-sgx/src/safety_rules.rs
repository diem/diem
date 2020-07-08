use libra_types::{
    validator_signer::ValidatorSigner,
    epoch_change::EpochChangeProof,
};
use consensus_types::{
    block_data::BlockData,
    timeout::Timeout,
    vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};

use crate::consensus_state::ConsensusState;

pub struct SafetyRules {
    validator_signer: ValidatorSigner,
}

impl SafetyRules {
    pub fn new() -> Self {
        Self {
            validator_signer: ValidatorSigner::from_int(1),
        }
    }

    pub fn initialize(&mut self, proof: &EpochChangeProof) {
    }

    pub fn construct_and_sign_proposal(&mut self, vote_proposal: &MaybeSignedVoteProposal) {
    }

    pub fn sign_proposal(&mut self, block_data: BlockData) {
    }

    pub fn sign_timeout(&mut self, timeout: &Timeout) {
    }

    pub fn consensus_state(&mut self) -> Option<ConsensusState> {
        eprintln!("Handling req:consensus_state!");
        None
    }
}

