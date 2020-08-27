// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    consensus_state::ConsensusState,
    counters,
    error::Error,
    logging::{self, LogEntry, LogEvent, LogField},
    persistent_safety_storage::PersistentSafetyStorage,
    t_safety_rules::TSafetyRules,
};
use consensus_types::{
    block::Block,
    block_data::BlockData,
    common::{Author, Round},
    quorum_cert::QuorumCert,
    safety_data::SafetyData,
    timeout::Timeout,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
};
use libra_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::HashValue,
    traits::Signature,
};
use libra_logger::prelude::*;
use libra_types::{
    block_info::BlockInfo, epoch_change::EpochChangeProof, epoch_state::EpochState,
    ledger_info::LedgerInfo, validator_signer::ValidatorSigner, waypoint::Waypoint,
};
use std::cmp::Ordering;

/// @TODO consider a cache of verified QCs to cut down on verification costs
pub struct SafetyRules {
    persistent_storage: PersistentSafetyStorage,
    execution_public_key: Option<Ed25519PublicKey>,
    validator_signer: Option<ValidatorSigner>,
    epoch_state: Option<EpochState>,
}

impl SafetyRules {
    /// Constructs a new instance of SafetyRules with the given persistent storage and the
    /// consensus private keys
    pub fn new(
        persistent_storage: PersistentSafetyStorage,
        verify_vote_proposal_signature: bool,
    ) -> Self {
        let execution_public_key = if verify_vote_proposal_signature {
            Some(
                persistent_storage
                    .execution_public_key()
                    .expect("Unable to retrieve execution public key"),
            )
        } else {
            None
        };
        Self {
            persistent_storage,
            execution_public_key,
            validator_signer: None,
            epoch_state: None,
        }
    }

    fn signer(&self) -> Result<&ValidatorSigner, Error> {
        self.validator_signer
            .as_ref()
            .ok_or_else(|| Error::NotInitialized("validator_signer".into()))
    }

    fn epoch_state(&self) -> Result<&EpochState, Error> {
        self.epoch_state
            .as_ref()
            .ok_or_else(|| Error::NotInitialized("epoch_state".into()))
    }

    /// Check if the executed result extends the parent result.
    fn extension_check(&self, vote_proposal: &VoteProposal) -> Result<VoteData, Error> {
        let proposed_block = vote_proposal.block();
        let new_tree = vote_proposal
            .accumulator_extension_proof()
            .verify(
                proposed_block
                    .quorum_cert()
                    .certified_block()
                    .executed_state_id(),
            )
            .map_err(|e| Error::InvalidAccumulatorExtension(e.to_string()))?;
        Ok(VoteData::new(
            proposed_block.gen_block_info(
                new_tree.root_hash(),
                new_tree.version(),
                vote_proposal.next_epoch_state().cloned(),
            ),
            proposed_block.quorum_cert().certified_block().clone(),
        ))
    }

    /// Produces a LedgerInfo that either commits a block based upon the 3-chain
    /// commit rule or an empty LedgerInfo for no commit. The 3-chain commit rule is: B0 and its
    /// prefixes can be committed if there exist certified blocks B1 and B2 that satisfy:
    /// 1) B0 <- B1 <- B2 <--
    /// 2) round(B0) + 1 = round(B1), and
    /// 3) round(B1) + 1 = round(B2).
    pub fn construct_ledger_info(&self, proposed_block: &Block) -> Result<LedgerInfo, Error> {
        let block2 = proposed_block.round();
        let block1 = proposed_block.quorum_cert().certified_block().round();
        let block0 = proposed_block.quorum_cert().parent_block().round();

        // verify 3-chain rule
        let next_round =
            |round: u64| u64::checked_add(round, 1).ok_or(Error::IncorrectRound(round));
        let commit = next_round(block0)? == block1 && next_round(block1)? == block2;

        // create a ledger info
        let ledger_info = if commit {
            LedgerInfo::new(
                proposed_block.quorum_cert().parent_block().clone(),
                HashValue::zero(),
            )
        } else {
            LedgerInfo::new(BlockInfo::empty(), HashValue::zero())
        };

        Ok(ledger_info)
    }

