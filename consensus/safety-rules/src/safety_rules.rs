// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    consensus_state::ConsensusState, error::Error,
    persistent_safety_storage::PersistentSafetyStorage, t_safety_rules::TSafetyRules, COUNTERS,
};
use consensus_types::{
    block::Block,
    block_data::BlockData,
    common::Author,
    quorum_cert::QuorumCert,
    timeout::Timeout,
    vote::Vote,
    vote_data::VoteData,
    vote_proposal::{MaybeSignedVoteProposal, VoteProposal},
};
use libra_crypto::{
    ed25519::{Ed25519PublicKey, Ed25519Signature},
    hash::{CryptoHash, HashValue},
    traits::Signature,
};
use libra_logger::debug;
use libra_types::{
    block_info::BlockInfo, epoch_change::EpochChangeProof, epoch_state::EpochState,
    ledger_info::LedgerInfo, validator_signer::ValidatorSigner, waypoint::Waypoint,
};
use std::cmp::Ordering;

/// SafetyRules is responsible for the safety of the consensus:
/// 1) voting rules
/// 2) commit rules
/// 3) ownership of the consensus private key
/// @TODO add a benchmark to evaluate SafetyRules
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

    /// First voting rule
    fn check_last_vote_round(&self, proposed_block: &BlockData) -> Result<(), Error> {
        let last_voted_round = self.persistent_storage.last_voted_round()?;
        if proposed_block.round() <= last_voted_round {
            debug!(
                "Vote proposal is old {} <= {}",
                proposed_block.round(),
                last_voted_round
            );
            return Err(Error::OldProposal {
                proposal_round: proposed_block.round(),
                last_voted_round: self.persistent_storage.last_voted_round()?,
            });
        }
        Ok(())
    }

    /// Second voting rule
    fn check_and_update_preferred_round(&mut self, quorum_cert: &QuorumCert) -> Result<(), Error> {
        let preferred_round = self.persistent_storage.preferred_round()?;
        let one_chain_round = quorum_cert.certified_block().round();
        let two_chain_round = quorum_cert.parent_block().round();

        if one_chain_round < preferred_round {
            debug!(
                "QC round does not match preferred round {} < {}",
                one_chain_round, preferred_round
            );
            return Err(Error::ProposalRoundLowerThanPreferredBlock { preferred_round });
        }

        // Update the preferred round
        match two_chain_round.cmp(&preferred_round) {
            Ordering::Greater => self
                .persistent_storage
                .set_preferred_round(quorum_cert.parent_block().round())?,
            Ordering::Less => debug!(
                "2-chain round {} is lower than preferred round {} but 1-chain round {} is higher.",
                two_chain_round, preferred_round, one_chain_round
            ),
            Ordering::Equal => (),
        }
        Ok(())
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
            .map_err(|e| Error::InvalidAccumulatorExtension {
                error: format!("{}", e),
            })?;
        Ok(VoteData::new(
            proposed_block.gen_block_info(
                new_tree.root_hash(),
                new_tree.version(),
                vote_proposal.next_epoch_state().cloned(),
            ),
            proposed_block.quorum_cert().certified_block().clone(),
        ))
    }

    /// Commit rule
    /// Produces a LedgerInfo that either commits a block based upon the 3-chain commit rule
    /// or an empty LedgerInfo for no commit. The 3-chain commit rule is: B0 (as well as its
    /// prefix) can be committed if there exist certified blocks B1 and B2 that satisfy:
    /// 1) B0 <- B1 <- B2 <--
    /// 2) round(B0) + 1 = round(B1), and
    /// 3) round(B1) + 1 = round(B2).
    pub fn construct_ledger_info(&self, proposed_block: &Block) -> LedgerInfo {
        let block2 = proposed_block.round();
        let block1 = proposed_block.quorum_cert().certified_block().round();
        let block0 = proposed_block.quorum_cert().parent_block().round();

        let commit = block0 + 1 == block1 && block1 + 1 == block2;
        if commit {
            LedgerInfo::new(
                proposed_block.quorum_cert().parent_block().clone(),
                HashValue::zero(),
            )
        } else {
            LedgerInfo::new(BlockInfo::empty(), HashValue::zero())
        }
    }

    /// This verifies a QC has valid signatures.
    fn verify_qc(&self, qc: &QuorumCert) -> Result<(), Error> {
        let epoch_state = self.epoch_state()?;

        qc.verify(&epoch_state.verifier)
            .map_err(|e| Error::InvalidQuorumCertificate(e.to_string()))?;
        Ok(())
    }

    /// This reconciles the key pair of a validator signer with a given validator set
    /// during epoch changes.
    /// Current impl of consensus does not expect key reconciliation to panic or fail.
    /// Instead, we signal debug messages indicating following error causes:
    ///     1. Validator not in the set
    ///     2. Validator in the set, but no matching key found in storage
    fn reconcile_key(&mut self, epoch_state: &EpochState) -> Result<(), Error> {
        let author = self.persistent_storage.author()?;
        if let Some(expected_key) = epoch_state.verifier.get_public_key(&author) {
            let curr_key = self.signer().ok().map(|s| s.public_key());
            if curr_key != Some(expected_key.clone()) {
                let consensus_key = self
                    .persistent_storage
                    .consensus_key_for_version(expected_key.clone())
                    .ok()
                    .ok_or_else(|| {
                        debug!("Validator key not found!");
                        self.validator_signer = None;
                        Error::InternalError {
                            error: "Validator key not found".into(),
                        }
                    })?;
                debug!(
                    "Reconciled pub key for signer {} [{:#?} -> {}]",
                    author, curr_key, expected_key
                );
                self.validator_signer = Some(ValidatorSigner::new(author, consensus_key));
            } else {
                debug!("Validator key matches the key in validator set.");
            }
        } else {
            debug!("The validator is not in set!");
            self.validator_signer = None;
        }
        Ok(())
    }

    /// This sets the current validator verifier and updates the epoch and round information
    /// if this is a new epoch ending ledger info. It also sets the current waypoint to this
    /// LedgerInfo.
    fn start_new_epoch(&mut self, ledger_info: &LedgerInfo) -> Result<(), Error> {
        debug!("Starting new epoch.");
        let epoch_state = ledger_info
            .next_epoch_state()
            .cloned()
            .ok_or(Error::InvalidLedgerInfo)?;
        self.reconcile_key(&epoch_state)?;

        let current_epoch = self.persistent_storage.epoch()?;

        if current_epoch < epoch_state.epoch {
            // This is ordered specifically to avoid configuration issues:
            // * First set the waypoint to lock in the minimum restarting point,
            // * set the round information,
            // * finally, set the epoch information because once the epoch is set, this `if`
            // statement cannot be re-entered.
            self.persistent_storage
                .set_waypoint(&Waypoint::new_epoch_boundary(ledger_info)?)?;
            self.persistent_storage.set_last_voted_round(0)?;
            self.persistent_storage.set_preferred_round(0)?;
            self.persistent_storage.set_epoch(epoch_state.epoch)?;
        }
        self.epoch_state = Some(epoch_state);

        Ok(())
    }

    /// This checks the epoch given against storage for consistent verification
    fn verify_epoch(&self, epoch: u64) -> Result<(), Error> {
        let expected_epoch = self.persistent_storage.epoch()?;
        if epoch != expected_epoch {
            Err(Error::IncorrectEpoch(epoch, expected_epoch))
        } else {
            Ok(())
        }
    }

    /// This checkes whether the author of one proposal is the validator signer
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
}

