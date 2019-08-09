// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    block_storage::{BlockReader, BlockStore, InsertError, NeedFetchResult, VoteReceptionResult},
    common::Author,
    consensus_types::{
        block::{block_test, Block},
        quorum_cert::QuorumCert,
    },
    safety::vote_msg::VoteMsg,
    test_utils::{
        build_empty_tree, build_empty_tree_with_custom_signing, placeholder_certificate_for_block,
        placeholder_ledger_info, TreeInserter,
    },
};
use crypto::HashValue;
use futures::executor::block_on;
use nextgen_crypto::{ed25519::*, *};
use proptest::prelude::*;
use std::{cmp::min, collections::HashSet, sync::Arc};
use types::{account_address::AccountAddress, validator_signer::ValidatorSigner};

fn build_simple_tree() -> (Vec<Arc<Block<Vec<usize>>>>, Arc<BlockStore<Vec<usize>>>) {
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let genesis_block_id = genesis.id();
    let genesis_block = block_store
        .get_block(genesis_block_id)
        .expect("genesis block must exist");
    assert_eq!(block_store.len(), 1);
    assert_eq!(block_store.child_links(), block_store.len() - 1);
    assert_eq!(block_store.block_exists(genesis_block.id()), true);

    //       | -> A1 -> A2 -> A3
    // Genesis -> B1 -> B2
    //             | -> C1
    let mut inserter = TreeInserter::new(block_store.clone());
    let a1 = inserter.insert_block(genesis_block.as_ref(), 1);
    let a2 = inserter.insert_block(a1.as_ref(), 2);
    let a3 = inserter.insert_block(a2.as_ref(), 3);
    let b1 = inserter.insert_block(genesis_block.as_ref(), 4);
    let b2 = inserter.insert_block(b1.as_ref(), 5);
    let c1 = inserter.insert_block(b1.as_ref(), 6);

    assert_eq!(block_store.len(), 7);
    assert_eq!(block_store.child_links(), block_store.len() - 1);

    (vec![genesis_block, a1, a2, a3, b1, b2, c1], block_store)
}

#[test]
fn test_block_store_create_block() {
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let a1 = block_store.create_block(Arc::clone(&genesis), vec![1], 1, 1);
    assert_eq!(a1.parent_id(), genesis.id());
    assert_eq!(a1.round(), 1);
    assert_eq!(a1.height(), 1);
    assert_eq!(a1.quorum_cert().certified_block_id(), genesis.id());

    let a1_ref = block_on(block_store.execute_and_insert_block(a1)).unwrap();

    // certify a1
    let vote_msg = VoteMsg::new(
        a1_ref.id(),
        block_store.get_state_for_block(a1_ref.id()).unwrap(),
        a1_ref.round(),
        a1_ref.quorum_cert().certified_parent_block_id(),
        a1_ref.quorum_cert().certified_parent_block_round(),
        a1_ref.quorum_cert().certified_grandparent_block_id(),
        a1_ref.quorum_cert().certified_grandparent_block_round(),
        block_store.signer().author(),
        placeholder_ledger_info(),
        block_store.signer(),
    );
    block_store.insert_vote_and_qc(vote_msg, 1);

    let b1 = block_store.create_block(Arc::clone(&a1_ref), vec![2], 2, 2);
    assert_eq!(b1.parent_id(), a1_ref.id());
    assert_eq!(b1.round(), 2);
    assert_eq!(b1.height(), 2);
    assert_eq!(b1.quorum_cert().certified_block_id(), a1_ref.id());
}