    /// Second voting rule
    fn verify_and_update_preferred_round(
        &mut self,
        quorum_cert: &QuorumCert,
        safety_data: &mut SafetyData,
    ) -> Result<bool, Error> {
        let preferred_round = safety_data.preferred_round;
        let one_chain_round = quorum_cert.certified_block().round();
        let two_chain_round = quorum_cert.parent_block().round();

        if one_chain_round < preferred_round {
            return Err(Error::IncorrectPreferredRound(
                one_chain_round,
                preferred_round,
            ));
        }

        let updated = match two_chain_round.cmp(&preferred_round) {
            Ordering::Greater => {
                safety_data.preferred_round = two_chain_round;
                sl_info!(
                    logging::safety_log(LogEntry::PreferredRound, LogEvent::Update)
                        .data(LogField::Message.as_str(), safety_data.preferred_round)
                );
                true
            }
            Ordering::Less => {
                trace!(
                "2-chain round {} is lower than preferred round {} but 1-chain round {} is higher.",
                two_chain_round, preferred_round, one_chain_round
            );
                false
            }
            Ordering::Equal => false,
        };
        Ok(updated)
    }

    /// This verifies whether the author of one proposal is the validator signer
    fn verify_author(&self, author: Option<Author>) -> Result<(), Error> {
        let validator_signer_author = &self.signer()?.author();
        let author = author
            .ok_or_else(|| Error::InvalidProposal("No author found in the proposal".into()))?;
        if validator_signer_author != &author {
            return Err(Error::InvalidProposal(
                "Proposal author is not validator signer!".into(),
            ));
        }
        Ok(())
    }

    /// This verifies the epoch given against storage for consistent verification
    fn verify_epoch(&self, epoch: u64, safety_data: &SafetyData) -> Result<(), Error> {
        let expected_epoch = safety_data.epoch;
        if epoch != expected_epoch {
            Err(Error::IncorrectEpoch(epoch, expected_epoch))
        } else {
            Ok(())
        }
    }

    /// First voting rule
    fn verify_and_update_last_vote_round(
        &self,
        round: Round,
        safety_data: &mut SafetyData,
    ) -> Result<(), Error> {
        let last_voted_round = safety_data.last_voted_round;
        if round > last_voted_round {
            safety_data.last_voted_round = round;
            sl_info!(
                logging::safety_log(LogEntry::LastVotedRound, LogEvent::Update)
                    .data(LogField::Message.as_str(), safety_data.last_voted_round)
            );
            return Ok(());
        }

        Err(Error::IncorrectLastVotedRound(round, last_voted_round))
    }

    /// This verifies a QC has valid signatures.
    fn verify_qc(&self, qc: &QuorumCert) -> Result<(), Error> {
        let epoch_state = self.epoch_state()?;

        qc.verify(&epoch_state.verifier)
            .map_err(|e| Error::InvalidQuorumCertificate(e.to_string()))?;
        Ok(())
    }

    // Internal functions mapped to the public interface to enable exhaustive logging and metrics

    fn guarded_consensus_state(&mut self) -> Result<ConsensusState, Error> {
        Ok(ConsensusState::new(
            self.persistent_storage.safety_data()?,
            self.persistent_storage.waypoint()?,
            self.signer().is_ok(),
        ))
    }

    fn guarded_initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let waypoint = self.persistent_storage.waypoint()?;
        let last_li = proof
            .verify(&waypoint)
            .map_err(|e| Error::InvalidEpochChangeProof(format!("{}", e)))?;
        let ledger_info = last_li.ledger_info();
        let epoch_state = ledger_info
            .next_epoch_state()
            .cloned()
            .ok_or(Error::InvalidLedgerInfo)?;

