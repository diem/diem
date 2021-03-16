// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// This module implements a technique to compute the natural loops of a graph.
// The implementation is based on the computation of the dominance relation
// of a graph using the technique method in this paper:
// Keith D. Cooper, Timothy J. Harvey, Ken Kennedy, "A Simple, Fast Dominance Algorithm ",
// Software Practice and Experience, 2001.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
};

pub struct NaturalLoop<T: Ord + Copy + Debug> {
    pub loop_header: T,
    pub loop_latch: T, // latch -> header is the back edge representing this loop
    pub loop_body: BTreeSet<T>,
}

pub struct Graph<T: Ord + Copy + Debug> {
    entry: T,
    nodes: Vec<T>,
    edges: Vec<(T, T)>,
    predecessors: BTreeMap<T, BTreeSet<T>>,
    successors: BTreeMap<T, BTreeSet<T>>,
}

impl<T: Ord + Copy + Debug> Graph<T> {
    /// This function creates a graph from a set of nodes (with a unique entry node)
    /// and a set of edges.
    pub fn new(entry: T, nodes: Vec<T>, edges: Vec<(T, T)>) -> Self {
        let mut predecessors: BTreeMap<T, BTreeSet<T>> =
            nodes.iter().map(|x| (*x, BTreeSet::new())).collect();
        let mut successors: BTreeMap<T, BTreeSet<T>> =
            nodes.iter().map(|x| (*x, BTreeSet::new())).collect();
        for edge in &edges {
            successors.entry(edge.0).and_modify(|x| {
                x.insert(edge.1);
            });
            predecessors.entry(edge.1).and_modify(|x| {
                x.insert(edge.0);
            });
        }
        Self {
            entry,
            nodes,
            edges,
            predecessors,
            successors,
        }
    }

    /// This function computes the loop headers and natural loops of a reducible graph.
    /// If the graph is irreducible, None is returned.
    pub fn compute_reducible(&self) -> Option<Vec<NaturalLoop<T>>> {
        let dom_relation = DomRelation::new(self);
        let mut loop_headers = BTreeSet::new();
        let mut back_edges = vec![];
        let mut non_back_edges = vec![];
        for e in &self.edges {
            if !dom_relation.is_reachable(e.0) {
                continue;
            }
            if dom_relation.is_dominated_by(e.0, e.1) {
                back_edges.push(*e);
                loop_headers.insert(e.1);
            } else {
                non_back_edges.push(*e);
            }
        }
        if Graph::new(self.entry, self.nodes.clone(), non_back_edges).is_acyclic() {
            let natural_loops = back_edges
                .into_iter()
                .map(|edge| self.natural_loop(edge))
                .collect();
            Some(natural_loops)
        } else {
            None
        }
    }

    fn is_acyclic(&self) -> bool {
        let mut visited = BTreeMap::new();
        let mut stack = vec![];
        visited.insert(self.entry, false);
        stack.push(self.entry);
        while !stack.is_empty() {
            let n = stack.pop().unwrap();
            if visited[&n] {
                visited.entry(n).and_modify(|x| {
                    *x = false;
                });
                continue;
            }
            stack.push(n);
            visited.entry(n).and_modify(|x| {
                *x = true;
            });
            for s in &self.successors[&n] {
                if visited.contains_key(s) {
                    if visited[s] {
                        return false;
                    }
                } else {
                    visited.insert(*s, false);
                    stack.push(*s);
                }
            }
        }
        true
    }

    fn natural_loop(&self, back_edge: (T, T)) -> NaturalLoop<T> {
        let loop_latch = back_edge.0;
        let loop_header = back_edge.1;
        let mut stack = vec![];
        let mut loop_body = BTreeSet::new();
        loop_body.insert(loop_header);
        if loop_latch != loop_header {
            loop_body.insert(loop_latch);
            stack.push(loop_latch);
        }
        while !stack.is_empty() {
            let m = stack.pop().unwrap();
            for p in &self.predecessors[&m] {
                if !loop_body.contains(p) {
                    loop_body.insert(*p);
                    stack.push(*p);
                }
            }
        }
        NaturalLoop {
            loop_header,
            loop_latch,
            loop_body,
        }
    }
}

struct DomRelation<T: Ord + Copy + Debug> {
    node_to_postorder_num: BTreeMap<T, usize>,
    postorder_num_to_node: Vec<T>,
    idom_tree: BTreeMap<usize, usize>,
}

impl<T: Ord + Copy + Debug> DomRelation<T> {
    /// This function computes the dominance relation on the subset of the graph
    /// that is reachable from its entry node.
    pub fn new(graph: &Graph<T>) -> Self {
        let mut dom_relation = Self {
            node_to_postorder_num: BTreeMap::new(),
            postorder_num_to_node: vec![],
            idom_tree: BTreeMap::new(),
        };
        dom_relation.postorder_visit(graph);
        dom_relation.compute_dominators(graph);
        dom_relation
    }

    /// This function returns true iff `x` is reachable from the entry node of the graph.
    pub fn is_reachable(&self, x: T) -> bool {
        self.node_to_postorder_num.contains_key(&x)
    }

    /// This function returns true iff `x` is dominated by `y`.
    pub fn is_dominated_by(&self, x: T, y: T) -> bool {
        let x_num = self.node_to_postorder_num[&x];
        let y_num = self.node_to_postorder_num[&y];
        let mut curr_num = x_num;
        loop {
            if curr_num == y_num {
                return true;
            }
            if curr_num == self.entry_num() {
                return false;
            }
            curr_num = self.idom_tree[&curr_num];
        }
    }

