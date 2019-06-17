// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockReader,
        liveness::proposal_generator::{ProposalGenerationError, ProposalGenerator},
        safety::vote_msg::VoteMsg,
        test_utils::{
            build_empty_tree, placeholder_ledger_info, MockTransactionManager, TreeInserter,
        },
    },
    mock_time_service::SimulatedTimeService,
};
use futures::executor::block_on;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn minute_from_now() -> Instant {
    Instant::now() + Duration::new(60, 0)
}

#[test]
fn test_proposal_generation_empty_tree() {
    let block_store = build_empty_tree();
    let proposal_generator = ProposalGenerator::new(
        block_store.clone(),
        Arc::new(MockTransactionManager::new()),
        Arc::new(SimulatedTimeService::new()),
        1,
        true,
    );
    let genesis = block_store.root();

    // Generate proposals for an empty tree.
    let proposal = block_on(proposal_generator.generate_proposal(1, minute_from_now())).unwrap();
    assert_eq!(proposal.parent_id(), genesis.id());
    assert_eq!(proposal.round(), 1);
    assert_eq!(proposal.height(), 1);
    assert_eq!(proposal.quorum_cert().certified_block_id(), genesis.id());

    // Duplicate proposals on the same round are not allowed
    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert_eq!(
        proposal_err.unwrap(),
        ProposalGenerationError::AlreadyProposed(1)
    );
}

#[test]
fn test_proposal_generation_parent() {
    let block_store = build_empty_tree();
    let mut inserter = TreeInserter::new(block_store.clone());
    let proposal_generator = ProposalGenerator::new(
        block_store.clone(),
        Arc::new(MockTransactionManager::new()),
        Arc::new(SimulatedTimeService::new()),
        1,
        true,
    );
    let genesis = block_store.root();
    let a1 = inserter.insert_block(genesis.as_ref(), 1);
    let b1 = inserter.insert_block(genesis.as_ref(), 2);

    // With no certifications the parent is genesis
    // generate proposals for an empty tree.
    assert_eq!(
        block_on(proposal_generator.generate_proposal(10, minute_from_now()))
            .unwrap()
            .parent_id(),
        genesis.id()
    );

    // Once a1 is certified, it should be the one to choose from
    let vote_msg_a1 = VoteMsg::new(
        a1.id(),
        block_store.get_state_for_block(a1.id()).unwrap(),
        a1.round(),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
    );
    block_on(block_store.insert_vote_and_qc(vote_msg_a1, 1));
    let a1_child_res =
        block_on(proposal_generator.generate_proposal(11, minute_from_now())).unwrap();
    assert_eq!(a1_child_res.parent_id(), a1.id());
    assert_eq!(a1_child_res.round(), 11);
    assert_eq!(a1_child_res.height(), 2);
    assert_eq!(a1_child_res.quorum_cert().certified_block_id(), a1.id());

    // Once b1 is certified, it should be the one to choose from
    let vote_msg_b1 = VoteMsg::new(
        b1.id(),
        block_store.get_state_for_block(b1.id()).unwrap(),
        b1.round(),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
    );

    block_on(block_store.insert_vote_and_qc(vote_msg_b1, 1));
    let b1_child_res =
        block_on(proposal_generator.generate_proposal(12, minute_from_now())).unwrap();
    assert_eq!(b1_child_res.parent_id(), b1.id());
    assert_eq!(b1_child_res.round(), 12);
    assert_eq!(b1_child_res.height(), 2);
    assert_eq!(b1_child_res.quorum_cert().certified_block_id(), b1.id());
}

#[test]
fn test_old_proposal_generation() {
    let block_store = build_empty_tree();
    let mut inserter = TreeInserter::new(block_store.clone());
    let proposal_generator = ProposalGenerator::new(
        block_store.clone(),
        Arc::new(MockTransactionManager::new()),
        Arc::new(SimulatedTimeService::new()),
        1,
        true,
    );
    let genesis = block_store.root();
    let a1 = inserter.insert_block(genesis.as_ref(), 1);
    let vote_msg_a1 = VoteMsg::new(
        a1.id(),
        block_store.get_state_for_block(a1.id()).unwrap(),
        a1.round(),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
    );
    block_on(block_store.insert_vote_and_qc(vote_msg_a1, 1));

    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert_eq!(
        proposal_err.unwrap(),
        ProposalGenerationError::GivenRoundTooLow(1)
    );
}
