// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockReader,
        liveness::proposal_generator::ProposalGenerator,
        test_utils::{
            self, build_empty_tree, placeholder_ledger_info, MockTransactionManager, TreeInserter,
        },
    },
    util::mock_time_service::SimulatedTimeService,
};
use consensus_types::{quorum_cert::QuorumCert, vote_data::VoteData, vote_msg::VoteMsg};
use futures::executor::block_on;
use libra_types::crypto_proxies::ValidatorVerifier;
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
    assert_eq!(proposal.quorum_cert().certified_block_id(), genesis.id());

    // Duplicate proposals on the same round are not allowed
    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert!(proposal_err.is_some());
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
    let a1 = inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis, 1);
    let b1 = inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis, 2);

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
        VoteData::new(
            a1.id(),
            block_store
                .get_compute_result(a1.id())
                .unwrap()
                .executed_state
                .state_id,
            a1.round(),
            a1.quorum_cert().parent_block_id(),
            a1.quorum_cert().parent_block_round(),
        ),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
        test_utils::placeholder_sync_info(),
    );
    let validator_verifier = Arc::new(ValidatorVerifier::new_single(
        block_store.signer().author(),
        block_store.signer().public_key(),
    ));
    block_store.insert_vote_and_qc(vote_msg_a1, validator_verifier);
    let a1_child_res =
        block_on(proposal_generator.generate_proposal(11, minute_from_now())).unwrap();
    assert_eq!(a1_child_res.parent_id(), a1.id());
    assert_eq!(a1_child_res.round(), 11);
    assert_eq!(a1_child_res.quorum_cert().certified_block_id(), a1.id());

    // Once b1 is certified, it should be the one to choose from
    let vote_msg_b1 = VoteMsg::new(
        VoteData::new(
            b1.id(),
            block_store
                .get_compute_result(b1.id())
                .unwrap()
                .executed_state
                .state_id,
            b1.round(),
            b1.quorum_cert().parent_block_id(),
            b1.quorum_cert().parent_block_round(),
        ),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
        test_utils::placeholder_sync_info(),
    );
    let validator_verifier = Arc::new(ValidatorVerifier::new_single(
        block_store.signer().author(),
        block_store.signer().public_key(),
    ));
    block_store.insert_vote_and_qc(vote_msg_b1, validator_verifier);
    let b1_child_res =
        block_on(proposal_generator.generate_proposal(12, minute_from_now())).unwrap();
    assert_eq!(b1_child_res.parent_id(), b1.id());
    assert_eq!(b1_child_res.round(), 12);
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
    let a1 = inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), &genesis, 1);
    let vote_msg_a1 = VoteMsg::new(
        VoteData::new(
            a1.id(),
            block_store
                .get_compute_result(a1.id())
                .unwrap()
                .executed_state
                .state_id,
            a1.round(),
            a1.quorum_cert().parent_block_id(),
            a1.quorum_cert().parent_block_round(),
        ),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
        test_utils::placeholder_sync_info(),
    );
    let validator_verifier = Arc::new(ValidatorVerifier::new_single(
        block_store.signer().author(),
        block_store.signer().public_key(),
    ));
    block_store.insert_vote_and_qc(vote_msg_a1, validator_verifier);

    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert!(proposal_err.is_some());
}