impl TSafetyRules for SafetyRules {
    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        Ok(ConsensusState::new(
            self.persistent_storage.epoch()?,
            self.persistent_storage.last_voted_round()?,
            self.persistent_storage.preferred_round()?,
            self.persistent_storage.waypoint()?,
            self.signer().is_ok(),
        ))
    }

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let waypoint = self.persistent_storage.waypoint()?;
        let last_li = proof
            .verify(&waypoint)
            .map_err(|e| Error::InvalidEpochChangeProof(format!("{}", e)))?;
        self.start_new_epoch(last_li.ledger_info())
    }

    fn construct_and_sign_vote(
        &mut self,
        maybe_signed_vote_proposal: &MaybeSignedVoteProposal,
    ) -> Result<Vote, Error> {
        let (vote_proposal, execution_signature) = (
            &maybe_signed_vote_proposal.vote_proposal,
            maybe_signed_vote_proposal.signature.as_ref(),
        );

        // Verify the signature from LEC.
        if let Some(public_key) = self.execution_public_key.as_ref() {
            execution_signature
                .ok_or_else(|| Error::SignatureNotFound)?
                .verify(&vote_proposal.hash(), public_key)?
        }

        let proposed_block = vote_proposal.block();
        self.verify_epoch(proposed_block.epoch())?;
        self.verify_qc(proposed_block.quorum_cert())?;
        proposed_block.validate_signature(&self.epoch_state()?.verifier)?;

        self.check_and_update_preferred_round(proposed_block.quorum_cert())?;
        self.check_last_vote_round(proposed_block.block_data())?;
        let vote_data = self.extension_check(vote_proposal)?;
        self.persistent_storage
            .set_last_voted_round(proposed_block.round())?;
        let validator_signer = self.signer()?;
        Ok(Vote::new(
            vote_data,
            validator_signer.author(),
            self.construct_ledger_info(proposed_block),
            &validator_signer,
        ))
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        debug!("Incoming proposal to sign.");
        self.verify_author(block_data.author())?;
        self.verify_epoch(block_data.epoch())?;

        self.check_last_vote_round(&block_data)?;

        let qc = block_data.quorum_cert();
        self.verify_qc(qc)?;
        self.check_and_update_preferred_round(&qc)?;

        let validator_signer = self.signer()?;
        COUNTERS.sign_proposal.inc();
        Ok(Block::new_proposal_from_block_data(
            block_data,
            &validator_signer,
        ))
    }

    /// Only sign the timeout if it is greater than or equal to the last_voted_round and ahead of
    /// the preferred_round. We may end up signing timeouts for rounds without first signing votes
    /// if we have received QCs but not proposals. Always map the last_voted_round to the last
    /// signed timeout to prevent equivocation. We can sign the last_voted_round timeout multiple
    /// times by requiring that the underlying signing scheme provides deterministic signatures.
    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        debug!("Incoming timeout message for round {}", timeout.round());
        COUNTERS.requested_sign_timeout.inc();

        self.signer()?;
        self.verify_epoch(timeout.epoch())?;

        let preferred_round = self.persistent_storage.preferred_round()?;
        if timeout.round() <= preferred_round {
            return Err(Error::BadTimeoutPreferredRound(
                timeout.round(),
                preferred_round,
            ));
        }

        let last_voted_round = self.persistent_storage.last_voted_round()?;
        if timeout.round() < last_voted_round {
            return Err(Error::BadTimeoutLastVotedRound(
                timeout.round(),
                last_voted_round,
            ));
        }
        if timeout.round() > last_voted_round {
            self.persistent_storage
                .set_last_voted_round(timeout.round())?;
        }

        let validator_signer = self.signer()?;
        let signature = timeout.sign(&validator_signer);
        COUNTERS.sign_timeout.inc();
        debug!("Successfully signed timeout message.");
        Ok(signature)
    }
}
