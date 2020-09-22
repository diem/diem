// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    network_tests::NetworkPlayground,
    test_utils::{consensus_runtime, timed_block_on},
    twins::{
        test_cases::{TestCase, TestCases},
        twins_node::SMRNode,
    },
};
use consensus_types::common::Round;
use futures::StreamExt;
use libra_crypto::HashValue;
use std::collections::HashMap;

pub fn execute(test_cases: TestCases) {
    let num_nodes = test_cases.num_of_nodes;
    let num_twins = test_cases.num_of_twins;
    for test_case in test_cases.scenarios {
        let mut runtime = consensus_runtime();
        let mut playground = NetworkPlayground::new(runtime.handle().clone());
        let stop_round = *test_case.round_leaders.keys().max().unwrap();
        // setup
        let mut nodes = SMRNode::start_num_nodes_with_twins(
            num_nodes,
            num_twins,
            &mut playground,
            Some(test_case),
        );
        // execute
        timed_block_on(
            &mut runtime,
            playground.start_until(move |msg| {
                NetworkPlayground::get_message_round(msg).map_or(false, |r| r > stop_round)
            }),
        );
        // check commits
        let mut all_commits = vec![];
        timed_block_on(&mut runtime, async {
            for node in &mut nodes {
                node.commit_cb_receiver.close();
                let mut commits = HashMap::new();
                while let Some(commit) = node.commit_cb_receiver.next().await {
                    let block_info = commit.ledger_info().commit_info();
                    commits.insert(block_info.round(), block_info.id());
                }
                all_commits.push(commits);
            }
        });
        assert!(is_safe(all_commits, stop_round));
    }
}

fn is_safe(all_commits: Vec<HashMap<Round, HashValue>>, highest_round: Round) -> bool {
    for round in 1..highest_round {
        let mut commit_id = None;
        for node in &all_commits {
            if let Some(id) = node.get(&round) {
                if id != commit_id.unwrap_or(id) {
                    return false;
                }
                commit_id = Some(id);
            }
        }
    }
    true
}

#[test]
fn execute_test_cases() {
    let test_case_1 = TestCase {
        round_leaders: (1..10).map(|r| (r, 0)).collect(),
        round_partitions: (1..10)
            .map(|r| (r, vec![vec![0, 1, 2], vec![3, 4]]))
            .collect(),
    };
    let test_case_2 = TestCase {
        round_leaders: (1..8).map(|r| (r, r as usize % 4)).collect(),
        round_partitions: vec![
            (1, vec![vec![0, 1], vec![2, 3, 4]]),
            (2, vec![vec![0, 2, 3], vec![1, 4]]),
            (3, vec![vec![0, 3], vec![1, 2, 4]]),
            (4, vec![vec![0, 1], vec![4, 2, 3]]),
            (5, vec![vec![0], vec![1], vec![2, 3, 4]]),
            (6, vec![vec![0, 2], vec![1, 3, 4]]),
            (7, vec![vec![0], vec![3], vec![1, 2, 4]]),
            (8, vec![vec![0], vec![1, 2, 3, 4]]),
        ]
        .into_iter()
        .collect(),
    };
    println!("{:?}", test_case_2.round_leaders);
    let test_cases = TestCases {
        num_of_nodes: 4,
        num_of_twins: 1,
        scenarios: vec![test_case_1, test_case_2],
    };
    execute(test_cases);
}