#[test]
fn test_highest_block_and_quorum_cert() {
    let block_store = build_empty_tree();
    assert_eq!(
        block_store.highest_certified_block().as_ref(),
        &Block::make_genesis_block()
    );
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        &QuorumCert::certificate_for_genesis()
    );

    let genesis = block_store.root();
    let mut inserter = TreeInserter::new(block_store.clone());

    // Genesis block and quorum certificate is still the highest
    let block_round_1 = inserter.insert_block(genesis.as_ref(), 1);
    assert_eq!(
        block_store.highest_certified_block().as_ref(),
        &Block::make_genesis_block()
    );
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        &QuorumCert::certificate_for_genesis()
    );

    // block_round_1 block and quorum certificate is now the highest
    let block_round_3 = inserter.insert_block(block_round_1.as_ref(), 3);
    assert_eq!(
        block_store.highest_certified_block().as_ref(),
        block_round_1.as_ref()
    );
    assert_eq!(
        block_store.highest_quorum_cert().as_ref(),
        block_store
            .get_block(block_round_3.id())
            .expect("block_round_1 should exist")
            .quorum_cert()
    );

    // block_round_1 block and quorum certificate is still the highest, since block_round_4
    // also builds on block_round_1
    let block_round_4 = inserter.insert_block(block_round_1.as_ref(), 4);
    assert_eq!(
        block_store.highest_certified_block().as_ref(),
        block_round_1.as_ref()
    );
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
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let mut inserter = TreeInserter::new(block_store.clone());
    let block_a_1 = inserter.insert_block(genesis.as_ref(), 1);
    let block_a_2 = inserter.insert_block(block_a_1.as_ref(), 2);

    assert_eq!(
        block_store.get_block(genesis.quorum_cert().certified_block_id()),
        None
    );
    assert_eq!(
        block_store.get_block(block_a_1.quorum_cert().certified_block_id()),
        Some(genesis)
    );
    assert_eq!(
        block_store.get_block(block_a_2.quorum_cert().certified_block_id()),
        Some(block_a_1)
    );
}

