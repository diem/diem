// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::{
    block::Block,
    block_data::BlockData,
    block_info::BlockInfo,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::VoteProposal,
};
use failure::Fail;
use libra_crypto::hash::HashValue;
use libra_types::{
    crypto_proxies::{Signature, ValidatorSigner},
    ledger_info::LedgerInfo,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(test)]
#[path = "safety_rules_test.rs"]
mod safety_rules_test;

#[derive(Debug, Fail, Eq, PartialEq)]
/// Different reasons for proposal rejection
pub enum Error {
    /// This proposal's round is less than round of preferred block.
    /// Returns the id of the preferred block.
    #[fail(
        display = "Proposal's round is lower than round of preferred block at round {:?}",
        preferred_block_round
    )]
    ProposalRoundLowerThenPreferredBlock { preferred_block_round: Round },

    /// This proposal is too old - return last_vote_round
    #[fail(
        display = "Proposal at round {:?} is not newer than the last vote round {:?}",
        proposal_round, last_vote_round
    )]
    OldProposal {
        last_vote_round: Round,
        proposal_round: Round,
    },
}

/// The state required to guarantee safety of the protocol.
/// We need to specify the specific state to be persisted for the recovery of the protocol.
/// (e.g., last vote round and preferred block round).
#[derive(Serialize, Default, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ConsensusState {
    epoch: u64,
    last_vote_round: Round,

    // A "preferred block" is the two-chain head with the highest block round.
    // We're using the `head` / `tail` terminology for describing the chains of QCs for describing
    // `head` <-- <block>* <-- `tail` chains.

    // A new proposal is voted for only if it's previous block's round is higher or equal to
    // the preferred_block_round.
    // 1) QC chains follow direct parenthood relations because a node must carry a QC to its
    // parent. 2) The "max round" rule applies to the HEAD of the chain and not its TAIL (one
    // does not necessarily apply the other).
    preferred_block_round: Round,
}

impl Display for ConsensusState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ConsensusState: [\n\
             \tepoch = {},
             \tlast_vote_round = {},\n\
             \tpreferred_block_round = {}\n\
             ]",
            self.epoch, self.last_vote_round, self.preferred_block_round
        )
    }
}

impl ConsensusState {
    pub fn new(epoch: u64, last_vote_round: Round, preferred_block_round: Round) -> Self {
        Self {
            epoch,
            last_vote_round,
            preferred_block_round,
        }
    }

    /// Returns the last round that was voted on
    pub fn last_vote_round(&self) -> Round {
        self.last_vote_round
    }

    /// Returns the preferred block round
    pub fn preferred_block_round(&self) -> Round {
        self.preferred_block_round
    }

    /// Set the last vote round that ensures safety.  If the last vote round increases, return
    /// the new consensus state based with the updated last vote round.  Otherwise, return None.
    fn set_last_vote_round(&mut self, last_vote_round: Round) -> Option<ConsensusState> {
        if last_vote_round <= self.last_vote_round {
            None
        } else {
            self.last_vote_round = last_vote_round;
            Some(self.clone())
        }
    }

    /// Set the preferred block round
    fn set_preferred_block_round(&mut self, preferred_block_round: Round) {
        self.preferred_block_round = preferred_block_round;
    }
}

/// SafetyRules is responsible for two things that are critical for the safety of the consensus:
/// 1) voting rules,
/// 2) commit rules.
/// SafetyRules is NOT THREAD SAFE (should be protected outside via e.g., RwLock).
/// The commit decisions are returned to the caller as result of learning about a new QuorumCert.
pub struct SafetyRules {
    // Keeps the state.
    state: ConsensusState,
    validator_signer: ValidatorSigner,
}

impl SafetyRules {
    /// Constructs a new instance of SafetyRules given the BlockTree and ConsensusState.
    pub fn new(state: ConsensusState, validator_signer: ValidatorSigner) -> Self {
        Self {
            state,
            validator_signer,
        }
    }

    pub fn signer(&self) -> &ValidatorSigner {
        &self.validator_signer
    }

