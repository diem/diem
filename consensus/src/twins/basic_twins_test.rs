// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network_interface::ConsensusMsg,
    network_tests::{NetworkPlayground, TwinId},
    test_utils::{consensus_runtime, timed_block_on},
    twins::twins_node::SMRNode,
};
use consensus_types::{block::Block, common::Round};
use diem_config::config::ConsensusProposerType::{FixedProposer, RotatingProposer, RoundProposer};
use futures::StreamExt;
use std::collections::HashMap;

#[test]
/// This test checks that the first proposal has its parent and
/// QC pointing to the genesis block.
///
/// Setup:
///
/// 4 honest nodes, and 0 twins
///
/// Run the test:
/// cargo xtest -p consensus basic_start_test -- --nocapture
fn basic_start_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 0;
    let nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RotatingProposer,
        None,
    );
    let genesis = Block::make_genesis_block_from_ledger_info(&nodes[0].storage.get_ledger_info());
    timed_block_on(&mut runtime, async {
        let msg = playground
            .wait_for_messages(1, NetworkPlayground::proposals_only)
            .await;
        let first_proposal = match &msg[0].1 {
            ConsensusMsg::ProposalMsg(proposal) => proposal,
            _ => panic!("Unexpected message found"),
        };
        assert_eq!(first_proposal.proposal().parent_id(), genesis.id());
        assert_eq!(
            first_proposal
                .proposal()
                .quorum_cert()
                .certified_block()
                .id(),
            genesis.id()
        );
    });
}

#[test]
/// This test checks that the split_network function works
/// as expected, that is: nodes in a partition with less nodes
/// than required for quorum do not commit anything.
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 0 twins.
/// Create two partitions p1=[n2], and p2=[n0, n1, n3] with
/// a proposer (n0) in p2.
///
/// Test:
///
/// Run consensus for enough rounds to potentially form a commit.
/// Check that n1 has no commits, and n0 has commits.
///
/// Run the test:
/// cargo xtest -p consensus drop_config_test -- --nocapture
#[ignore] // TODO: https://github.com/diem/diem/issues/8767
fn drop_config_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 0;
    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        FixedProposer,
        None,
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    let n1_twin_id = nodes[1].id;
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    assert!(playground.split_network(vec![n2_twin_id], vec![n0_twin_id, n1_twin_id, n3_twin_id]));
    runtime.spawn(playground.start());

    timed_block_on(&mut runtime, async {
        // Check that the commit log for n0 is not empty
        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        assert!(node0_commit.is_some());

        // Check that the commit log for n2 is empty
        let node2_commit = match nodes[2].commit_cb_receiver.try_next() {
            Ok(Some(node_commit)) => Some(node_commit),
            _ => None,
        };
        assert!(node2_commit.is_none());
    });
}

#[test]
/// This test checks that the vote of a node and its twin
/// should be counted as duplicate vote (because they have
/// the same public keys)
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 1 twin (twin0)
/// Create 2 partitions, p1=[n1, n3], p2=[n0, twin0, n2]
///
/// Test:
///
/// Extract enough votes to potentially form commits. Check
/// that no node commits any block. This is because we need
/// 3 nodes to form a quorum and no partition has enough votes
/// (note there are 3 nodes in p2, but one of them is a twin,
/// and its vote will be counted as duplicate of n0).
///
/// Run the test:
/// cargo xtest -p consensus twins_vote_dedup_test -- --nocapture
fn twins_vote_dedup_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 1;
    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RotatingProposer,
        None,
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    // twin of n0 has same author as node[0]
    let twin0_twin_id = nodes[4].id;
    assert_eq!(n0_twin_id.author, twin0_twin_id.author);
    let n1_twin_id = nodes[1].id;
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    assert!(playground.split_network(
        vec![n1_twin_id, n3_twin_id],
        vec![twin0_twin_id, n0_twin_id, n2_twin_id],
    ));
    runtime.spawn(playground.start());

    timed_block_on(&mut runtime, async {
        // No node should be able to commit because of the way partitions
        // have been created
        let mut commit_seen = false;
        for node in &mut nodes {
            if let Ok(Some(_node_commit)) = node.commit_cb_receiver.try_next() {
                commit_seen = true;
            }
        }
        assert!(!commit_seen);
    });
}

