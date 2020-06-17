// Copyright (c) The Libra Core Contributors
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

pub struct Reducible<T: Ord + Copy + Debug> {
    pub loop_headers: BTreeSet<T>, // set of reachable loop headers
    pub natural_loops: BTreeMap<(T, T), BTreeSet<T>>, // map from reachable back edges to natural loops
}

pub struct Graph<T: Ord + Copy + Debug> {
    entry: T,
    nodes: BTreeSet<T>,
    edges: BTreeSet<(T, T)>,
    predecessors: BTreeMap<T, BTreeSet<T>>,
    successors: BTreeMap<T, BTreeSet<T>>,
}

impl<T: Ord + Copy + Debug> Graph<T> {
    /// This function creates a graph from a set of nodes (with a unique entry node)
    /// and a set of edges.
    pub fn new(entry: T, nodes: BTreeSet<T>, edges: BTreeSet<(T, T)>) -> Self {
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
    pub fn compute_reducible(&self) -> Option<Reducible<T>> {
        let dom_relation = DomRelation::new(self);
        let mut loop_headers = BTreeSet::new();
        let mut back_edges = BTreeSet::new();
        let mut non_back_edges = BTreeSet::new();
        for e in &self.edges {
            if !dom_relation.is_reachable(e.0) {
                continue;
            }
            if dom_relation.is_dominated_by(e.0, e.1) {
                back_edges.insert(*e);
                loop_headers.insert(e.1);
            } else {
                non_back_edges.insert(*e);
            }
        }
        if Graph::new(self.entry, self.nodes.clone(), non_back_edges).is_acyclic() {
            let natural_loops = back_edges
                .into_iter()
                .map(|edge| (edge, self.natural_loop(edge)))
                .collect();
            Some(Reducible {
                loop_headers,
                natural_loops,
            })
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

    fn natural_loop(&self, back_edge: (T, T)) -> BTreeSet<T> {
        let n = back_edge.0;
        let d = back_edge.1;
        let mut stack = vec![];
        let mut natural_loop = BTreeSet::new();
        natural_loop.insert(d);
        if n != d {
            natural_loop.insert(n);
            stack.push(n);
        }
        while !stack.is_empty() {
            let m = stack.pop().unwrap();
            for p in &self.predecessors[&m] {
                if !natural_loop.contains(p) {
                    natural_loop.insert(*p);
                    stack.push(*p);
                }
            }
        }
        natural_loop
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
