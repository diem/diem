// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::{Author, Round};
use libra_config::config::RoundProposerConfig;
use libra_types::account_address::AccountAddress;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

type Idx = usize;

#[derive(Serialize, Deserialize)]
struct TestCase {
    round_leaders: HashMap<Round, Idx>,
    round_partitions: HashMap<Round, Vec<Vec<Idx>>>,
}

#[derive(Serialize, Deserialize)]
struct TestCases {
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
    fn to_round_proposer_config(&self, authors: &Vec<Author>) -> Vec<RoundProposerConfig> {
        let mut round_proposers = HashMap::new();
        let mut timeouts: Vec<_> = authors.iter().map(|_| HashSet::new()).collect();
        for (round, leader_idx) in &self.round_leaders {
            let leader = authors[*leader_idx];
            round_proposers.insert(*round, leader);
            for partition in self.round_partitions.get(&round).unwrap() {
                let with_leader = partition.iter().find(|idx| **idx == *leader_idx).is_some();
                if !with_leader {
                    for node_idx in partition {
                        timeouts[*node_idx].insert(*round);
                    }
                }
            }
        }
        timeouts
            .into_iter()
            .map(|timeout_rounds| RoundProposerConfig {
                round_proposers: round_proposers.clone(),
                timeout_rounds,
            })
            .collect()
    }
}

#[test]
fn test_case_conversion() {
    let round_leaders: HashMap<_, _> = vec![(1, 0), (2, 1), (3, 2)].into_iter().collect();
    let round_partitions = vec![
        (1, vec![vec![0, 1], vec![2, 3]]),
        (2, vec![vec![0], vec![1, 2, 3]]),
        (3, vec![vec![0, 3], vec![2], vec![1]]),
    ]
    .into_iter()
    .collect();
    let authors: Vec<_> = (0..4).map(|_| AccountAddress::random()).collect();
    let expected_leaders: HashMap<Round, Author> = round_leaders
        .iter()
        .map(|(r, idx)| (*r, authors[*idx]))
        .collect();
    let test_case = TestCase {
        round_leaders,
        round_partitions,
    };
    let configs = test_case.to_round_proposer_config(&authors);
    for config in &configs {
        assert_eq!(config.round_proposers, expected_leaders);
    }

    assert_eq!(configs[0].timeout_rounds, vec![2, 3].into_iter().collect());
    assert_eq!(configs[1].timeout_rounds, vec![3].into_iter().collect());
    assert_eq!(configs[2].timeout_rounds, vec![1].into_iter().collect());
    assert_eq!(configs[3].timeout_rounds, vec![1, 3].into_iter().collect());
}
