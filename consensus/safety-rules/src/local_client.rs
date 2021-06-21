// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error, SafetyRules, TSafetyRules};
use consensus_types::{
    block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::ed25519::Ed25519Signature;
use diem_infallible::RwLock;
use diem_types::epoch_change::EpochChangeProof;
use std::sync::Arc;
use consensus_types::experimental::commit_proposal::CommitProposal;
use consensus_types::experimental::commit_decision::CommitDecision;
use diem_types::ledger_info::LedgerInfoWithSignatures;
use diem_types::validator_verifier::ValidatorVerifier;

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
        self.internal.write().consensus_state()
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        self.internal.write().initialize(proof)
    }

    fn construct_and_sign_vote(
        &mut self,
        vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        self.internal.write().construct_and_sign_vote(vote_proposal)
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        self.internal.write().sign_proposal(block_data)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.internal.write().sign_timeout(timeout)
    }

    fn sign_commit_proposal(&mut self, ledger_info: LedgerInfoWithSignatures, verifier: &ValidatorVerifier) -> Result<Ed25519Signature, Error> {
        self.internal.write().sign_commit_proposal(ledger_info, verifier)
    }
}