    /// Learn about a new quorum certificate. Several things can happen as a result of that:
    /// 1) update the preferred block to a higher value.
    /// 2) commit some blocks.
    /// In case of commits the last committed block is returned.
    /// Requires that all the ancestors of the block are available for at least up to the last
    /// committed block, might panic otherwise.
    /// The update function is invoked whenever a system learns about a potentially high QC.
    pub fn update(&mut self, qc: &QuorumCert) {
        // Preferred block rule: choose the highest 2-chain head.
        if qc.parent_block().round() > self.state.preferred_block_round() {
            self.state
                .set_preferred_block_round(qc.parent_block().round());
        }
    }

    fn ledger_info_from_block_info(block_info: &BlockInfo) -> LedgerInfo {
        LedgerInfo::new(
            block_info.version(),
            block_info.executed_state_id(),
            HashValue::zero(),
            block_info.id(),
            block_info.epoch(),
            block_info.timestamp_usecs(),
            block_info.next_validator_set().cloned(),
        )
    }

    fn empty_ledger_info() -> LedgerInfo {
        LedgerInfo::new(
            0,
            HashValue::zero(),
            HashValue::zero(),
            HashValue::zero(),
            0,
            0,
            None,
        )
    }

    /// Produces a LedgerInfo that either commits a block based upon the 3-chain commit rule
    /// or an empty LedgerInfo for no commit. The 3-chain commit rule is: B0 (as well as its
    /// prefix) can be committed if there exist certified blocks B1 and B2 that satisfy:
    /// 1) B0 <- B1 <- B2 <--
    /// 2) round(B0) + 1 = round(B1), and
    /// 3) round(B1) + 1 = round(B2).
    fn construct_ledger_info<T: Payload>(&self, proposed_block: &Block<T>) -> LedgerInfo {
        let block2 = proposed_block.round();
        let block1 = proposed_block.quorum_cert().certified_block().round();
        let block0 = proposed_block.quorum_cert().parent_block().round();

        let commit = block0 + 1 == block1 && block1 + 1 == block2;
        match commit {
            true => Self::ledger_info_from_block_info(proposed_block.quorum_cert().parent_block()),
            false => Self::empty_ledger_info(),
        }
    }

    /// Clones the up-to-date state of consensus (for monitoring / debugging purposes)
    pub fn consensus_state(&self) -> ConsensusState {
        self.state.clone()
    }

    /// Attempts to vote for a given proposal following the voting rules.
    /// The returned value is then going to be used for either sending the vote or doing nothing.
    /// In case of a vote a cloned consensus state is returned (to be persisted before the vote is
    /// sent).
    /// Requires that all the ancestors of the block are available for at least up to the last
    /// committed block, might panic otherwise.
    pub fn construct_and_sign_vote<T: Payload>(
        &mut self,
        vote_proposal: &VoteProposal<T>,
    ) -> Result<Vote, Error> {
        let proposed_block = vote_proposal.block();

        if proposed_block.round() <= self.state.last_vote_round() {
            return Err(Error::OldProposal {
                proposal_round: proposed_block.round(),
                last_vote_round: self.state.last_vote_round(),
            });
        }

        let respects_preferred_block = proposed_block.quorum_cert().certified_block().round()
            >= self.state.preferred_block_round();

        if !respects_preferred_block {
            return Err(Error::ProposalRoundLowerThenPreferredBlock {
                preferred_block_round: self.state.preferred_block_round(),
            });
        }

        self.state.set_last_vote_round(proposed_block.round());

        Ok(Vote::new(
            VoteData::new(
                BlockInfo::from_block(
                    proposed_block,
                    vote_proposal.executed_state_id(),
                    vote_proposal.version(),
                    vote_proposal.next_validator_set().cloned(),
                ),
                proposed_block.quorum_cert().certified_block().clone(),
            ),
            self.validator_signer.author(),
            self.construct_ledger_info(proposed_block),
            &self.validator_signer,
        ))
    }

    pub fn sign_proposal<T: Payload>(&self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        Ok(Block::new_proposal_from_block_data(
            block_data,
            &self.validator_signer,
        ))
    }

    /// As the holder of the private key, SafetyRules also signs what is effectively a
    /// timeout message. This returns the signature for that timeout message.
    pub fn sign_timeout(&self, timeout: &Timeout) -> Result<Signature, Error> {
        Ok(timeout.sign(&self.validator_signer))
    }
}