// This test should be continuously extended to eventually become the
// single-page spec for the logic of our block storage.
proptest! {

    #[test]
    fn test_block_store_insert(
        (mut private_keys, blocks) in block_test::block_forest_and_its_keys(
            // quorum size
            10,
            // recursion depth
            50)
    ){
        let authors: HashSet<Author> = private_keys.iter().map(|private_key| AccountAddress::from_public_key(&private_key.public_key())).collect();
        let priv_key = private_keys.pop().expect("several keypairs generated");
        let signer = ValidatorSigner::new(None, priv_key);
        let block_store = build_empty_tree_with_custom_signing(signer);
        for block in blocks {
            if block.round() > 0 && authors.contains(&block.author().unwrap()) {
                let known_parent = block_store.block_exists(block.parent_id());
                let certified_parent = block.quorum_cert().certified_block_id() == block.parent_id();
                let res = block_on(block_store.execute_and_insert_block(block.clone()));
                if !certified_parent {
                    prop_assert_eq!(res.err(), Some(InsertError::ParentNotCertified));
                } else if !known_parent {
                    // We cannot really bring blocks in this test because the block retrieval
                    // functionality invokes event processing, which is not setup here.
                    assert!(res.is_err());
                }
                else {
                    // The parent must be present if we get to this line.
                    let parent = block_store.get_block(block.parent_id()).unwrap();
                    if block.height() != parent.height() + 1 {
                        prop_assert_eq!(res.err(), Some(InsertError::InvalidBlockHeight));
                    } else if block.round() <= parent.round() {
                        prop_assert_eq!(res.err(), Some(InsertError::InvalidBlockRound));
                    } else {
                        prop_assert_eq!(res.clone().ok(),
                            Some(Arc::new(block.clone())),
                            "expected ok on block: {:#?}, got {:#?}", block, res);
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
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let mut cur_node = block_store.get_block(genesis.id()).unwrap();
    let mut added_blocks = vec![];

    let mut inserter = TreeInserter::new(block_store.clone());
    for round in 1..100 {
        cur_node = inserter.insert_block(cur_node.as_ref(), round);
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
    let block_store = build_empty_tree();
    let genesis = block_store.get_block(block_store.root().id()).unwrap();
    let mut inserter = TreeInserter::new(block_store.clone());
    let b1 = inserter.insert_block(genesis.as_ref(), 1);
    let b2 = inserter.insert_block(b1.as_ref(), 2);
    let b3 = inserter.insert_block(b2.as_ref(), 3);

    assert_eq!(
        block_store.path_from_root(b3.clone()),
        Some(vec![b3.clone(), b2.clone(), b1.clone()])
    );
    assert_eq!(block_store.path_from_root(genesis.clone()), Some(vec![]));

    block_store.prune_tree(b2.id());

    assert_eq!(
        block_store.path_from_root(b3.clone()),
        Some(vec![b3.clone()])
    );
    assert_eq!(block_store.path_from_root(genesis.clone()), None);
}

#[test]
fn test_insert_vote() {
    // Set up enough different authors to support different votes for the same block.
    let qc_size = 10;
    let mut signers = vec![];
    let mut author_public_keys = vec![];

    for i in 0..qc_size {
        let signer = ValidatorSigner::<Ed25519PrivateKey>::random([i as u8; 32]);
        author_public_keys.push((
            AccountAddress::from_public_key(&signer.public_key()),
            signer.public_key(),
        ));
        signers.push(signer);
    }
    let my_signer = ValidatorSigner::random([qc_size as u8; 32]);
    author_public_keys.push((
        AccountAddress::from_public_key(&my_signer.public_key()),
        my_signer.public_key(),
    ));
    let block_store = build_empty_tree_with_custom_signing(my_signer);
    let genesis = block_store.root();
    let mut inserter = TreeInserter::new(block_store.clone());
    let block = inserter.insert_block(genesis.as_ref(), 1);

    assert!(block_store.get_quorum_cert_for_block(block.id()).is_none());
    let qc_size = 10;
    for (i, voter) in signers.iter().enumerate().take(10).skip(1) {
        let vote_msg = VoteMsg::new(
            block.id(),
            block_store.get_state_for_block(block.id()).unwrap(),
            block.round(),
            block.quorum_cert().certified_parent_block_id(),
            block.quorum_cert().certified_parent_block_round(),
            block.quorum_cert().certified_grandparent_block_id(),
            block.quorum_cert().certified_grandparent_block_round(),
            voter.author(),
            placeholder_ledger_info(),
            voter,
        );
        let vote_res = block_store.insert_vote_and_qc(vote_msg.clone(), qc_size);

        // first vote of an author is accepted
        assert_eq!(vote_res, VoteReceptionResult::VoteAdded(i));
        // filter out duplicates
        assert_eq!(
            block_store.insert_vote_and_qc(vote_msg, qc_size),
            VoteReceptionResult::DuplicateVote,
        );
        // qc is still not there
        assert!(block_store.get_quorum_cert_for_block(block.id()).is_none());
    }

    // Add the final vote to form a QC
    let final_voter = &signers[0];
    let vote_msg = VoteMsg::new(
        block.id(),
        block_store.get_state_for_block(block.id()).unwrap(),
        block.round(),
        block.quorum_cert().certified_parent_block_id(),
        block.quorum_cert().certified_parent_block_round(),
        block.quorum_cert().certified_grandparent_block_id(),
        block.quorum_cert().certified_grandparent_block_round(),
        final_voter.author(),
        placeholder_ledger_info(),
        final_voter,
    );
    match block_store.insert_vote_and_qc(vote_msg, qc_size) {
        VoteReceptionResult::NewQuorumCertificate(qc) => {
            assert_eq!(qc.certified_block_id(), block.id());
        }
        _ => {
            panic!("QC not formed!");
        }
    }

    let block_qc = block_store.get_quorum_cert_for_block(block.id()).unwrap();
    assert_eq!(block_qc.certified_block_id(), block.id());
}

#[test]
fn test_illegal_timestamp() {
    let block_store = build_empty_tree();
    let genesis = block_store.root();
    let block_with_illegal_timestamp = Block::<Vec<usize>>::new_internal(
        vec![],
        genesis.id(),
        1,
        1,
        // This timestamp is illegal, it is the same as genesis
        genesis.timestamp_usecs(),
        QuorumCert::certificate_for_genesis(),
        block_store.signer(),
    );
    let result = block_on(block_store.execute_and_insert_block(block_with_illegal_timestamp));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), InsertError::NonIncreasingTimestamp);
}

#[test]
fn test_highest_qc() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());

    // build a tree of the following form
    // genesis <- a1 <- a2 <- a3
    let genesis = block_tree.root();
    let a1 = inserter.insert_block(genesis.as_ref(), 1);
    assert_eq!(block_tree.highest_certified_block(), genesis.clone());
    let a2 = inserter.insert_block(a1.as_ref(), 2);
    assert_eq!(block_tree.highest_certified_block(), a1.clone());
    let _a3 = inserter.insert_block(a2.as_ref(), 3);
    assert_eq!(block_tree.highest_certified_block(), a2.clone());
}

#[test]
fn test_need_fetch_for_qc() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());

    // build a tree of the following form
    // genesis <- a1 <- a2 <- a3
    let genesis = block_tree.root();
    let a1 = inserter.insert_block(genesis.as_ref(), 1);
    let a2 = inserter.insert_block(a1.as_ref(), 2);
    let a3 = inserter.insert_block(a2.as_ref(), 3);
    block_tree.prune_tree(a2.id());
    let need_fetch_qc = placeholder_certificate_for_block(
        vec![block_tree.signer()],
        HashValue::zero(),
        a3.round() + 1,
        HashValue::zero(),
        a3.round(),
        HashValue::zero(),
        a3.round() - 1,
    );
    let too_old_qc = QuorumCert::certificate_for_genesis();
    let can_insert_qc = placeholder_certificate_for_block(
        vec![block_tree.signer()],
        a3.id(),
        a3.round(),
        a2.id(),
        a2.round(),
        a1.id(),
        a1.round(),
    );
    let duplicate_qc = block_tree.get_quorum_cert_for_block(a2.id()).unwrap();
    assert_eq!(
        block_tree.need_fetch_for_quorum_cert(&need_fetch_qc),
        NeedFetchResult::NeedFetch
    );
    assert_eq!(
        block_tree.need_fetch_for_quorum_cert(&too_old_qc),
        NeedFetchResult::QCRoundBeforeRoot,
    );
    assert_eq!(
        block_tree.need_fetch_for_quorum_cert(&can_insert_qc),
        NeedFetchResult::QCBlockExist,
    );
    assert_eq!(
        block_tree.need_fetch_for_quorum_cert(duplicate_qc.as_ref()),
        NeedFetchResult::QCAlreadyExist,
    );
}