    fn entry_num(&self) -> usize {
        self.num_nodes() - 1
    }

    fn num_nodes(&self) -> usize {
        self.node_to_postorder_num.len()
    }

    fn postorder_visit(&mut self, graph: &Graph<T>) {
        let mut stack = vec![];
        let mut visited = BTreeSet::new();
        let mut grey = BTreeSet::new();
        stack.push(graph.entry);
        visited.insert(graph.entry);
        while !stack.is_empty() {
            let curr = stack.pop().unwrap();
            if grey.contains(&curr) {
                let curr_num = self.postorder_num_to_node.len();
                self.postorder_num_to_node.push(curr);
                self.node_to_postorder_num.insert(curr, curr_num);
            } else {
                grey.insert(curr);
                stack.push(curr);
                for child in &graph.successors[&curr] {
                    if !visited.contains(child) {
                        visited.insert(*child);
                        stack.push(*child);
                    }
                }
            }
        }
    }

    fn compute_dominators(&mut self, graph: &Graph<T>) {
        let entry_num = self.entry_num();
        self.idom_tree.insert(entry_num, entry_num);
        let mut changed = true;
        while changed {
            changed = false;
            for node_num in (0..self.num_nodes() - 1).rev() {
                let b = self.postorder_num_to_node[node_num];
                let mut new_idom = self.num_nodes();
                for p in &graph.predecessors[&b] {
                    if !self.node_to_postorder_num.contains_key(p) {
                        continue; // not all nodes are reachable
                    }
                    let pred_num = self.node_to_postorder_num[p];
                    if self.idom_tree.contains_key(&pred_num) {
                        if new_idom == self.num_nodes() {
                            new_idom = pred_num;
                        } else {
                            new_idom = self.intersect(pred_num, new_idom);
                        }
                    }
                }
                assert!(new_idom != self.num_nodes());
                if self.idom_tree.contains_key(&node_num) && self.idom_tree[&node_num] == new_idom {
                    continue;
                } else {
                    self.idom_tree
                        .entry(node_num)
                        .and_modify(|x| {
                            *x = new_idom;
                        })
                        .or_insert(new_idom);
                    changed = true;
                }
            }
        }
    }

    fn intersect(&self, x: usize, y: usize) -> usize {
        let mut finger1 = x;
        let mut finger2 = y;
        while finger1 != finger2 {
            while finger1 < finger2 {
                finger1 = self.idom_tree[&finger1];
            }
            while finger2 < finger1 {
                finger2 = self.idom_tree[&finger2];
            }
        }
        finger1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let nodes = vec![1, 2, 3, 4, 5];
        let edges = vec![(1, 2), (2, 1), (4, 1), (3, 2), (5, 3), (5, 4)];
        let source = 5;
        let graph = Graph::new(source, nodes, edges);
        assert!(graph.compute_reducible().is_none());
    }

    #[test]
    fn test2() {
        let nodes = vec![1, 2, 3, 4, 5, 6];
        let edges = vec![
            (1, 2),
            (2, 1),
            (2, 3),
            (3, 2),
            (4, 2),
            (4, 3),
            (5, 1),
            (6, 5),
            (6, 4),
        ];
        let source = 6;
        let graph = Graph::new(source, nodes, edges);
        assert!(graph.compute_reducible().is_none());
    }

    #[test]
    fn test3() {
        let nodes = vec![1, 2, 3, 4, 5, 6];
        let edges = vec![(1, 2), (2, 3), (2, 4), (2, 6), (3, 5), (4, 5), (5, 2)];
        let source = 1;
        let graph = Graph::new(source, nodes, edges);
        let natural_loops = graph.compute_reducible().unwrap();
        assert_eq!(natural_loops.len(), 1);
        let single_loop = &natural_loops[0];
        assert_eq!(single_loop.loop_header, 2);
        assert_eq!(single_loop.loop_latch, 5);
        assert_eq!(
            single_loop.loop_body,
            vec![2, 3, 4, 5].into_iter().collect()
        );
    }

    #[test]
    fn test4() {
        let nodes = vec![1, 2, 3, 4, 5, 6];
        let edges = vec![(1, 2), (2, 3), (2, 4), (2, 6), (3, 5), (4, 5), (5, 2)];
        let source = 6;
        let graph = Graph::new(source, nodes, edges);
        assert!(graph.compute_reducible().is_some());
    }

    #[test]
    fn test5() {
        // nested natural loops
        let nodes = vec![1, 2, 3, 4, 5, 6];
        let edges = vec![
            (1, 2),
            (2, 3),
            (2, 4),
            (2, 6),
            (3, 5),
            (4, 5),
            (5, 2),
            (3, 2),
        ];
        let source = 1;
        let graph = Graph::new(source, nodes, edges);
        let natural_loops = graph.compute_reducible().unwrap();

        assert_eq!(natural_loops.len(), 2);
        let (inner_loop, outer_loop) = if natural_loops[0].loop_latch == 3 {
            assert_eq!(natural_loops[1].loop_latch, 5);
            (&natural_loops[0], &natural_loops[1])
        } else {
            assert_eq!(natural_loops[0].loop_latch, 5);
            assert_eq!(natural_loops[1].loop_latch, 3);
            (&natural_loops[1], &natural_loops[0])
        };

        assert_eq!(inner_loop.loop_header, 2);
        assert_eq!(inner_loop.loop_body, vec![2, 3].into_iter().collect());

        assert_eq!(outer_loop.loop_header, 2);
        assert_eq!(outer_loop.loop_body, vec![2, 3, 4, 5].into_iter().collect());
    }
}