        let author = self.persistent_storage.author()?;
        if let Some(expected_key) = epoch_state.verifier.get_public_key(&author) {
            let curr_key = self.signer().ok().map(|s| s.public_key());
            if curr_key != Some(expected_key.clone()) {
                let consensus_key = self
                    .persistent_storage
                    .consensus_key_for_version(expected_key)
                    .ok()
                    .ok_or_else(|| {
                        sl_error!(logging::safety_log(
                            LogEntry::KeyReconciliation,
                            LogEvent::Error
                        )
                        .data(LogField::Message.as_str(), "Validator key not found"));

                        self.validator_signer = None;
                        Error::InternalError("Validator key not found".into())
                    })?;

                self.validator_signer = Some(ValidatorSigner::new(author, consensus_key));
            }

            sl_debug!(
                logging::safety_log(LogEntry::KeyReconciliation, LogEvent::Success)
                    .data(LogField::Message.as_str(), "in set")
            );
        } else {
            sl_debug!(
                logging::safety_log(LogEntry::KeyReconciliation, LogEvent::Success)
                    .data(LogField::Message.as_str(), "not in set")
            );
            self.validator_signer = None;
        }

        let current_epoch = self.persistent_storage.safety_data()?.epoch;

        if current_epoch < epoch_state.epoch {
            // This is ordered specifically to avoid configuration issues:
            // * First set the waypoint to lock in the minimum restarting point,
            // * set the round information,
            // * finally, set the epoch information because once the epoch is set, this `if`
            // statement cannot be re-entered.
            self.persistent_storage
                .set_waypoint(&Waypoint::new_epoch_boundary(ledger_info)?)?;
            self.persistent_storage.set_safety_data(SafetyData::new(
                epoch_state.epoch,
                0,
                0,
                None,
            ))?;
            sl_info!(logging::safety_log(LogEntry::Epoch, LogEvent::Update)
                .data(LogField::Message.as_str(), epoch_state.epoch));
        }
        self.epoch_state = Some(epoch_state);

        Ok(())
    }

    fn guarded_construct_and_sign_vote(
        &mut self,
        maybe_signed_vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        // Exit early if we cannot sign
        self.signer()?;
        let mut safety_data = self.persistent_storage.safety_data()?;

        let (vote_proposal, execution_signature) = (
            &maybe_signed_vote_proposal.vote_proposal,
            maybe_signed_vote_proposal.signature.as_ref(),
        );

        if let Some(public_key) = self.execution_public_key.as_ref() {
            execution_signature
                .ok_or_else(|| Error::VoteProposalSignatureNotFound)?
                .verify(vote_proposal, public_key)?
        }

        let proposed_block = vote_proposal.block();
        self.verify_epoch(proposed_block.epoch(), &safety_data)?;
        self.verify_qc(proposed_block.quorum_cert())?;
        proposed_block.validate_signature(&self.epoch_state()?.verifier)?;

        self.verify_and_update_preferred_round(proposed_block.quorum_cert(), &mut safety_data)?;
        // if already voted on this round, send back the previous vote.
        if let Some(vote) = safety_data.last_vote.clone() {
            if vote.vote_data().proposed().round() == proposed_block.round() {
                safety_data.last_voted_round = proposed_block.round();
                self.persistent_storage.set_safety_data(safety_data)?;
                return Ok(vote);
            }
        }
        self.verify_and_update_last_vote_round(
            proposed_block.block_data().round(),
            &mut safety_data,
        )?;

        let vote_data = self.extension_check(vote_proposal)?;

        let validator_signer = self.signer()?;
        let vote = Vote::new(
            vote_data,
            validator_signer.author(),
            self.construct_ledger_info(proposed_block)?,
            validator_signer,
        );
        safety_data.last_vote = Some(vote.clone());
        self.persistent_storage.set_safety_data(safety_data)?;

        Ok(vote)
    }

    fn guarded_sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let mut safety_data = self.persistent_storage.safety_data()?;
        self.signer()?;
        self.verify_author(block_data.author())?;
        self.verify_epoch(block_data.epoch(), &safety_data)?;
        if block_data.round() <= safety_data.last_voted_round {
            return Err(Error::InvalidProposal(format!(
                "Proposed round {} is not higher than last voted round {}",
                block_data.round(),
                safety_data.last_voted_round
            )));
        }
        self.verify_qc(block_data.quorum_cert())?;
        if self.verify_and_update_preferred_round(block_data.quorum_cert(), &mut safety_data)? {
            self.persistent_storage.set_safety_data(safety_data)?;
        }

        Ok(Block::new_proposal_from_block_data(
            block_data,
            self.signer()?,
        ))
    }

    fn guarded_sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        self.signer()?;
        let mut safety_data = self.persistent_storage.safety_data()?;
        self.verify_epoch(timeout.epoch(), &safety_data)?;

        if timeout.round() <= safety_data.preferred_round {
            return Err(Error::IncorrectPreferredRound(
                timeout.round(),
                safety_data.preferred_round,
            ));
        }

        if timeout.round() < safety_data.last_voted_round {
            return Err(Error::IncorrectLastVotedRound(
                timeout.round(),
                safety_data.last_voted_round,
            ));
        }
        if timeout.round() > safety_data.last_voted_round {
            self.verify_and_update_last_vote_round(timeout.round(), &mut safety_data)?;
            self.persistent_storage.set_safety_data(safety_data)?;
        }

        let validator_signer = self.signer()?;
        let signature = timeout.sign(&validator_signer);

        Ok(signature)
    }
}

