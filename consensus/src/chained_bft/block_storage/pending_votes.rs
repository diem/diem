// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    block_storage::VoteReceptionResult,
    common::Author,
    consensus_types::{quorum_cert::QuorumCert, vote_msg::VoteMsg},
};
use crypto::{hash::CryptoHash, HashValue};
use std::{collections::HashMap, sync::Arc};
use types::{
    crypto_proxies::{LedgerInfoWithSignatures, ValidatorVerifier},
    validator_verifier::VerifyError,
};

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
    /// Map of Author to last voted ledger info digest. Any pending vote from Author is cleaned up
    /// whenever new vote is added by same Author
    author_to_last_voted_li_digest: HashMap<Author, HashValue>,
}

impl PendingVotes {
    pub fn new() -> Self {
        PendingVotes {
            li_digest_to_votes: HashMap::new(),
            author_to_last_voted_li_digest: HashMap::new(),
        }
    }

    pub fn insert_vote(
        &mut self,
        vote_msg: &VoteMsg,
        validator_verifier: Arc<ValidatorVerifier>,
    ) -> VoteReceptionResult {
        let author = vote_msg.author();
        // Note that the digest covers the ledger info information, which is also indirectly
        // covering vote data hash (in its `consensus_data_hash` field).
        let li_digest = vote_msg.ledger_info().hash();

        if let Err(e) = self.check_vote_valid(vote_msg) {
            return e;
        }

        let li_with_sig = self.li_digest_to_votes.entry(li_digest).or_insert_with(|| {
            LedgerInfoWithSignatures::new(vote_msg.ledger_info().clone(), HashMap::new())
        });
        vote_msg.signature().clone().add_to_li(author, li_with_sig);

        match validator_verifier.check_voting_power(li_with_sig.signatures().keys()) {
            Ok(_) => VoteReceptionResult::NewQuorumCertificate(Arc::new(QuorumCert::new(
                vote_msg.vote_data().clone(),
                li_with_sig.clone(),
            ))),
            Err(VerifyError::TooLittleVotingPower { voting_power, .. }) => {
                VoteReceptionResult::VoteAdded(voting_power)
            }
            _ => panic!("Unexpected verification error, vote_msg = {}", vote_msg),
        }
    }

    /// Check if vote is valid. If this is the first vote from Author, add it to map. If Author has
    /// already voted on same block then return DuplicateVote error. If Author has already voted
    /// on some other result, prune the last vote and insert new one in map.
    fn check_vote_valid(&mut self, vote_msg: &VoteMsg) -> Result<(), VoteReceptionResult> {
        let author = vote_msg.author();
        let li_digest = vote_msg.ledger_info().hash();
        let last_voted_li_digest = match self
            .author_to_last_voted_li_digest
            .insert(author, li_digest)
        {
            None => {
                // First vote from Author, do nothing.
                return Ok(());
            }
            Some(last_voted_li_digest) => last_voted_li_digest,
        };

        // Prune last pending vote from Author
        if li_digest == last_voted_li_digest {
            // Author has already voted for this LedgerInfo
            return Err(VoteReceptionResult::DuplicateVote);
        }

        if let Some(li_pending_votes) = self.li_digest_to_votes.get_mut(&last_voted_li_digest) {
            // Removing signature from last voted block
            li_pending_votes.remove_signature(author);
            if li_pending_votes.signatures().is_empty() {
                // Last vote for that LI digest, remove the digest entry
                self.li_digest_to_votes.remove(&last_voted_li_digest);
            }
        }
        Ok(())
    }
}
