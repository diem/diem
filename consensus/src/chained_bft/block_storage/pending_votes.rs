// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::VoteReceptionResult;
use consensus_types::{
    common::{Author, Round},
    quorum_cert::QuorumCert,
    timeout_certificate::TimeoutCertificate,
    vote::Vote,
};
use libra_crypto::{hash::CryptoHash, HashValue};
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    validator_verifier::{ValidatorVerifier, VerifyError},
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[cfg(test)]
#[path = "pending_votes_test.rs"]
mod pending_votes_test;

struct LastVoteInfo {
    li_digest: HashValue,
    round: Round,
    is_timeout: bool, // true if a vote includes a round signature that can be aggregated to TC
}

/// Last pending votes of the authors. Should be cleared upon reconfiguration.
pub struct PendingVotes {
    /// `li_digest_to_votes` might keep multiple LedgerInfos per proposed block in order
    /// to tolerate non-determinism in execution: given a proposal, a QuorumCertificate is going
    /// to be collected only for all the votes that carry identical LedgerInfo.
    /// LedgerInfo digest covers the potential commit ids, as well as the vote information
    /// (including the 3-chain of a voted proposal).
    /// Thus, the structure of `li_digest_to_votes` is as follows:
    /// HashMap<ledger_info_digest, LedgerInfoWithSignatures>
    li_digest_to_votes: HashMap<HashValue, LedgerInfoWithSignatures>,
    /// Tracks all the signatures of the votes for the given round. In case we succeed to
    /// aggregate 2f+1 signatures for the same round a TimeoutCertificate is formed.
    /// Note that QuorumCert has higher priority than TimeoutCertificate (in case 2f+1 votes are
    /// gathered for the same ledger info we are going to generate QuorumCert and not the
    /// TimeoutCertificate).
    round_to_tc: HashMap<Round, TimeoutCertificate>,
    /// Map of Author to last vote info. Any pending vote from Author is cleaned up
    /// whenever a new vote is added by same Author
    author_to_last_voted_info: HashMap<Author, LastVoteInfo>,
}

impl PendingVotes {
    pub fn new() -> Self {
        PendingVotes {
            li_digest_to_votes: HashMap::new(),
            round_to_tc: HashMap::new(),
            author_to_last_voted_info: HashMap::new(),
        }
    }

    /// Insert a vote and if the vote is valid, return a QuorumCertificate preferentially over a
    /// TimeoutCertificate if either can can be formed
    pub fn insert_vote(
        &mut self,
        vote: &Vote,
        validator_verifier: &ValidatorVerifier,
    ) -> VoteReceptionResult {
        if let Err(e) = self.replace_prev_vote(vote) {
            return e;
        }
        let vote_aggr_res = self.aggregate_qc(vote, validator_verifier);
        if let VoteReceptionResult::NewQuorumCertificate(_) = vote_aggr_res {
            return vote_aggr_res;
        }
        match self.aggregate_tc(vote, &validator_verifier) {
            Some(VoteReceptionResult::NewTimeoutCertificate(tc)) => {
                VoteReceptionResult::NewTimeoutCertificate(tc)
            }
            _ => vote_aggr_res,
        }
    }

    /// Check whether the newly inserted vote completes a QC
    fn aggregate_qc(
        &mut self,
        vote: &Vote,
        validator_verifier: &ValidatorVerifier,
    ) -> VoteReceptionResult {
        // Note that the digest covers the ledger info information, which is also indirectly
        // covering vote data hash (in its `consensus_data_hash` field).
        let li_digest = vote.ledger_info().hash();
        let li_with_sig = self.li_digest_to_votes.entry(li_digest).or_insert_with(|| {
            LedgerInfoWithSignatures::new(vote.ledger_info().clone(), BTreeMap::new())
        });
        li_with_sig.add_signature(vote.author(), vote.signature().clone());

        match validator_verifier.check_voting_power(li_with_sig.signatures().keys()) {
            Ok(_) => VoteReceptionResult::NewQuorumCertificate(Arc::new(QuorumCert::new(
                vote.vote_data().clone(),
                li_with_sig.clone(),
            ))),
            Err(VerifyError::TooLittleVotingPower { voting_power, .. }) => {
                VoteReceptionResult::VoteAdded(voting_power)
            }
            Err(error) => {
                error!("MUST_FIX: vote received could not be added: {}", error);
                VoteReceptionResult::ErrorAddingVote(error)
            }
        }
    }

