// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::graph::{Graph, Reducible};
use std::collections::BTreeSet;

#[test]
fn test1() {
    let nodes: BTreeSet<u16> = vec![1, 2, 3, 4, 5].into_iter().collect();
    let edges: BTreeSet<(u16, u16)> = vec![(1, 2), (2, 1), (4, 1), (3, 2), (5, 3), (5, 4)]
        .into_iter()
        .collect();
    let source = 5;
    let graph = Graph::new(source, nodes, edges);
    assert!(graph.compute_reducible().is_none());
}

#[test]
fn test2() {
    let nodes: BTreeSet<u16> = vec![1, 2, 3, 4, 5, 6].into_iter().collect();
    let edges: BTreeSet<(u16, u16)> = vec![
        (1, 2),
        (2, 1),
        (2, 3),
        (3, 2),
        (4, 2),
        (4, 3),
        (5, 1),
        (6, 5),
        (6, 4),
    ]
    .into_iter()
    .collect();
    let source = 6;
    let graph = Graph::new(source, nodes, edges);
    assert!(graph.compute_reducible().is_none());
}

#[test]
fn test3() {
    let nodes: BTreeSet<u16> = vec![1, 2, 3, 4, 5, 6].into_iter().collect();
    let edges: BTreeSet<(u16, u16)> = vec![(1, 2), (2, 3), (2, 4), (2, 6), (3, 5), (4, 5), (5, 2)]
        .into_iter()
        .collect();
    let source = 1;
    let graph = Graph::new(source, nodes, edges);
    let Reducible {
        loop_headers,
        natural_loops,
    } = graph.compute_reducible().unwrap();
    assert!(loop_headers == vec![2].into_iter().collect());
    assert!(
        natural_loops.len() == 1
            && natural_loops.contains_key(&(5, 2))
            && natural_loops[&(5, 2)] == vec![2, 3, 4, 5].into_iter().collect()
    );
}

#[test]
fn test4() {
    let nodes: BTreeSet<u16> = vec![1, 2, 3, 4, 5, 6].into_iter().collect();
    let edges: BTreeSet<(u16, u16)> = vec![(1, 2), (2, 3), (2, 4), (2, 6), (3, 5), (4, 5), (5, 2)]
        .into_iter()
        .collect();
    let source = 6;
    let graph = Graph::new(source, nodes, edges);
    assert!(graph.compute_reducible().is_some());
}