impl TSafetyRules for SafetyRules {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let log_cb = |log: StructuredLogEntry| log;
        let cb = || self.guarded_consensus_state();
        run_and_log(cb, log_cb, LogEntry::ConsensusState)
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let log_cb = |log: StructuredLogEntry| log;
        let cb = || self.guarded_initialize(proof);
        run_and_log(cb, log_cb, LogEntry::Initialize)
    }

    fn construct_and_sign_vote(
        &mut self,
        maybe_signed_vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let round = maybe_signed_vote_proposal.vote_proposal.block().round();
        let log_cb = |log: StructuredLogEntry| log.data(LogField::Round.as_str(), round);
        let cb = || self.guarded_construct_and_sign_vote(maybe_signed_vote_proposal);
        run_and_log(cb, log_cb, LogEntry::ConstructAndSignVote)
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let round = block_data.round();
        let log_cb = |log: StructuredLogEntry| log.data(LogField::Round.as_str(), round);
        let cb = || self.guarded_sign_proposal(block_data);
        run_and_log(cb, log_cb, LogEntry::SignProposal)
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let log_cb = |log: StructuredLogEntry| log.data(LogField::Round.as_str(), timeout.round());
        let cb = || self.guarded_sign_timeout(timeout);
        run_and_log(cb, log_cb, LogEntry::SignTimeout)
    }
}

fn run_and_log<F, L, R>(callback: F, log_cb: L, log_entry: LogEntry) -> Result<R, Error>
where
    F: FnOnce() -> Result<R, Error>,
    L: Fn(StructuredLogEntry) -> StructuredLogEntry,
{
    sl_debug!(log_cb(logging::safety_log(log_entry, LogEvent::Request)));
    counters::increment_query(log_entry.as_str(), "request");
    callback()
        .map(|v| {
            sl_info!(log_cb(logging::safety_log(log_entry, LogEvent::Success)));
            counters::increment_query(log_entry.as_str(), "success");
            v
        })
        .map_err(|err| {
            sl_error!(log_cb(logging::safety_log(log_entry, LogEvent::Error))
                .data(LogField::Message.as_str(), &err));
            counters::increment_query(log_entry.as_str(), "error");
            err
        })
}
