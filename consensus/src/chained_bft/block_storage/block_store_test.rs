// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    block_storage::{
        block_store::sync_manager::NeedFetchResult, BlockReader, PendingVotes, VoteReceptionResult,
    },
    test_utils::{build_empty_tree, build_simple_tree, TreeInserter},
};
use consensus_types::{
    block::{
        block_test_utils::{
            self, certificate_for_genesis, placeholder_certificate_for_block,
            placeholder_ledger_info,
        },
        Block,
    },
    common::Author,
    vote::Vote,
    vote_data::VoteData,
};
use libra_crypto::{HashValue, TPrivateKey};
use libra_types::{
    account_address::AccountAddress, validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier,
};
use proptest::prelude::*;
use std::{cmp::min, collections::HashSet};

#[test]
fn test_highest_block_and_quorum_cert() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    assert_eq!(
        block_store.highest_certified_block().block(),
        &Block::make_genesis_block()
    );
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        &certificate_for_genesis()
    );

    let genesis = block_store.root();

    // Genesis block and quorum certificate is still the highest
    let block_round_1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    assert_eq!(
        block_store.highest_certified_block().block(),
        &Block::make_genesis_block()
    );
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        &certificate_for_genesis()
    );

    // block_round_1 block and quorum certificate is now the highest
    let block_round_3 = inserter.insert_block(&block_round_1, 3, None);
    assert_eq!(block_store.highest_certified_block(), block_round_1);
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        block_store
            .get_block(block_round_3.id())
            .expect("block_round_1 should exist")
            .quorum_cert()
    );

    // block_round_1 block and quorum certificate is still the highest, since block_round_4
    // also builds on block_round_1
    let block_round_4 = inserter.insert_block(&block_round_1, 4, None);
    assert_eq!(block_store.highest_certified_block(), block_round_1);
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        block_store
            .get_block(block_round_4.id())
            .expect("block_round_1 should exist")
            .quorum_cert()
    );
}

#[test]
fn test_qc_ancestry() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let block_a_1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let block_a_2 = inserter.insert_block(&block_a_1, 2, None);

    assert_eq!(
        block_store.get_block(genesis.quorum_cert().certified_block().id()),
        None
    );
    assert_eq!(
        block_store.get_block(block_a_1.quorum_cert().certified_block().id()),
        Some(genesis)
    );
    assert_eq!(
        block_store.get_block(block_a_2.quorum_cert().certified_block().id()),
        Some(block_a_1)
    );
}

// This test should be continuously extended to eventually become the
// single-page spec for the logic of our block storage.
proptest! {

    #[test]
    fn test_block_store_insert(
        (private_keys, blocks) in block_test_utils::block_forest_and_its_keys(
            // quorum size
            10,
            // recursion depth
            50)
    ){
        let authors: HashSet<Author> = private_keys.iter().map(|private_key| AccountAddress::from_public_key(&private_key.public_key())).collect();
        let block_store = build_empty_tree();
        for block in blocks {
            if block.round() > 0 && authors.contains(&block.author().unwrap()) {
                let known_parent = block_store.block_exists(block.parent_id());
                let certified_parent = block.quorum_cert().certified_block().id() == block.parent_id();
                let verify_res = block.verify_well_formed();
                let res = block_store.execute_and_insert_block(block.clone());
                if !certified_parent {
                    prop_assert!(verify_res.is_err());
                } else if !known_parent {
                    // We cannot really bring blocks in this test because the block retrieval
                    // functionality invokes event processing, which is not setup here.
                    assert!(res.is_err());
                }
                else {
                    // The parent must be present if we get to this line.
                    let parent = block_store.get_block(block.parent_id()).unwrap();
                    if block.round() <= parent.round() {
                        prop_assert!(res.is_err());
                    } else {
                        let executed_block = res.unwrap();
                        prop_assert_eq!(executed_block.block(),
                             &block,
                            "expected ok on block: {:#?}, got {:#?}", block, executed_block.block());
                    }
                }
            }
        }
    }
}

