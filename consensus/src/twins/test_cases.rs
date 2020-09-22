// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use consensus_types::common::Round;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

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
fn test_format() {
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
