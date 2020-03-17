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
use consensus_types::block::{
    block_test_utils::{certificate_for_genesis, gen_test_certificate},
    Block,
};
use futures::executor::block_on;
use libra_types::validator_signer::ValidatorSigner;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn minute_from_now() -> Instant {
    Instant::now() + Duration::new(60, 0)
}

#[test]
fn test_proposal_generation_empty_tree() {
    let signer = ValidatorSigner::random(None);
    let block_store = build_empty_tree();
    let mut proposal_generator = ProposalGenerator::new(
        signer.author(),
        block_store.clone(),
        Box::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.root();

    // Generate proposals for an empty tree.
    let proposal_data =
        block_on(proposal_generator.generate_proposal(1, minute_from_now())).unwrap();
    let proposal = Block::new_proposal_from_block_data(proposal_data, &signer);
    assert_eq!(proposal.parent_id(), genesis.id());
    assert_eq!(proposal.round(), 1);
    assert_eq!(proposal.quorum_cert().certified_block().id(), genesis.id());

    // Duplicate proposals on the same round are not allowed
    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert!(proposal_err.is_some());
}

#[test]
fn test_proposal_generation_parent() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let mut proposal_generator = ProposalGenerator::new(
        inserter.signer().author(),
        block_store.clone(),
        Box::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let b1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 2);

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
    assert_eq!(a1_child_res.quorum_cert().certified_block().id(), a1.id());

    // Once b1 is certified, it should be the one to choose from
    inserter.insert_qc_for_block(b1.as_ref(), None);
    let b1_child_res =
        block_on(proposal_generator.generate_proposal(12, minute_from_now())).unwrap();
    assert_eq!(b1_child_res.parent_id(), b1.id());
    assert_eq!(b1_child_res.round(), 12);
    assert_eq!(b1_child_res.quorum_cert().certified_block().id(), b1.id());
}

#[test]
fn test_old_proposal_generation() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let mut proposal_generator = ProposalGenerator::new(
        inserter.signer().author(),
        block_store.clone(),
        Box::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    inserter.insert_qc_for_block(a1.as_ref(), None);

    let proposal_err = block_on(proposal_generator.generate_proposal(1, minute_from_now())).err();
    assert!(proposal_err.is_some());
}

#[test]
fn test_empty_proposal_after_reconfiguration() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let mut proposal_generator = ProposalGenerator::new(
        inserter.signer().author(),
        block_store.clone(),
        Box::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    // Normal proposal is not empty
    let normal_proposal_1 =
        block_on(proposal_generator.generate_proposal(42, minute_from_now())).unwrap();
    assert!(!normal_proposal_1.payload().unwrap().is_empty());
    let a2 = inserter.insert_reconfiguration_block(&a1, 2);
    inserter.insert_qc_for_block(a2.as_ref(), None);
    // The direct child is empty
    let empty_proposal_1 =
        block_on(proposal_generator.generate_proposal(43, minute_from_now())).unwrap();
    assert!(empty_proposal_1.payload().unwrap().is_empty());
    // insert one more block after reconfiguration
    let a3 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a2.as_ref(), None),
        a2.as_ref().timestamp_usecs(),
        3,
        vec![],
    );
    let a3 = block_store.execute_and_insert_block(a3).unwrap();
    inserter.insert_qc_for_block(a3.as_ref(), None);
    // Indirect child is empty too
    let empty_proposal_2 =
        block_on(proposal_generator.generate_proposal(44, minute_from_now())).unwrap();
    assert!(empty_proposal_2.payload().unwrap().is_empty());
    // if reconfiguration is committed, not allow to generate proposal
    let a4 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a3.as_ref(), None),
        a2.as_ref().timestamp_usecs(),
        4,
        vec![],
    );
    let a4 = block_store.execute_and_insert_block(a4).unwrap();
    let li = gen_test_certificate(
        vec![inserter.signer()],
        a4.block_info(),
        a3.block_info(),
        Some(a2.block_info()),
    );
    block_store.insert_single_quorum_cert(li).unwrap();
    let err_proposal = block_on(proposal_generator.generate_proposal(45, minute_from_now()));
    assert!(err_proposal.is_err());
}