    /// In case a timeout certificate is formed (there are 2f+1 votes in the same round) return the
    /// new TimeoutCertificate, otherwise, return None.
    fn aggregate_tc(
        &mut self,
        vote: &Vote,
        validator_verifier: &ValidatorVerifier,
    ) -> Option<VoteReceptionResult> {
        let timeout_signature = vote.timeout_signature().cloned()?;
        let timeout = vote.timeout();
        let tc = self
            .round_to_tc
            .entry(timeout.round())
            .or_insert_with(|| TimeoutCertificate::new(timeout));
        tc.add_signature(vote.author(), timeout_signature);
        match validator_verifier.check_voting_power(tc.signatures().keys()) {
            Ok(_) => Some(VoteReceptionResult::NewTimeoutCertificate(Arc::new(
                tc.clone(),
            ))),
            Err(VerifyError::TooLittleVotingPower { .. }) => None,
            _ => panic!("Unexpected verification error, vote = {}", vote),
        }
    }

    /// If this is the first vote from Author, add it to map. If Author has
    /// already voted on same block then return DuplicateVote error. If Author has already voted
    /// on some other result, prune the last vote and insert new one in map.
    fn replace_prev_vote(&mut self, vote: &Vote) -> Result<(), VoteReceptionResult> {
        let author = vote.author();
        let round = vote.vote_data().proposed().round();
        let li_digest = vote.ledger_info().hash();
        let is_timeout = vote.is_timeout();
        let vote_info = LastVoteInfo {
            li_digest,
            round,
            is_timeout,
        };
        let last_voted_info = match self.author_to_last_voted_info.insert(author, vote_info) {
            None => {
                // First vote from Author, do nothing.
                return Ok(());
            }
            Some(last_voted_info) => last_voted_info,
        };

        if li_digest == last_voted_info.li_digest {
            if is_timeout == last_voted_info.is_timeout {
                // Author has already voted for the very same LedgerInfo
                return Err(VoteReceptionResult::DuplicateVote);
            } else {
                // Author has already voted for this LedgerInfo, but this time the Vote's
                // round signature is different.
                // Do not replace the prev vote, try to may be gather a TC.
                return Ok(());
            }
        }

        // Prune last pending vote from the pending votes.
        if let Some(li_pending_votes) = self.li_digest_to_votes.get_mut(&last_voted_info.li_digest)
        {
            // Removing signature from last voted block
            li_pending_votes.remove_signature(author);
            if li_pending_votes.signatures().is_empty() {
                // Last vote for that LI digest, remove the digest entry
                self.li_digest_to_votes.remove(&last_voted_info.li_digest);
            }
        }

        // Prune last pending vote from the pending timeout certificates.
        if round == last_voted_info.round {
            // The same author has already sent a vote for this round but a different LedgerInfo
            // digest: this is not a valid behavior.
            error!(
                "Validator {} sent two different votes for the same round {}!",
                author.short_str(),
                round
            );
            return Err(VoteReceptionResult::EquivocateVote);
        }
        if let Some(pending_tc) = self.round_to_tc.get_mut(&last_voted_info.round) {
            // Removing signature from last tc
            pending_tc.remove_signature(author);
            if pending_tc.signatures().is_empty() {
                // Last vote for that round, remove the TC
                self.round_to_tc.remove(&last_voted_info.round);
            }
        }

        Ok(())
    }
}