#[test]
fn test_need_sync_for_qc() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());

    // build a tree of the following form
    // genesis <- a1 <- a2 <- a3
    let genesis = block_tree.root();
    let a1 = inserter.insert_block(genesis.as_ref(), 1);
    let a2 = inserter.insert_block(a1.as_ref(), 2);
    let a3 = inserter.insert_block(a2.as_ref(), 3);
    block_tree.prune_tree(a3.id());
    let qc = placeholder_certificate_for_block(
        vec![block_tree.signer()],
        HashValue::zero(),
        a3.round() + 3,
        HashValue::zero(),
        a3.round() + 2,
        HashValue::zero(),
        a3.round() + 1,
    );
    assert_eq!(
        block_tree.need_sync_for_quorum_cert(HashValue::zero(), &qc),
        true
    );
    let qc = placeholder_certificate_for_block(
        vec![block_tree.signer()],
        HashValue::zero(),
        a3.round() + 2,
        HashValue::zero(),
        a3.round() + 1,
        HashValue::zero(),
        a3.round(),
    );
    assert_eq!(
        block_tree.need_sync_for_quorum_cert(HashValue::zero(), &qc),
        false,
    );
    assert_eq!(
        block_tree.need_sync_for_quorum_cert(genesis.id(), &QuorumCert::certificate_for_genesis()),
        false
    );
}
