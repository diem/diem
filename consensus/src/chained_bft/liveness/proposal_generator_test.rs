// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    chained_bft::{
        block_storage::BlockReader,
        liveness::proposal_generator::ProposalGenerator,
        test_utils::{build_empty_tree, MockTransactionManager, TreeInserter},
    },
    util::mock_time_service::SimulatedTimeService,
};
use consensus_types::quorum_cert::QuorumCert;
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
    inserter.insert_qc_for_block(a1.as_ref(), None);
    let a1_child_res =
        block_on(proposal_generator.generate_proposal(11, minute_from_now())).unwrap();
    assert_eq!(a1_child_res.parent_id(), a1.id());
    assert_eq!(a1_child_res.round(), 11);
    assert_eq!(a1_child_res.quorum_cert().certified_block_id(), a1.id());

    // Once b1 is certified, it should be the one to choose from
    inserter.insert_qc_for_block(b1.as_ref(), None);
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
    inserter.insert_qc_for_block(a1.as_ref(), None);

    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert!(proposal_err.is_some());
}

#[test]
fn test_empty_proposal_after_reconfiguration() {
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
    // Normal proposal is not empty
    let normal_proposal =
        block_on(proposal_generator.generate_proposal(42, minute_from_now())).unwrap();
    assert!(!normal_proposal.payload().unwrap().is_empty());
    let a2 = inserter.insert_reconfiguration_block(&a1, 2);
    inserter.insert_qc_for_block(a2.as_ref(), None);
    // The direct child is empty
    let empty_proposal_1 =
        block_on(proposal_generator.generate_proposal(43, minute_from_now())).unwrap();
    assert!(empty_proposal_1.payload().unwrap().is_empty());
    // insert one more block after reconfiguration
    let a3 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a2.as_ref(), None),
        a2.as_ref(),
        4,
        vec![],
    );
    let a3 = block_on(block_store.execute_and_insert_block(a3)).unwrap();
    inserter.insert_qc_for_block(a3.as_ref(), None);
    // Indirect child is empty too
    let empty_proposal_2 =
        block_on(proposal_generator.generate_proposal(44, minute_from_now())).unwrap();
    assert!(empty_proposal_2.payload().unwrap().is_empty());
}