#[test]
fn test_block_store_prune() {
    let (blocks, block_store) = build_simple_tree();
    // Attempt to prune genesis block (should be no-op)
    assert_eq!(block_store.prune_tree(blocks[0].id()).len(), 0);
    assert_eq!(block_store.len(), 7);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    assert_eq!(block_store.pruned_blocks_in_mem(), 0);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block A1
    assert_eq!(block_store.prune_tree(blocks[1].id()).len(), 4);
    assert_eq!(block_store.len(), 3);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    assert_eq!(block_store.pruned_blocks_in_mem(), 4);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block A2
    assert_eq!(block_store.prune_tree(blocks[2].id()).len(), 5);
    assert_eq!(block_store.len(), 2);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    assert_eq!(block_store.pruned_blocks_in_mem(), 5);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block A3
    assert_eq!(block_store.prune_tree(blocks[3].id()).len(), 6);
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block B1
    assert_eq!(block_store.prune_tree(blocks[4].id()).len(), 4);
    assert_eq!(block_store.len(), 3);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block B2
    assert_eq!(block_store.prune_tree(blocks[5].id()).len(), 6);
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    let (blocks, block_store) = build_simple_tree();
    // Prune up to block C1
    assert_eq!(block_store.prune_tree(blocks[6].id()).len(), 6);
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    // Prune the chain of Genesis -> B1 -> B2
    let (blocks, block_store) = build_simple_tree();
    // Prune up to block B1
    assert_eq!(block_store.prune_tree(blocks[4].id()).len(), 4);
    assert_eq!(block_store.len(), 3);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    // Prune up to block B2
    assert_eq!(block_store.prune_tree(blocks[5].id()).len(), 2);
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
}

#[test]
fn test_block_tree_gc() {
    // build a tree with 100 nodes, max_pruned_nodes_in_mem = 10
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let mut cur_node = block_store.get_block(genesis.id()).unwrap();
    let mut added_blocks = vec![];

    for round in 1..100 {
        if round == 1 {
            cur_node = inserter.insert_block_with_qc(certificate_for_genesis(), &cur_node, round);
        } else {
            cur_node = inserter.insert_block(&cur_node, round, None);
        }
        added_blocks.push(cur_node.clone());
    }

    for (i, block) in added_blocks.iter().enumerate() {
        assert_eq!(block_store.len(), 100 - i);
        assert_eq!(block_store.pruned_blocks_in_mem(), min(i, 10));
        block_store.prune_tree(block.id());
    }
}

#[test]
fn test_path_from_root() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.get_block(block_store.root().id()).unwrap();
    let b1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let b2 = inserter.insert_block(&b1, 2, None);
    let b3 = inserter.insert_block(&b2, 3, None);

    assert_eq!(
        block_store.path_from_root(b3.id()),
        Some(vec![b1, b2.clone(), b3.clone()])
    );
    assert_eq!(block_store.path_from_root(genesis.id()), Some(vec![]));

    block_store.prune_tree(b2.id());

    assert_eq!(block_store.path_from_root(b3.id()), Some(vec![b3.clone()]));
    assert_eq!(block_store.path_from_root(genesis.id()), None);
}

#[test]
fn test_insert_vote() {
    ::libra_logger::Logger::new().environment_only(true).init();
    // Set up enough different authors to support different votes for the same block.
    let (signers, validator_verifier) = random_validator_verifier(11, Some(10), false);
    let my_signer = signers[10].clone();
    let mut inserter = TreeInserter::new(my_signer);
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let block = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let mut pending_votes = PendingVotes::new();

    assert!(block_store.get_quorum_cert_for_block(block.id()).is_none());
    for (i, voter) in signers.iter().enumerate().take(10).skip(1) {
        let vote = Vote::new(
            VoteData::new(
                block.block().gen_block_info(
                    block.compute_result().root_hash(),
                    block.compute_result().version(),
                    block.compute_result().validators().clone(),
                ),
                block.quorum_cert().certified_block().clone(),
            ),
            voter.author(),
            placeholder_ledger_info(),
            voter,
        );
        let vote_res = pending_votes.insert_vote(&vote, &validator_verifier);

        // first vote of an author is accepted
        assert_eq!(vote_res, VoteReceptionResult::VoteAdded(i as u64));
        // filter out duplicates
        assert_eq!(
            pending_votes.insert_vote(&vote, &validator_verifier),
            VoteReceptionResult::DuplicateVote,
        );
        // qc is still not there
        assert!(block_store.get_quorum_cert_for_block(block.id()).is_none());
    }

    // Add the final vote to form a QC
    let final_voter = &signers[0];
    let vote = Vote::new(
        VoteData::new(
            block.block().gen_block_info(
                block.compute_result().root_hash(),
                block.compute_result().version(),
                block.compute_result().validators().clone(),
            ),
            block.quorum_cert().certified_block().clone(),
        ),
        final_voter.author(),
        placeholder_ledger_info(),
        final_voter,
    );
    match pending_votes.insert_vote(&vote, &validator_verifier) {
        VoteReceptionResult::NewQuorumCertificate(qc) => {
            assert_eq!(qc.certified_block().id(), block.id());
            block_store
                .insert_single_quorum_cert(qc.as_ref().clone())
                .unwrap();
        }
        _ => {
            panic!("QC not formed!");
        }
    }

    let block_qc = block_store.get_quorum_cert_for_block(block.id()).unwrap();
    assert_eq!(block_qc.certified_block().id(), block.id());
}

