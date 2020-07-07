// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! PendingVotes store pending votes observed for a fixed epoch and round.
//! It is meant to be used inside of a RoundState.
//! The module takes care of creating a QC or a TC
//! when enough votes (or timeout votes) have been observed.
//! Votes are automatically dropped when the structure goes out of scope.

use crate::block_storage::VoteReceptionResult;
use consensus_types::{
    common::Author, quorum_cert::QuorumCert, timeout_certificate::TimeoutCertificate, vote::Vote,
};
use libra_crypto::{hash::CryptoHash, HashValue};
use libra_logger::prelude::*;
use libra_types::{
    ledger_info::LedgerInfoWithSignatures,
    validator_verifier::{ValidatorVerifier, VerifyError},
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    sync::Arc,
};

#[cfg(test)]
#[path = "pending_votes_test.rs"]
mod pending_votes_test;

/// A PendingVotes structure keep track of votes
pub struct PendingVotes {
    /// Maps LedgerInfo digest to associated signatures (contained in a partial LedgerInfoWithSignatures).
    /// This might keep multiple LedgerInfos (for the current round) to tolerate non-determinism in execution.
    li_digest_to_votes: HashMap<HashValue /* LedgerInfo digest */, LedgerInfoWithSignatures>,
    /// Tracks all the signatures of the votes for the given round. In case we succeed to
    /// aggregate 2f+1 signatures a TimeoutCertificate is formed.
    maybe_partial_tc: Option<TimeoutCertificate>,
    /// Map of Author to vote. This is useful to discard multiple votes.
    author_to_vote: HashMap<Author, Vote>,
}

impl PendingVotes {
    /// Creates an empty PendingVotes structure for a specific epoch and round
    pub fn new() -> Self {
        PendingVotes {
            li_digest_to_votes: HashMap::new(),
            maybe_partial_tc: None,
            author_to_vote: HashMap::new(),
        }
    }

    /// Insert a vote and if the vote is valid, return a QuorumCertificate preferentially over a
    /// TimeoutCertificate if either can can be formed
    pub fn insert_vote(
        &mut self,
        vote: &Vote,
        validator_verifier: &ValidatorVerifier,
    ) -> VoteReceptionResult {
        // derive data from vote
        let li_digest = vote.ledger_info().hash();

        //
        // 1. Has the author already voted for this round?
        //

        if let Some(previously_seen_vote) = self.author_to_vote.get(&vote.author()) {
            // is it the same vote?
            if li_digest == previously_seen_vote.ledger_info().hash() {
                // we've already seen an equivalent vote before
                let new_timeout_vote = vote.is_timeout() && !previously_seen_vote.is_timeout();
                if !new_timeout_vote {
                    // it's not a new timeout vote
                    return VoteReceptionResult::DuplicateVote;
                }
            } else {
                // we have seen a different vote for the same round
                return VoteReceptionResult::EquivocateVote;
            }
        }

        //
        // 2. Store new vote (or update, in case it's a new timeout vote)
        //

        self.author_to_vote.insert(vote.author(), vote.clone());

        //
        // 3. Let's check if we can create a QC
        //

        // obtain the ledger info with signatures associated to the vote's ledger info
        let li_with_sig = self.li_digest_to_votes.entry(li_digest).or_insert_with(|| {
            // if the ledger info with signatures doesn't exist yet, create it
            LedgerInfoWithSignatures::new(vote.ledger_info().clone(), BTreeMap::new())
        });

        // add this vote to the ledger info with signatures
        li_with_sig.add_signature(vote.author(), vote.signature().clone());

        // check if we have enough signatures to create a QC
        let voting_power =
            match validator_verifier.check_voting_power(li_with_sig.signatures().keys()) {
                // a quorum of signature was reached, a new QC is formed
                Ok(_) => {
                    return VoteReceptionResult::NewQuorumCertificate(Arc::new(QuorumCert::new(
                        vote.vote_data().clone(),
                        li_with_sig.clone(),
                    )));
                }

                // not enough votes
                Err(VerifyError::TooLittleVotingPower { voting_power, .. }) => voting_power,

                // error
                Err(error) => {
                    error!(
                        "MUST_FIX: vote received could not be added: {}, vote: {}",
                        error, vote
                    );
                    return VoteReceptionResult::ErrorAddingVote(error);
                }
            };

        //
        // 4. We couldn't form a QC, let's check if we can create a TC
        //

        if let Some(timeout_signature) = vote.timeout_signature() {
            // form timeout struct
            // TODO(mimoo): stronger: pass the (epoch, round) tuple as arguments of this function
            let timeout = vote.timeout();

            // if no partial TC exist, create one
            let partial_tc = self
                .maybe_partial_tc
                .get_or_insert_with(|| TimeoutCertificate::new(timeout));

            // add the timeout signature
            partial_tc.add_signature(vote.author(), timeout_signature.clone());

            // did the TC reach a threshold?
            match validator_verifier.check_voting_power(partial_tc.signatures().keys()) {
                // A quorum of signature was reached, a new TC was formed!
                Ok(_) => {
                    return VoteReceptionResult::NewTimeoutCertificate(Arc::new(partial_tc.clone()))
                }

                // not enough votes
                Err(VerifyError::TooLittleVotingPower { .. }) => (),

                // error
                Err(error) => {
                    error!(
                        "MUST_FIX: timeout vote received could not be added: {}, vote: {}",
                        error, vote
                    );
                    return VoteReceptionResult::ErrorAddingVote(error);
                }
            }
        }

        //
        // 5. No QC (or TC) could be formed, return the QC's voting power
        //

        VoteReceptionResult::VoteAdded(voting_power)
    }
}

impl fmt::Display for PendingVotes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // collect votes per ledger info
        let votes = self
            .li_digest_to_votes
            .iter()
            .map(|(li_digest, li)| (li_digest, li.signatures().keys().collect::<Vec<_>>()))
            .collect::<BTreeMap<_, _>>();

        // collect timeout votes
        let timeout_votes = self
            .maybe_partial_tc
            .as_ref()
            .map(|partial_tc| partial_tc.signatures().keys().collect::<Vec<_>>());

        // write
        write!(f, "PendingVotes: [")?;

        for (hash, authors) in votes {
            write!(f, "LI {} has {} votes {:?} ", hash, authors.len(), authors)?;
        }

        for authors in timeout_votes {
            write!(f, "{} timeout {:?}", authors.len(), authors)?;
        }

        write!(f, "]")
    }
}
