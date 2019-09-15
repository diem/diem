// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::chained_bft::{
    block_storage::BlockReader,
    common::Round,
    consensus_types::{
        block::{block_test, Block},
        quorum_cert::QuorumCert,
    },
    safety::safety_rules::{ConsensusState, ProposalReject, SafetyRules},
    test_utils::{build_empty_tree, build_empty_tree_with_custom_signing, TreeInserter},
};
use cached::{cached_key, SizedCache};
use crypto::HashValue;
use proptest::prelude::*;
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
    sync::Arc,
};
use types::validator_signer::ValidatorSigner;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

cached_key! {
    // We memoize the dfs of max_chain_depth. The size limits reflects that if we call more
    // times than the forest size or so, we probably have changed the
    // map's state.
    LENGTH: SizedCache<String, (usize, Vec<HashValue>)> = SizedCache::with_size(50);
    Key = { format!("{}{:?}{:?}", calculate_hash(children_table), query, initial_contiguous_links ) };
    // This returns the length of the maximal chain constructible from the
    // (block_id, block_round) node, along with an example of such a chain
    // (they are not unique)
    fn max_chain_depth(children_table: &BTreeMap<HashValue, Vec<(HashValue, Round)>>, query: (HashValue, Round), initial_contiguous_links: i64) -> (usize, Vec<HashValue>) = {
        if let Some(children) = children_table.get(&query.0) {
            if let Some((depth, mut subchain)) = children.iter().cloned().flat_map(|(child_id, child_round)|
                                          if initial_contiguous_links > 0 && child_round > query.1 + 1 {
                                              // we're asked for a contiguous chain link and can't deliver on this child
                                              None
                                          } else {
                                              Some(max_chain_depth(children_table, (child_id, child_round), std::cmp::max(initial_contiguous_links - 1, 0)))
                                          }).max_by(|x, y| x.0.cmp(&y.0)) {
                subchain.push(query.0);
                (depth + 1, subchain)
            } else {
                (0, vec![query.0])
            }
        } else {
            (0, vec![query.0])
        }
    }
}

proptest! {
    #[test]
    fn test_blocks_commits_safety_rules(
        (mut keypairs, blocks) in block_test::block_forest_and_its_keys(
            // quorum size
            10,
            // recursion depth
            50)
    ) {
        let first_key = keypairs.pop().expect("several keys");
        let first_signer = ValidatorSigner::new(None, first_key);
        let mut qc_signers = vec![first_signer.clone()];

        for priv_key in keypairs {
            let signer = ValidatorSigner::new(None, priv_key);
            qc_signers.push(signer);
        }

        let block_tree = build_empty_tree_with_custom_signing(first_signer.clone());
        let mut inserter = TreeInserter::new(block_tree.clone());
        let mut safety_rules = SafetyRules::new(ConsensusState::default());

        // This commit_candidate tracks the commit that would get
        // committed if the current block would get a QC
        let mut commit_candidate = block_tree.root().id();

        // children_table contains a map from parent block id to
        // [(block_id, block_round), ...] of its children
        let mut children_table = BTreeMap::new();

        // inserted contains the blocks newly inserted in the tree
        let mut inserted = Vec::new();

        for block in blocks {
            let known_parent = block_tree.block_exists(block.parent_id());
            if !known_parent {
                continue;
            }

            let insert_res = inserter.insert_pre_made_block(block.clone(), &first_signer, qc_signers.iter().collect());
            let id_and_qc = |ref block: Arc<Block<Vec<usize>>>| { (block.id(), block.quorum_cert().clone()) };
            let (inserted_id, inserted_qc) = id_and_qc(insert_res.clone());
            safety_rules.update(&inserted_qc);

            let siblings = children_table.entry(block.parent_id()).or_insert_with(|| vec![]);
            siblings.push((inserted_id, block.round()));

            inserted.push((inserted_id, block.round()));

            let long_chained_blocks: Vec<(&HashValue, &Round, usize, Vec<HashValue>)> = inserted.iter().map(|(b_id, b_round)| {
                let (chain_depth, chain) = max_chain_depth(&children_table, (*b_id, *b_round), 0);
                (b_id, b_round, chain_depth, chain)
            }).collect();

            // The preferred_block is the latest (highest round-wise) 2-chain
            let preferred_block_round = safety_rules.consensus_state().preferred_block_round;
            let highest_two_chain = long_chained_blocks.clone().iter()
                .filter(|(_bid, _bround, chain_depth, _chain)| *chain_depth >= 2)
                // highest = max by round
                .max_by(|b1, b2| (*b1.1).cmp(b2.1))
                .map_or((block_tree.root().id(), 0, vec![]), |(bid, bround, _, chain)| (**bid, **bround, chain.to_vec()));
            prop_assert_eq!(highest_two_chain.1, preferred_block_round,
                            "Preferred block mismatch, expected {:?} because of chain {:#?}\n", highest_two_chain.0, highest_two_chain.2);

            let long_contiguous_chained_blocks: Vec<(&HashValue, &Round, usize, Vec<HashValue>)> = inserted.iter().map(|(b_id, b_round)| {
                // We ask for 2 contiguous initial rounds this time
                let (chain_depth, chain) = max_chain_depth(&children_table, (*b_id, *b_round), 2);
                (b_id, b_round, chain_depth, chain)
            }).collect();

            let highest_contiguous_3_chain_prefix = long_contiguous_chained_blocks.iter()
                // We have a chain of 3 blocks (two links) which are contiguous
                .filter(|(_bid, _bround, chain_depth, _chain)| *chain_depth == 2)
                // max by round
                .max_by(|b1, b2| (*b1.1).cmp(b2.1))
                .map_or((block_tree.root().id(), 0, vec![]), |(bid, bround, _, chain)| (**bid, **bround, chain.to_vec()));

            if highest_contiguous_3_chain_prefix.0 != commit_candidate {
                // We have a potential change of commit candidate ->
                // the current block can be voted on and if gathered a
                // QC, would trigger a different commit
                let block_arc = block_tree.get_block(inserted_id).expect("we just inserted this");
                let vote_info = safety_rules.voting_rule(&block_arc).and_then(|x| Ok(x.potential_commit_id()));
                prop_assert_eq!(vote_info, Ok(Some(highest_contiguous_3_chain_prefix.0)),
                                "Commit mismatch: expected committing {:?} upon hearing about {:?} with preferred block {:?} because of chain {:#?}\n", highest_contiguous_3_chain_prefix.0, block.id(), highest_two_chain.0, highest_contiguous_3_chain_prefix.2
                );
                commit_candidate = highest_contiguous_3_chain_prefix.0;
            }


        }

    }
}