#[test]
/// This test checks that when a node becomes a proposer, its
/// twin becomes one too.
///
/// Setup:
///
/// 4 honest nodes (n0, n1, n2, n3), and 2 twins (twin0, twin1)
/// Create 2 partitions, p1=[n0, n1, n2], p2=[n3, twin0, twin1]
/// Let n0 (and implicitly twin0) be proposers
///
/// Test:
///
/// Extract enough votes so nodes in both partitions form commits.
/// The commits should be on two different blocks
///
/// Run the test:
/// cargo xtest -p consensus twins_proposer_test -- --nocapture
fn twins_proposer_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 2;

    // Specify round leaders
    // Will default to the first node, if no leader specified for given round
    let mut round_proposers: HashMap<Round, usize> = HashMap::new();
    // Leaders are n0 (and implicitly twin0) for round 1..10
    for i in 1..10 {
        round_proposers.insert(i, 0);
    }

    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RoundProposer(HashMap::new()),
        Some(round_proposers),
    );

    // 4 honest nodes
    let n0_twin_id = nodes[0].id;
    // twin of n0 has same author as node_authors[0]
    let twin0_twin_id = nodes[4].id;
    assert_eq!(n0_twin_id.author, twin0_twin_id.author);
    let n1_twin_id = nodes[1].id;
    // twin of n1 has same author as node_authors[1]
    let twin1_twin_id = nodes[5].id;
    assert_eq!(n1_twin_id.author, twin1_twin_id.author);
    let n2_twin_id = nodes[2].id;
    let n3_twin_id = nodes[3].id;

    // Create per round partitions
    let mut round_partitions: HashMap<u64, Vec<Vec<TwinId>>> = HashMap::new();
    // Round 1 to 10 partitions: [node0, node1, node2], [node3, twin0, twin1]
    for i in 1..10 {
        round_partitions.insert(
            i,
            vec![
                vec![n0_twin_id, n1_twin_id, n2_twin_id],
                vec![n3_twin_id, twin0_twin_id, twin1_twin_id],
            ],
        );
    }
    assert!(playground.split_network_round(&round_partitions));
    runtime.spawn(playground.start());

    timed_block_on(&mut runtime, async {
        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        let twin0_commit = nodes[4].commit_cb_receiver.next().await;

        match (node0_commit, twin0_commit) {
            (Some(node0_commit_inner), Some(twin0_commit_inner)) => {
                let node0_commit_id = node0_commit_inner.ledger_info().commit_info().id();
                let twin0_commit_id = twin0_commit_inner.ledger_info().commit_info().id();
                // Proposal from both node0 and twin_node0 are going to
                // get committed in their respective partitions
                assert_ne!(node0_commit_id, twin0_commit_id);
            }
            _ => panic!("[TwinsTest] Test failed due to no commit(s)"),
        }
    });
}

#[test]
#[ignore] // TODO: https://github.com/diem/diem/issues/6615
/// This test checks that when a node and its twin are both leaders
/// for a round, only one of the two proposals gets committed
///
/// Setup:
///
/// Network of 4 nodes (n0, n1, n2, n3), and 1 twin (twin0)
///
/// Test:
///
/// Let n0 (and implicitly twin0) be proposers
/// Pull out enough votes so a commit can be formed
/// Check that the commit of n0 and twin0 matches
///
/// Run the test:
/// cargo xtest -p consensus twins_commit_test -- --nocapture
fn twins_commit_test() {
    let mut runtime = consensus_runtime();
    let mut playground = NetworkPlayground::new(runtime.handle().clone());
    let num_nodes = 4;
    let num_twins = 1;

    // Specify round leaders
    // Will default to the first node, if no leader specified for given round
    let mut round_proposers: HashMap<Round, usize> = HashMap::new();
    // Leaders are n0 and twin0 for round 1..10
    for i in 1..10 {
        round_proposers.insert(i, 0);
    }

    let mut nodes = SMRNode::start_num_nodes_with_twins(
        num_nodes,
        num_twins,
        &mut playground,
        RoundProposer(HashMap::new()),
        Some(round_proposers),
    );
    runtime.spawn(playground.start());

    timed_block_on(&mut runtime, async {
        let node0_commit = nodes[0].commit_cb_receiver.next().await;
        let twin0_commit = nodes[4].commit_cb_receiver.next().await;

        match (node0_commit, twin0_commit) {
            (Some(node0_commit_inner), Some(twin0_commit_inner)) => {
                let node0_commit_id = node0_commit_inner.ledger_info().commit_info().id();
                let twin0_commit_id = twin0_commit_inner.ledger_info().commit_info().id();
                // Proposals from both node0 and twin_node0 are going to race,
                // but only one of them will form a commit
                assert_eq!(node0_commit_id, twin0_commit_id);
            }
            _ => panic!("[TwinsTest] Test failed due to no commit(s)"),
        }
    });
}
