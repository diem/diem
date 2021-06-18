// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{error::Error, safety_rules::next_round, SafetyRules};
use consensus_types::{
    block::Block,
    safety_data::SafetyData,
    timeout_2chain::{TwoChainTimeout, TwoChainTimeoutCertificate},
    vote::Vote,
    vote_proposal::MaybeSignedVoteProposal,
};
use diem_crypto::{ed25519::Ed25519Signature, hash::CryptoHash, HashValue};
use diem_types::{block_info::BlockInfo, ledger_info::LedgerInfo};

/// 2-chain safety rules implementation
impl SafetyRules {
    pub(crate) fn guarded_sign_timeout_with_qc(
        &mut self,
        timeout: &TwoChainTimeout,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Ed25519Signature, Error> {
        self.signer()?;
        let mut safety_data = self.persistent_storage.safety_data()?;
        self.verify_epoch(timeout.epoch(), &safety_data)?;
        self.verify_qc(timeout.quorum_cert())?;
        if let Some(tc) = timeout_cert {
            self.verify_tc(tc)?;
        }

        self.safe_to_timeout(timeout, timeout_cert, &safety_data)?;
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

        let signature = self.sign(&timeout.signing_format())?;
        Ok(signature)
    }

    pub(crate) fn guarded_construct_and_sign_vote_two_chain(
        &mut self,
        maybe_signed_vote_proposal: &MaybeSignedVoteProposal,
        timeout_cert: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<Vote, Error> {
        // Exit early if we cannot sign
        self.signer()?;

        let vote_data = self.verify_proposal(maybe_signed_vote_proposal)?;
        if let Some(tc) = timeout_cert {
            self.verify_tc(tc)?;
        }
        let proposed_block = maybe_signed_vote_proposal.vote_proposal.block();
        let mut safety_data = self.persistent_storage.safety_data()?;

        // if already voted on this round, send back the previous vote
        // note: this needs to happen after verifying the epoch as we just check the round here
        if let Some(vote) = safety_data.last_vote.clone() {
            if vote.vote_data().proposed().round() == proposed_block.round() {
                return Ok(vote);
            }
        }

        // Two voting rules
        self.verify_and_update_last_vote_round(
            proposed_block.block_data().round(),
            &mut safety_data,
        )?;
        self.safe_to_vote(proposed_block, timeout_cert)?;

        // Record 1-chain data
        self.observe_qc(proposed_block.quorum_cert(), &mut safety_data);
        // Construct and sign vote
        let author = self.signer()?.author();
        let ledger_info = self.construct_ledger_info_2chain(proposed_block, vote_data.hash())?;
        let signature = self.sign(&ledger_info)?;
        let vote = Vote::new_with_signature(vote_data, author, ledger_info, signature);

        safety_data.last_vote = Some(vote.clone());
        self.persistent_storage.set_safety_data(safety_data)?;

        Ok(vote)
    }

    /// Core safety timeout rule for 2-chain protocol. Return success if 1 and 2 are true
    /// 1. round == timeout.qc.round + 1 || round == tc.round + 1
    /// 2. timeout.qc.round >= one_chain_round
    fn safe_to_timeout(
        &self,
        timeout: &TwoChainTimeout,
        maybe_tc: Option<&TwoChainTimeoutCertificate>,
        safety_data: &SafetyData,
    ) -> Result<(), Error> {
        let round = timeout.round();
        let qc_round = timeout.hqc_round();
        let tc_round = maybe_tc.map_or(0, |tc| tc.round());
        if (round == next_round(qc_round)? || round == next_round(tc_round)?)
            && qc_round >= safety_data.one_chain_round
        {
            Ok(())
        } else {
            Err(Error::NotSafeToTimeout(
                round,
                qc_round,
                tc_round,
                safety_data.one_chain_round,
            ))
        }
    }

    /// Core safety voting rule for 2-chain protocol. Return success if 1 or 2 is true
    /// 1. block.round == block.qc.round + 1
    /// 2. block.round == tc.round + 1 && block.qc.round >= tc.highest_hqc.round
    fn safe_to_vote(
        &self,
        block: &Block,
        maybe_tc: Option<&TwoChainTimeoutCertificate>,
    ) -> Result<(), Error> {
        let round = block.round();
        let qc_round = block.quorum_cert().certified_block().round();
        let tc_round = maybe_tc.map_or(0, |tc| tc.round());
        let hqc_round = maybe_tc.map_or(0, |tc| tc.highest_hqc_round());
        if round == next_round(qc_round)?
            || (round == next_round(tc_round)? && qc_round >= hqc_round)
        {
            Ok(())
        } else {
            Err(Error::NotSafeToVote(round, qc_round, tc_round, hqc_round))
        }
    }

    fn verify_tc(&self, tc: &TwoChainTimeoutCertificate) -> Result<(), Error> {
        let epoch_state = self.epoch_state()?;

        tc.verify(&epoch_state.verifier)
            .map_err(|e| Error::InvalidTimeoutCertificate(e.to_string()))?;
        Ok(())
    }

    /// Produces a LedgerInfo that either commits a block based upon the 2-chain
    /// commit rule or an empty LedgerInfo for no commit. The 2-chain commit rule is: B0 and its
    /// prefixes can be committed if there exist certified block B1 that satisfy:
    /// 1) B0 <- B1 <--
    /// 2) round(B0) + 1 = round(B1)
    fn construct_ledger_info_2chain(
        &self,
        proposed_block: &Block,
        consensus_data_hash: HashValue,
    ) -> Result<LedgerInfo, Error> {
        let block1 = proposed_block.round();
        let block0 = proposed_block.quorum_cert().certified_block().round();

        // verify 2-chain rule
        let commit = next_round(block0)? == block1;

        // create a ledger info
        let commit_info = if commit {
            proposed_block.quorum_cert().certified_block().clone()
        } else {
            BlockInfo::empty()
        };

        Ok(LedgerInfo::new(commit_info, consensus_data_hash))
    }
}
