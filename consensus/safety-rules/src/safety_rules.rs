// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::persistent_storage::PersistentStorage;
use consensus_types::{
    block::Block,
    block_data::BlockData,
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
    block_info::BlockInfo,
    crypto_proxies::{Signature, ValidatorSigner},
    ledger_info::LedgerInfo,
};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;

#[derive(Debug, Fail, PartialEq)]
/// Different reasons for proposal rejection
pub enum Error {
    #[fail(
        display = "Unable to verify that the new tree extneds the parent: {:?}",
        error
    )]
    InvalidAccumulatorExtension { error: String },

    /// This proposal's round is less than round of preferred block.
    /// Returns the id of the preferred block.
    #[fail(
        display = "Proposal's round is lower than round of preferred block at round {:?}",
        preferred_round
    )]
    ProposalRoundLowerThenPreferredBlock { preferred_round: Round },

    /// This proposal is too old - return last_voted_round
    #[fail(
        display = "Proposal at round {:?} is not newer than the last vote round {:?}",
        proposal_round, last_voted_round
    )]
    OldProposal {
        last_voted_round: Round,
        proposal_round: Round,
    },
}

/// Public representation of the internal state of SafetyRules for monitoring / debugging purposes.
/// This does not include sensitive data like private keys.
/// @TODO add hash of ledger info (waypoint)
#[derive(Serialize, Default, Deserialize, Debug, Eq, PartialEq, Clone)]
pub struct ConsensusState {
    epoch: u64,
    last_voted_round: Round,
    // A "preferred block" is the two-chain head with the highest block round. The expectation is
    // that a new proposal's parent is higher or equal to the preferred_round.
    preferred_round: Round,
}

impl Display for ConsensusState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "ConsensusState: [\n\
             \tepoch = {},
             \tlast_voted_round = {},\n\
             \tpreferred_round = {}\n\
             ]",
            self.epoch, self.last_voted_round, self.preferred_round
        )
    }
}

impl ConsensusState {
    pub fn new(epoch: u64, last_voted_round: Round, preferred_round: Round) -> Self {
        Self {
            epoch,
            last_voted_round,
            preferred_round,
        }
    }

    /// Returns the current epoch
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Returns the last round that was voted on
    pub fn last_voted_round(&self) -> Round {
        self.last_voted_round
    }

    /// Returns the preferred block round
    pub fn preferred_round(&self) -> Round {
        self.preferred_round
    }
}

/// SafetyRules is responsible for the safety of the consensus:
/// 1) voting rules
/// 2) commit rules
/// 3) ownership of the consensus private key
/// @TODO add a benchmark to evaluate SafetyRules
/// @TODO consider a cache of verified QCs to cut down on verification costs
/// @TODO bootstrap with a hash of a ledger info (waypoint) that includes a validator set
/// @TODO update storage with hash of ledger info (waypoint) during epoch changes (includes a new validator
/// set)
pub struct SafetyRules {
    persistent_storage: Box<dyn PersistentStorage>,
    validator_signer: Arc<ValidatorSigner>,
}

impl SafetyRules {
    /// Constructs a new instance of SafetyRules with the given persistent storage and the
    /// consensus private keys
    /// @TODO replace this with an API that takes in a SafetyRulesConfig
    /// @TODO load private key from persistent store
    pub fn new(
        persistent_storage: Box<dyn PersistentStorage>,
        validator_signer: Arc<ValidatorSigner>,
    ) -> Self {
        Self {
            persistent_storage,
            validator_signer,
        }
    }

    pub fn signer(&self) -> &ValidatorSigner {
        &self.validator_signer
    }

    /// Learn about a new quorum certificate. In normal state, this updates the preferred round,
    /// if the parent is greater than our current preferred round. This can also trigger upgrading
    /// to a new epoch.
    /// @TODO verify signatures of the QC
    /// @TODO improving signaling by stating reaction to passed in QC:
    ///     QC has older preferred round,
    ///     signatures are incorrect,
    ///     epoch is unexpected
    ///     updating to new preferred round
    /// @TODO update epoch with validator set
    /// @TODO if public key does not match private key in validator set, access persistent storage
    /// to identify new key
    pub fn update(&mut self, qc: &QuorumCert) {
        if qc.ledger_info().ledger_info().epoch() > self.persistent_storage.epoch() {
            self.persistent_storage
                .set_epoch(qc.ledger_info().ledger_info().epoch());
            self.persistent_storage.set_last_voted_round(0);
            self.persistent_storage.set_preferred_round(0);
        }

        if qc.parent_block().round() > self.persistent_storage.preferred_round() {
            self.persistent_storage
                .set_preferred_round(qc.parent_block().round());
        }
    }

