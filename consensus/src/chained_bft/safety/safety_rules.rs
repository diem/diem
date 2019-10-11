// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::counters;

use consensus_types::{
    block::Block,
    common::{Payload, Round},
    quorum_cert::QuorumCert,
};

use crypto::HashValue;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[cfg(test)]
#[path = "safety_rules_test.rs"]
mod safety_rules_test;

/// Vote information is returned if a proposal passes the voting rules.
/// The caller might need to persist some of the consensus state before sending out the actual
/// vote message.
/// Vote info also includes the block id that is going to be committed in case this vote gathers
/// QC.
#[derive(Debug, Eq, PartialEq)]
pub struct VoteInfo {
    /// Block id of the proposed block.
    proposal_id: HashValue,
    /// Round of the proposed block.
    proposal_round: Round,
    /// Consensus state after the voting (e.g., with the updated vote round)
    consensus_state: ConsensusState,
    /// The block that should be committed in case this vote gathers QC.
    /// If no block is committed in case the vote gathers QC, return None.
    potential_commit_id: Option<HashValue>,

    /// The id of the parent block of the proposal
    parent_block_id: HashValue,
    /// The round of the parent block of the proposal
    parent_block_round: Round,
}

impl VoteInfo {
    pub fn proposal_id(&self) -> HashValue {
        self.proposal_id
    }

    pub fn consensus_state(&self) -> &ConsensusState {
        &self.consensus_state
    }

    pub fn potential_commit_id(&self) -> Option<HashValue> {
        self.potential_commit_id
    }

    pub fn parent_block_id(&self) -> HashValue {
        self.parent_block_id
    }

    pub fn parent_block_round(&self) -> Round {
        self.parent_block_round
    }
}

#[derive(Debug, Fail, Eq, PartialEq)]
/// Different reasons for proposal rejection
pub enum ProposalReject {
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
             \tlast_vote_round = {},\n\
             \tpreferred_block_round = {}\n\
             ]",
            self.last_vote_round, self.preferred_block_round
        )
    }
}

impl ConsensusState {
    #[cfg(test)]
    pub fn new(last_vote_round: Round, preferred_block_round: Round) -> Self {
        Self {
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
            counters::LAST_VOTE_ROUND.set(last_vote_round as i64);
            Some(self.clone())
        }
    }

    /// Set the preferred block round
    fn set_preferred_block_round(&mut self, preferred_block_round: Round) {
        self.preferred_block_round = preferred_block_round;
        counters::PREFERRED_BLOCK_ROUND.set(preferred_block_round as i64);
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
}

impl SafetyRules {
    /// Constructs a new instance of SafetyRules given the BlockTree and ConsensusState.
    pub fn new(state: ConsensusState) -> Self {
        Self { state }
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
        if qc.parent_block_round() > self.state.preferred_block_round() {
            self.state
                .set_preferred_block_round(qc.parent_block_round());
        }
    }

    /// Check if a one-chain at round r+2 causes a commit at round r and return the committed
    /// block id at round r if possible
    fn commit_rule_for_certified_block(
        &self,
        block_parent_qc: &QuorumCert,
        block_round: u64,
    ) -> Option<HashValue> {
        // We're using a so-called 3-chain commit rule: B0 (as well as its prefix)
        // can be committed if there exist certified blocks B1 and B2 that satisfy:
        // 1) B0 <- B1 <- B2 <--
        // 2) round(B0) + 1 = round(B1), and
        // 3) round(B1) + 1 = round(B2).

        if block_parent_qc.parent_block_round() + 1 == block_parent_qc.certified_block_round()
            && block_parent_qc.certified_block_round() + 1 == block_round
        {
            return Some(block_parent_qc.parent_block_id());
        }
        None
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
    pub fn voting_rule<T: Payload>(
        &mut self,
        proposed_block: &Block<T>,
    ) -> Result<VoteInfo, ProposalReject> {
        if proposed_block.round() <= self.state.last_vote_round() {
            return Err(ProposalReject::OldProposal {
                proposal_round: proposed_block.round(),
                last_vote_round: self.state.last_vote_round(),
            });
        }

        let respects_preferred_block = proposed_block.quorum_cert().certified_block_round()
            >= self.state.preferred_block_round();
        if respects_preferred_block {
            self.state.set_last_vote_round(proposed_block.round());

            // If the vote for the given proposal is gathered into QC, then this QC might eventually
            // commit another block following the rules defined in
            // `commit_rule_for_certified_block()` function.
            let potential_commit_id = self.commit_rule_for_certified_block(
                proposed_block.quorum_cert(),
                proposed_block.round(),
            );

            Ok(VoteInfo {
                proposal_id: proposed_block.id(),
                proposal_round: proposed_block.round(),
                consensus_state: self.state.clone(),
                potential_commit_id,
                parent_block_id: proposed_block.quorum_cert().certified_block_id(),
                parent_block_round: proposed_block.quorum_cert().certified_block_round(),
            })
        } else {
            Err(ProposalReject::ProposalRoundLowerThenPreferredBlock {
                preferred_block_round: self.state.preferred_block_round(),
            })
        }
    }
}
