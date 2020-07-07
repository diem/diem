// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::block_storage::{pending_votes::PendingVotes, VoteReceptionResult};
use consensus_types::{vote::Vote, vote_data::VoteData};
use libra_crypto::HashValue;
use libra_types::{
    block_info::BlockInfo, ledger_info::LedgerInfo, validator_verifier::random_validator_verifier,
};

/// Creates a random ledger info for epoch 1 and round 1.
fn random_ledger_info() -> LedgerInfo {
    LedgerInfo::new(
        BlockInfo::new(1, 0, HashValue::random(), HashValue::random(), 0, 0, None),
        HashValue::random(),
    )
}

/// Creates a random VoteData for epoch 1 and round 1,
/// extending a random block at epoch1 and round 0.
fn random_vote_data() -> VoteData {
    VoteData::new(BlockInfo::random(1), BlockInfo::random(0))
}

#[test]
/// Verify that votes are properly aggregated to QC based on their LedgerInfo digest
fn test_qc_aggregation() {
    ::libra_logger::Logger::new().environment_only(true).init();

    // set up 4 validators
    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    // create random vote from validator[0]
    let li1 = random_ledger_info();
    let vote_data_1 = random_vote_data();
    let vote_data_1_author_0 = Vote::new(vote_data_1, signers[0].author(), li1, &signers[0]);

    // first time a new vote is added -> VoteAdded
    assert_eq!(
        pending_votes.insert_vote(&vote_data_1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // same author voting for the same thing -> DuplicateVote
    assert_eq!(
        pending_votes.insert_vote(&vote_data_1_author_0, &validator),
        VoteReceptionResult::DuplicateVote
    );

    // same author voting for a different result -> EquivocateVote
    let li2 = random_ledger_info();
    let vote_data_2 = random_vote_data();
    let vote_data_2_author_0 = Vote::new(
        vote_data_2.clone(),
        signers[0].author(),
        li2.clone(),
        &signers[0],
    );
    assert_eq!(
        pending_votes.insert_vote(&vote_data_2_author_0, &validator),
        VoteReceptionResult::EquivocateVote
    );

    // a different author voting for a different result -> VoteAdded
    let vote_data_2_author_1 = Vote::new(
        vote_data_2.clone(),
        signers[1].author(),
        li2.clone(),
        &signers[1],
    );
    assert_eq!(
        pending_votes.insert_vote(&vote_data_2_author_1, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // two votes for the ledger info -> NewQuorumCertificate
    let vote_data_2_author_2 = Vote::new(vote_data_2, signers[2].author(), li2, &signers[2]);
    match pending_votes.insert_vote(&vote_data_2_author_2, &validator) {
        VoteReceptionResult::NewQuorumCertificate(qc) => {
            assert!(validator
                .check_voting_power(qc.ledger_info().signatures().keys())
                .is_ok());
        }
        _ => {
            panic!("No QC formed.");
        }
    };
}

#[test]
/// Verify that votes are properly aggregated to TC based on their rounds
fn test_tc_aggregation() {
    ::libra_logger::Logger::new().environment_only(true).init();

    // set up 4 validators
    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    // submit a new vote from validator[0] -> VoteAdded
    let li1 = random_ledger_info();
    let vote1 = random_vote_data();
    let mut vote1_author_0 = Vote::new(vote1, signers[0].author(), li1, &signers[0]);

    assert_eq!(
        pending_votes.insert_vote(&vote1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // submit the same vote but enhanced with a timeout -> VoteAdded
    let timeout = vote1_author_0.timeout();
    let signature = timeout.sign(&signers[0]);
    vote1_author_0.add_timeout_signature(signature);

    assert_eq!(
        pending_votes.insert_vote(&vote1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // another vote for a different block cannot form a TC if it doesn't have a timeout signature
    let li2 = random_ledger_info();
    let vote2 = random_vote_data();
    let mut vote2_author_1 = Vote::new(vote2, signers[1].author(), li2, &signers[1]);
    assert_eq!(
        pending_votes.insert_vote(&vote2_author_1, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // if that vote is now enhanced with a timeout signature -> NewTimeoutCertificate
    let timeout = vote2_author_1.timeout();
    let signature = timeout.sign(&signers[1]);
    vote2_author_1.add_timeout_signature(signature);
    match pending_votes.insert_vote(&vote2_author_1, &validator) {
        VoteReceptionResult::NewTimeoutCertificate(tc) => {
            assert!(validator.check_voting_power(tc.signatures().keys()).is_ok());
        }
        _ => {
            panic!("No TC formed.");
        }
    };
}
