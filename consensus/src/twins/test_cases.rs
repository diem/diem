// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::network_tests::TwinId;
use consensus_types::common::{Author, Round};
use libra_config::config::RoundProposerConfig;
use libra_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

type Idx = usize;

#[derive(Serialize, Deserialize)]
/// Specify the leaders and partitions for each round.
/// Default is FixedProposer and no partitions.
/// In order to synchronize the round across all nodes, we have two hacks in NetworkPlayround:
/// 1. Do not drop timeout votes.
/// 2. When a proposal is dropped, still send the sync info.
pub struct TestCase {
    pub round_leaders: HashMap<Round, Idx>,
    pub round_partitions: HashMap<Round, Vec<Vec<Idx>>>,
}

#[derive(Serialize, Deserialize)]
pub struct TestCases {
    num_of_nodes: u64,
    num_of_twins: u64,
    scenarios: Vec<TestCase>,
}

#[test]
fn test_case_format() {
    let sample = r#"
    {"num_of_nodes": 4,
     "num_of_twins": 1,
     "scenarios": [
         {"round_leaders": {"1": 0, "2": 4, "3": 0},
          "round_partitions": {"1": [[0, 1, 2, 3], [4]], "2": [[0, 1, 2, 3], [4]], "3": [[0, 1, 2, 3], [4]]}
         },
         {"round_leaders": {"1": 4, "2": 0, "3": 0},
          "round_partitions": {"1": [[0, 1, 2, 3], [4]], "2": [[0, 1, 2, 4], [3]], "3": [[0, 1, 2, 4], [3]]}
         }
     ]
    }
    "#;
    let test_cases: TestCases = serde_json::from_str(sample).unwrap();
    assert_eq!(test_cases.num_of_nodes, 4);
    assert_eq!(test_cases.num_of_twins, 1);
    assert_eq!(test_cases.scenarios.len(), 2);
}

impl TestCase {
    /// We decide whether a round should timeout based on if the leader partition is able to aggregate QC.
    pub fn to_round_proposer_config(&self, nodes: &[TwinId]) -> Vec<RoundProposerConfig> {
        let mut round_proposers = HashMap::new();
        let total_size = nodes
            .iter()
            .map(|id| id.author)
            .collect::<HashSet<_>>()
            .len();
        let quorum_size = total_size * 2 / 3 + 1;
        let mut timeout_rounds = HashSet::new();
        for (round, leader_idx) in &self.round_leaders {
            let leader = nodes[*leader_idx].author;
            let next_leader_idx = self.round_leaders.get(&(*round + 1)).unwrap_or(&0);
            let next_leader = nodes[*next_leader_idx].author;
            round_proposers.insert(*round, leader);
            let mut should_timeout = true;
            for partition in self.round_partitions.get(&round).unwrap() {
                // if leader partition doesn't have quorum, everyone timeout, else no one timeout
                let has_leader = partition.iter().any(|idx| nodes[*idx].author == leader);
                let has_next_leader = partition
                    .iter()
                    .any(|idx| nodes[*idx].author == next_leader);
                if has_leader && has_next_leader {
                    let partition_size = partition
                        .iter()
                        .map(|idx| nodes[*idx].author)
                        .collect::<HashSet<_>>()
                        .len();
                    if partition_size >= quorum_size {
                        should_timeout = false;
                    }
                }
            }
            if should_timeout {
                timeout_rounds.insert(*round);
            }
        }
        let config = RoundProposerConfig {
            round_proposers,
            timeout_rounds,
        };

        (0..nodes.len()).map(|_| config.clone()).collect()
    }

    pub fn into_partitions(self, nodes: &[TwinId]) -> HashMap<Round, Vec<Vec<TwinId>>> {
        self.round_partitions
            .into_iter()
            .map(|(r, partitions)| {
                (
                    r,
                    partitions
                        .into_iter()
                        .map(|p| p.into_iter().map(|idx| nodes[idx]).collect())
                        .collect(),
                )
            })
            .collect()
    }
}

#[test]
fn test_case_conversion() {
    let round_leaders: HashMap<_, _> = vec![(1, 0), (2, 1), (3, 2), (4, 0)].into_iter().collect();
    let round_partitions = vec![
        (1, vec![vec![0, 1], vec![2, 3, 4]]),
        (2, vec![vec![0, 4], vec![1, 2, 3]]),
        (3, vec![vec![0, 3], vec![2, 4], vec![1]]),
        (4, vec![vec![0, 3, 4], vec![1, 2]]),
    ]
    .into_iter()
    .collect();
    let mut nodes_id: Vec<_> = (0..4)
        .map(|id| TwinId {
            id,
            author: AccountAddress::random(),
        })
        .collect();
    // create one twins
    nodes_id.push(nodes_id[0]);
    let expected_leaders: HashMap<Round, Author> = round_leaders
        .iter()
        .map(|(r, idx)| (*r, nodes_id[*idx].author))
        .collect();
    let test_case = TestCase {
        round_leaders,
        round_partitions,
    };
    let configs = test_case.to_round_proposer_config(&nodes_id);
    for config in &configs {
        assert_eq!(config.round_proposers, expected_leaders);
        assert_eq!(config.timeout_rounds, vec![1, 3, 4].into_iter().collect());
    }
}