#[test]
fn test_illegal_timestamp() {
    let signer = ValidatorSigner::random(None);
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let block_with_illegal_timestamp = Block::<Vec<usize>>::new_proposal(
        vec![],
        0,
        // This timestamp is illegal, it is the same as genesis
        genesis.timestamp_usecs(),
        certificate_for_genesis(),
        &signer,
    );
    let result = block_store.execute_and_insert_block(block_with_illegal_timestamp);
    assert!(result.is_err());
}

#[test]
fn test_highest_qc() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();

    // build a tree of the following form
    // genesis <- a1 <- a2 <- a3
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    assert_eq!(block_store.highest_certified_block(), genesis);
    let a2 = inserter.insert_block(&a1, 2, None);
    assert_eq!(block_store.highest_certified_block(), a1);
    let _a3 = inserter.insert_block(&a2, 3, None);
    assert_eq!(block_store.highest_certified_block(), a2);
}

#[test]
fn test_need_fetch_for_qc() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();

    // build a tree of the following form
    // genesis <- a1 <- a2 <- a3
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_block(&a2, 3, None);
    block_store.prune_tree(a2.id());
    let need_fetch_qc = placeholder_certificate_for_block(
        vec![inserter.signer()],
        HashValue::zero(),
        a3.round() + 1,
        HashValue::zero(),
        a3.round(),
    );
    let too_old_qc = certificate_for_genesis();
    let can_insert_qc = placeholder_certificate_for_block(
        vec![inserter.signer()],
        a3.id(),
        a3.round(),
        a2.id(),
        a2.round(),
    );
    let duplicate_qc = block_store.get_quorum_cert_for_block(a2.id()).unwrap();
    assert_eq!(
        block_store.need_fetch_for_quorum_cert(&need_fetch_qc),
        NeedFetchResult::NeedFetch
    );
    assert_eq!(
        block_store.need_fetch_for_quorum_cert(&too_old_qc),
        NeedFetchResult::QCRoundBeforeRoot,
    );
    assert_eq!(
        block_store.need_fetch_for_quorum_cert(&can_insert_qc),
        NeedFetchResult::QCBlockExist,
    );
    assert_eq!(
        block_store.need_fetch_for_quorum_cert(duplicate_qc.as_ref()),
        NeedFetchResult::QCAlreadyExist,
    );
}

#[test]
fn test_empty_reconfiguration_suffix() {
    let mut inserter = TreeInserter::default();
    let block_store = inserter.block_store();
    let genesis = block_store.root();
    let a1 = inserter.insert_block_with_qc(certificate_for_genesis(), &genesis, 1);
    let a2 = inserter.insert_block(&a1, 2, None);
    let a3 = inserter.insert_reconfiguration_block(&a2, 3);
    let a4 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a3.as_ref(), None),
        a3.as_ref().timestamp_usecs(),
        4,
        vec![42],
    );
    // Child of reconfiguration carries a payload will fail to insert
    assert!(a4.verify_well_formed().is_err());
    let a5 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a3.as_ref(), None),
        a3.as_ref().timestamp_usecs(),
        4,
        vec![],
    );
    // Child of reconfiguration doesn't carry payload will succeed and roll over the validator set
    let a5 = block_store.execute_and_insert_block(a5).unwrap();
    assert!(a5.compute_result().has_reconfiguration());
    // Block continues another branch can carry payload
    inserter.insert_block(&a2, 4, None);
    block_store.prune_tree(a3.id());
    // If reconfiguration is committed, the child block will fail to insert
    let a6 = inserter.create_block_with_qc(
        inserter.create_qc_for_block(a5.as_ref(), None),
        a5.as_ref().timestamp_usecs(),
        5,
        vec![42],
    );
    assert!(a6.verify_well_formed().is_err());
}