#[test]
fn test_initial_state() {
    // Start from scratch, verify the state
    let block_tree = build_empty_tree();

    let safety_rules = SafetyRules::new(ConsensusState::default());
    let state = safety_rules.consensus_state();
    assert_eq!(state.last_vote_round(), 0);
    assert_eq!(state.preferred_block_round(), block_tree.root().round());
}

#[test]
fn test_preferred_block_rule() {
    // Preferred block is the highest 2-chain head.
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

    // build a tree of the following form:
    //             _____    _____
    //            /     \  /     \
    // genesis---a1  b1  b2  a2  b3  a3---a4
    //         \_____/ \_____/ \_____/
    //
    // PB should change from genesis to b1, and then to a2.
    let genesis = block_tree.root();
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 2);
    let b2 = inserter.insert_block(a1.as_ref(), 3);
    let a2 = inserter.insert_block(b1.as_ref(), 4);
    let b3 = inserter.insert_block(b2.as_ref(), 5);
    let a3 = inserter.insert_block(a2.as_ref(), 6);
    let a4 = inserter.insert_block(a3.as_ref(), 7);

    safety_rules.update(a1.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis.round()
    );

    safety_rules.update(b1.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis.round()
    );

    safety_rules.update(a2.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis.round()
    );

    safety_rules.update(b2.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        genesis.round()
    );

    safety_rules.update(a3.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.round()
    );

    safety_rules.update(b3.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        b1.round()
    );

    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.consensus_state().preferred_block_round(),
        a2.round()
    );
}