    /// Produces a LedgerInfo that either commits a block based upon the 3-chain commit rule
    /// or an empty LedgerInfo for no commit. The 3-chain commit rule is: B0 (as well as its
    /// prefix) can be committed if there exist certified blocks B1 and B2 that satisfy:
    /// 1) B0 <- B1 <- B2 <--
    /// 2) round(B0) + 1 = round(B1), and
    /// 3) round(B1) + 1 = round(B2).
    pub fn construct_ledger_info<T: Payload>(&self, proposed_block: &Block<T>) -> LedgerInfo {
        let block2 = proposed_block.round();
        let block1 = proposed_block.quorum_cert().certified_block().round();
        let block0 = proposed_block.quorum_cert().parent_block().round();

        let commit = block0 + 1 == block1 && block1 + 1 == block2;
        match commit {
            true => LedgerInfo::new(
                proposed_block.quorum_cert().parent_block().clone(),
                HashValue::zero(),
            ),
            false => LedgerInfo::new(BlockInfo::empty(), HashValue::zero()),
        }
    }

    /// Provides the internal state of SafetyRules for monitoring / debugging purposes. This does
    /// not include sensitive data like private keys.
    pub fn consensus_state(&self) -> ConsensusState {
        ConsensusState {
            epoch: self.persistent_storage.epoch(),
            last_voted_round: self.persistent_storage.last_voted_round(),
            preferred_round: self.persistent_storage.preferred_round(),
        }
    }

    /// Attempts to vote for a given proposal following the voting rules.
    /// @TODO verify signature on vote proposal
    /// @TODO verify QC correctness
    /// @TODO verify epoch on vote proposal
    pub fn construct_and_sign_vote<T: Payload>(
        &mut self,
        vote_proposal: &VoteProposal<T>,
    ) -> Result<Vote, Error> {
        let proposed_block = vote_proposal.block();

        if proposed_block.round() <= self.persistent_storage.last_voted_round() {
            return Err(Error::OldProposal {
                proposal_round: proposed_block.round(),
                last_voted_round: self.persistent_storage.last_voted_round(),
            });
        }

        let respects_preferred_block = proposed_block.quorum_cert().certified_block().round()
            >= self.persistent_storage.preferred_round();

        if !respects_preferred_block {
            return Err(Error::ProposalRoundLowerThenPreferredBlock {
                preferred_round: self.persistent_storage.preferred_round(),
            });
        }

        let new_tree = vote_proposal
            .accumulator_extension_proof()
            .verify(
                proposed_block
                    .quorum_cert()
                    .certified_block()
                    .executed_state_id(),
            )
            .map_err(|e| Error::InvalidAccumulatorExtension {
                error: format!("{}", e),
            })?;

        self.persistent_storage
            .set_last_voted_round(proposed_block.round());

        Ok(Vote::new(
            VoteData::new(
                proposed_block.gen_block_info(
                    new_tree.root_hash(),
                    new_tree.version(),
                    vote_proposal.next_validator_set().cloned(),
                ),
                proposed_block.quorum_cert().certified_block().clone(),
            ),
            self.validator_signer.author(),
            self.construct_ledger_info(proposed_block),
            &self.validator_signer,
        ))
    }

    /// As the holder of the private key, SafetyRules also signs proposals or blocks.
    /// A Block is a signed BlockData along with some additional metadata.
    /// @TODO only sign blocks that are later than last_voted_round and match the current epoch
    /// @TODO verify QC correctness
    /// @TODO verify QC matches preferred round
    pub fn sign_proposal<T: Payload>(&self, block_data: BlockData<T>) -> Result<Block<T>, Error> {
        Ok(Block::new_proposal_from_block_data(
            block_data,
            &self.validator_signer,
        ))
    }

    /// As the holder of the private key, SafetyRules also signs what is effectively a
    /// timeout message. This returns the signature for that timeout message.
    /// @TODO only sign a timeout if it matches last_voted_round or last_voted_round + 1
    /// @TODO update last_voted_round
    pub fn sign_timeout(&self, timeout: &Timeout) -> Result<Signature, Error> {
        Ok(timeout.sign(&self.validator_signer))
    }
}
