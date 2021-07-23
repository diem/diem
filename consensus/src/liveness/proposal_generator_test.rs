// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    block_storage::BlockReader,
    liveness::proposal_generator::ProposalGenerator,
    test_utils::{build_empty_tree, MockTransactionManager, TreeInserter},
    util::mock_time_service::SimulatedTimeService,
};
use consensus_types::block::{block_test_utils::certificate_for_genesis, Block};
use diem_types::validator_signer::ValidatorSigner;
use std::sync::Arc;

#[tokio::test]
async fn test_proposal_generation_empty_tree() {
    let signer = ValidatorSigner::random(None);
    let block_store = build_empty_tree();
    let mut proposal_generator = ProposalGenerator::new(
        signer.author(),
        block_store.clone(),
        Arc::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.ordered_root();

    // Generate proposals for an empty tree.
    let proposal_data = proposal_generator.generate_proposal(1).await.unwrap();
    let proposal = Block::new_proposal_from_block_data(proposal_data, &signer);
    assert_eq!(proposal.parent_id(), genesis.id());
    assert_eq!(proposal.round(), 1);
    assert_eq!(proposal.quorum_cert().certified_block().id(), genesis.id());

    // Duplicate proposals on the same round are not allowed
    let proposal_err = proposal_generator.generate_proposal(1).await.err();
    assert!(proposal_err.is_some());
}

#[tokio::test]
async fn test_proposal_generation_parent() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let mut proposal_generator = ProposalGenerator::new(
        inserter.signer().author(),
        block_store.clone(),
        Arc::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.ordered_root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let b1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 2);

    // With no certifications the parent is genesis
    // generate proposals for an empty tree.
    assert_eq!(
        proposal_generator
            .generate_proposal(10)
            .await
            .unwrap()
            .parent_id(),
        genesis.id()
    );

    // Once a1 is certified, it should be the one to choose from
    inserter.insert_qc_for_block(a1.as_ref(), None);
    let a1_child_res = proposal_generator.generate_proposal(11).await.unwrap();
    assert_eq!(a1_child_res.parent_id(), a1.id());
    assert_eq!(a1_child_res.round(), 11);
    assert_eq!(a1_child_res.quorum_cert().certified_block().id(), a1.id());

    // Once b1 is certified, it should be the one to choose from
    inserter.insert_qc_for_block(b1.as_ref(), None);
    let b1_child_res = proposal_generator.generate_proposal(12).await.unwrap();
    assert_eq!(b1_child_res.parent_id(), b1.id());
    assert_eq!(b1_child_res.round(), 12);
    assert_eq!(b1_child_res.quorum_cert().certified_block().id(), b1.id());
}

#[tokio::test]
async fn test_old_proposal_generation() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let mut proposal_generator = ProposalGenerator::new(
        inserter.signer().author(),
        block_store.clone(),
        Arc::new(MockTransactionManager::new(None)),
        Arc::new(SimulatedTimeService::new()),
        1,
    );
    let genesis = block_store.ordered_root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    inserter.insert_qc_for_block(a1.as_ref(), None);

    let proposal_err = proposal_generator.generate_proposal(1).await.err();
    assert!(proposal_err.is_some());
}
