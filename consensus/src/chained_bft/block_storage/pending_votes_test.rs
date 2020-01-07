// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::block_storage::pending_votes::PendingVotes;
use crate::chained_bft::block_storage::VoteReceptionResult;
use consensus_types::{common::Round, vote::Vote, vote_data::VoteData};
use libra_crypto::HashValue;
use libra_types::block_info::BlockInfo;
use libra_types::crypto_proxies::random_validator_verifier;
use libra_types::ledger_info::LedgerInfo;

fn random_ledger_info() -> LedgerInfo {
    LedgerInfo::new(
        BlockInfo::new(1, 0, HashValue::random(), HashValue::random(), 0, 0, None),
        HashValue::random(),
    )
}

fn random_vote_data(round: Round) -> VoteData {
    assert!(round >= 1);
    VoteData::new(BlockInfo::random(round), BlockInfo::random(round - 1))
}

#[test]
/// Verify that votes are properly aggregated to QC based on their LedgerInfo digest
fn test_qc_aggregation() {
    ::libra_logger::try_init_for_testing();
    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    let li1 = random_ledger_info();
    let vote_data_1 = random_vote_data(1);
    let vote_data_1_author_0 = Vote::new(vote_data_1, signers[0].author(), li1, &signers[0]);

    // first time a new vote is added the result is VoteAdded
    assert_eq!(
        pending_votes.insert_vote(&vote_data_1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );
    // same author voting for the same thing: result is DuplicateVote
    assert_eq!(
        pending_votes.insert_vote(&vote_data_1_author_0, &validator),
        VoteReceptionResult::DuplicateVote
    );
    // same author voting for a different result in the same round:
    // override the prev value and return equivocation
    let li2 = random_ledger_info();
    let vote_data_2 = random_vote_data(1);
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
    // A different author voting for a different result in the same round but without a round
    // signature: VoteAdded
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
    // Two votes for the ledger info form a QC
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
/// Verify that only the last votes are kept in the system for qc aggregation
fn test_qc_aggregation_keep_last_only() {
    ::libra_logger::try_init_for_testing();

    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    let li1 = random_ledger_info();
    let vote_round_1 = random_vote_data(1);
    let vote_round_1_author_0 = Vote::new(
        vote_round_1.clone(),
        signers[0].author(),
        li1.clone(),
        &signers[0],
    );

    // first time a new vote is added the result is VoteAdded
    assert_eq!(
        pending_votes.insert_vote(&vote_round_1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // same author voting for the next round: the previous vote is replaced
    let li2 = random_ledger_info();
    let vote_round_2 = random_vote_data(2);
    let vote_round_2_author_0 = Vote::new(
        vote_round_2.clone(),
        signers[0].author(),
        li2.clone(),
        &signers[0],
    );
    assert_eq!(
        pending_votes.insert_vote(&vote_round_2_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // another author voting for round 1 cannot form a QC because the old vote is gone
    let vote_round_1_author_1 = Vote::new(vote_round_1, signers[1].author(), li1, &signers[1]);
    assert_eq!(
        pending_votes.insert_vote(&vote_round_1_author_1, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // another author voting for the vote data in round 2 can finally form a QC
    let vote_round_2_author_1 = Vote::new(vote_round_2, signers[1].author(), li2, &signers[1]);
    match pending_votes.insert_vote(&vote_round_2_author_1, &validator) {
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
    ::libra_logger::try_init_for_testing();

    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    let li1 = random_ledger_info();
    let vote_round_1 = random_vote_data(1);
    let mut vote_round_1_author_0 = Vote::new(vote_round_1, signers[0].author(), li1, &signers[0]);

    let timeout = vote_round_1_author_0.timeout();
    let signature = timeout.sign(&signers[0]);
    vote_round_1_author_0.add_timeout_signature(signature);

    // first time a new vote is added the result is VoteAdded
    assert_eq!(
        pending_votes.insert_vote(&vote_round_1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // another vote for the same round with a different value cannot form a TC if it doesn't have a
    // round signature
    let li2 = random_ledger_info();
    let vote2_round_1 = random_vote_data(1);
    let mut vote2_round_1_author_1 =
        Vote::new(vote2_round_1, signers[1].author(), li2, &signers[1]);
    assert_eq!(
        pending_votes.insert_vote(&vote2_round_1_author_1, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // if that vote is now enhanced with a round signature, it can form a TC
    let timeout = vote2_round_1_author_1.timeout();
    let signature = timeout.sign(&signers[1]);
    vote2_round_1_author_1.add_timeout_signature(signature);
    match pending_votes.insert_vote(&vote2_round_1_author_1, &validator) {
        VoteReceptionResult::NewTimeoutCertificate(tc) => {
            assert!(validator.check_voting_power(tc.signatures().keys()).is_ok());
        }
        _ => {
            panic!("No TC formed.");
        }
    };
}

#[test]
/// Verify that only the last votes are kept in the system for TC aggregation
fn test_tc_aggregation_keep_last_only() {
    ::libra_logger::try_init_for_testing();

    let (signers, validator) = random_validator_verifier(4, Some(2), false);
    let mut pending_votes = PendingVotes::new();

    let li1 = random_ledger_info();
    let vote_round_1 = random_vote_data(1);
    let mut vote_round_1_author_0 = Vote::new(vote_round_1, signers[0].author(), li1, &signers[0]);
    let timeout = vote_round_1_author_0.timeout();
    let signature = timeout.sign(&signers[0]);
    vote_round_1_author_0.add_timeout_signature(signature);

    // first time a new vote is added the result is VoteAdded
    assert_eq!(
        pending_votes.insert_vote(&vote_round_1_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // A vote for round 2 overrides the previous vote
    let li2 = random_ledger_info();
    let vote_round_2 = random_vote_data(2);
    let mut vote_round_2_author_0 = Vote::new(vote_round_2, signers[0].author(), li2, &signers[0]);
    let timeout = vote_round_2_author_0.timeout();
    let signature = timeout.sign(&signers[0]);
    vote_round_2_author_0.add_timeout_signature(signature);
    assert_eq!(
        pending_votes.insert_vote(&vote_round_2_author_0, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // a new vote for round 1 cannot form a TC
    let li3 = random_ledger_info();
    let vote3_round_1 = random_vote_data(1);
    let mut vote3_round_1_author_1 =
        Vote::new(vote3_round_1, signers[1].author(), li3, &signers[1]);
    let timeout = vote3_round_1_author_1.timeout();
    let signature = timeout.sign(&signers[1]);
    vote3_round_1_author_1.add_timeout_signature(signature);
    assert_eq!(
        pending_votes.insert_vote(&vote3_round_1_author_1, &validator),
        VoteReceptionResult::VoteAdded(1)
    );

    // a new vote for round 2 should form a TC
    let li4 = random_ledger_info();
    let vote4_round_2 = random_vote_data(2);
    let mut vote4_round_2_author_1 =
        Vote::new(vote4_round_2, signers[1].author(), li4, &signers[1]);
    let timeout = vote4_round_2_author_1.timeout();
    let signature = timeout.sign(&signers[1]);
    vote4_round_2_author_1.add_timeout_signature(signature);
    match pending_votes.insert_vote(&vote4_round_2_author_1, &validator) {
        VoteReceptionResult::NewTimeoutCertificate(tc) => {
            assert!(validator.check_voting_power(tc.signatures().keys()).is_ok());
        }
        _ => {
            panic!("No TC formed.");
        }
    };
}