#[test]
fn test_voting() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

    // build a tree of the following form:
    //             _____    __________
    //            /     \  /          \
    // genesis---a1  b1  b2  a2---a3  b3  a4  b4
    //         \_____/ \_____/     \______/   /
    //                    \__________________/
    //
    //
    // We'll introduce the votes in the following order:
    // a1 (ok), potential_commit is None
    // b1 (ok), potential commit is None
    // a2 (ok), potential_commit is None
    // b2 (old proposal)
    // a3 (ok), potential commit is None
    // b3 (ok), potential commit is None
    // a4 (ok), potential commit is None
    // a4 (old proposal)
    // b4 (round lower then round of pb. PB: a2, parent(b4)=b2)
    let genesis = block_tree.root();
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 2);
    let b2 = inserter.insert_block(a1.as_ref(), 3);
    let a2 = inserter.insert_block(b1.as_ref(), 4);
    let a3 = inserter.insert_block(a2.as_ref(), 5);
    let b3 = inserter.insert_block(b2.as_ref(), 6);
    let a4 = inserter.insert_block(a3.as_ref(), 7);
    let b4 = inserter.insert_block(b2.as_ref(), 8);

    safety_rules.update(a1.quorum_cert());
    let mut voting_info = safety_rules.voting_rule(&a1).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b1.quorum_cert());
    voting_info = safety_rules.voting_rule(&b1).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a2.quorum_cert());
    voting_info = safety_rules.voting_rule(&a2).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b2.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&b2),
        Err(ProposalReject::OldProposal {
            last_vote_round: 4,
            proposal_round: 3,
        })
    );

    safety_rules.update(a3.quorum_cert());
    voting_info = safety_rules.voting_rule(&a3).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(b3.quorum_cert());
    voting_info = safety_rules.voting_rule(&b3).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a4.quorum_cert());
    voting_info = safety_rules.voting_rule(&a4).unwrap();
    assert_eq!(voting_info.potential_commit_id, None);

    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a4),
        Err(ProposalReject::OldProposal {
            last_vote_round: 7,
            proposal_round: 7,
        })
    );
    safety_rules.update(b4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&b4),
        Err(ProposalReject::ProposalRoundLowerThenPreferredBlock {
            preferred_block_round: 4,
        })
    );
}

#[test]
/// Test the potential ledger info that we're going to use in case of voting
fn test_voting_potential_commit_id() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());
    let mut safety_rules = SafetyRules::new(ConsensusState::default());

    // build a tree of the following form:
    //            _____
    //           /     \
    // genesis--a1  b1  a2--a3--a4--a5
    //        \_____/
    //
    // All the votes before a4 cannot produce any potential commits.
    // A potential commit for proposal a4 is a2, a potential commit for proposal a5 is a3.

    let genesis = block_tree.root();
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 2);
    let a2 = inserter.insert_block(a1.as_ref(), 3);
    let a3 = inserter.insert_block(a2.as_ref(), 4);
    let a4 = inserter.insert_block(a3.as_ref(), 5);
    let a5 = inserter.insert_block(a4.as_ref(), 6);

    let vec_with_no_potential_commits = vec![a1.clone(), b1.clone(), a2.clone(), a3.clone()];
    for b in vec_with_no_potential_commits {
        safety_rules.update(b.quorum_cert());
        let voting_info = safety_rules.voting_rule(&b).unwrap();
        assert_eq!(voting_info.potential_commit_id, None);
    }
    safety_rules.update(a4.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a4).unwrap().potential_commit_id,
        Some(a2.id())
    );
    safety_rules.update(a5.quorum_cert());
    assert_eq!(
        safety_rules.voting_rule(&a5).unwrap().potential_commit_id,
        Some(a3.id())
    );
}

#[test]
fn test_commit_rule_consecutive_rounds() {
    let block_tree = build_empty_tree();
    let mut inserter = TreeInserter::new(block_tree.clone());
    let safety_rules = SafetyRules::new(ConsensusState::default());

    // build a tree of the following form:
    //             ___________
    //            /           \
    // genesis---a1  b1---b2   a2---a3---a4
    //         \_____/
    //
    // a1 cannot be committed after a3 gathers QC because a1 and a2 are not consecutive
    // a2 can be committed after a4 gathers QC

    let genesis = block_tree.root();
    let a1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 1);
    let b1 =
        inserter.insert_block_with_qc(QuorumCert::certificate_for_genesis(), genesis.as_ref(), 2);
    let b2 = inserter.insert_block(b1.as_ref(), 3);
    let a2 = inserter.insert_block(a1.as_ref(), 4);
    let a3 = inserter.insert_block(a2.as_ref(), 5);
    let a4 = inserter.insert_block(a3.as_ref(), 6);

    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a1.quorum_cert(), a1.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(b1.quorum_cert(), b1.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(b2.quorum_cert(), b2.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a2.quorum_cert(), a2.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a3.quorum_cert(), a3.round()),
        None
    );
    assert_eq!(
        safety_rules.commit_rule_for_certified_block(a4.quorum_cert(), a4.round()),
        Some(a2.id())
    );
}
