// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{ConsensusState, Error};
use consensus_types::{
    block::Block, block_data::BlockData, quorum_cert::QuorumCert, timeout::Timeout, vote::Vote,
    vote_proposal::VoteProposal,
};
use libra_types::crypto_proxies::Signature;

/// Interface for SafetyRules
pub trait TSafetyRules<T> {
    /// Provides the internal state of SafetyRules for monitoring / debugging purposes. This does
    /// not include sensitive data like private keys.
    fn consensus_state(&self) -> ConsensusState;

    /// Learn about a new quorum certificate. In normal state, this updates the preferred round,
    /// if the parent is greater than our current preferred round.
    fn update(&mut self, qc: &QuorumCert) -> Result<(), Error>;

    /// Notify the safety rules about the new epoch start.
    fn start_new_epoch(&mut self, qc: &QuorumCert) -> Result<(), Error>;

    /// Attempts to vote for a given proposal following the voting rules.
    fn construct_and_sign_vote(&mut self, vote_proposal: &VoteProposal<T>) -> Result<Vote, Error>;

    /// As the holder of the private key, SafetyRules also signs proposals or blocks.
    /// A Block is a signed BlockData along with some additional metadata.
    fn sign_proposal(&self, block_data: BlockData<T>) -> Result<Block<T>, Error>;

    /// As the holder of the private key, SafetyRules also signs what is effectively a
    /// timeout message. This returns the signature for that timeout message.
    fn sign_timeout(&self, timeout: &Timeout) -> Result<Signature, Error>;
}
