// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, BTreeSet};

use bytecode_verifier::absint::{AbstractDomain, JoinResult};
use vm::file_format::{CodeOffset, SignatureToken};

use crate::{
    dataflow_analysis::{DataflowAnalysis, StateMap, TransferFunctions},
    stackless_bytecode::{
        StacklessBytecode::{self, *},
        TempIndex,
    },
    stackless_control_flow_graph::StacklessControlFlowGraph,
};

/// Represents a node in the borrow graph.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Node {
    /// Root means global storage or local storage. For example when there's BorrowLoc(dest, _) or
    /// BorrowGlobal(dest,_,_) then an edge will be added between Root -> Local(dest)
    Root,

    /// Node representing a local. When BorrowField(dest, src), an edge between Local(src)
    /// and Local(dest) is added
    Local(TempIndex),
}

#[derive(Clone, Debug, Default)]
pub struct BorrowGraph {
    graph: BTreeMap<Node, BTreeSet<Node>>,
    reverse_graph: BTreeMap<Node, BTreeSet<Node>>,
    moved_locals: BTreeSet<TempIndex>,
}

impl BorrowGraph {
    pub fn new() -> Self {
        BorrowGraph {
            graph: BTreeMap::new(),
            reverse_graph: BTreeMap::new(),
            moved_locals: BTreeSet::new(),
        }
    }

    /// Add an edge in the graph
    pub fn add_edge(&mut self, from: Node, to: Node) {
        // Fancy statement that modifies the value of key from if it exists, and inserts otherwise.
        self.graph
            .entry(from)
            .and_modify(|e| {
                e.insert(to);
            })
            .or_insert_with(|| [to].iter().cloned().collect());

        // Add edge to reverse map
        self.reverse_graph
            .entry(to)
            .and_modify(|e| {
                e.insert(from);
            })
            .or_insert_with(|| [from].iter().cloned().collect());
    }

    /// Mark a local as moved. Notice that moving the local doesn't mean
    /// it's dead and can be removed
    pub fn move_local(&mut self, local: TempIndex) {
        let node = Node::Local(local);

        // Only mark local if it actually exists in the graph
        if self.graph.contains_key(&node) || self.reverse_graph.contains_key(&node) {
            self.moved_locals.insert(local);
        }
    }

    /// Replace all appearances of before with after in the graph
    pub fn replace_local(&mut self, before: Node, after: Node) {
        if let Some(to_neighbors) = self.graph.remove(&before) {
            for to in &to_neighbors {
                self.reverse_graph.entry(*to).and_modify(|e| {
                    e.remove(&before);
                    e.insert(after);
                });
            }
            self.graph.insert(after, to_neighbors);
        }
        if let Some(from_neighbors) = self.reverse_graph.remove(&before) {
            for from in &from_neighbors {
                self.graph.entry(*from).and_modify(|e| {
                    e.remove(&before);
                    e.insert(after);
                });
            }
            self.reverse_graph.insert(after, from_neighbors);
        }
    }

    /// Copy all appearances of src to dest in the graph
    pub fn copy_local(&mut self, dest: Node, src: Node) {
        if self.graph.contains_key(&src) {
            let to_neighbors = self.graph[&src].clone();
            for to in &to_neighbors {
                self.reverse_graph.entry(*to).and_modify(|e| {
                    e.insert(dest);
                });
            }
            self.graph.insert(dest, to_neighbors);
        }
        if self.reverse_graph.contains_key(&src) {
            let from_neighbors = self.reverse_graph[&src].clone();
            for from in &from_neighbors {
                self.graph.entry(*from).and_modify(|e| {
                    e.insert(dest);
                });
            }
            self.reverse_graph.insert(dest, from_neighbors);
        }
    }

    /// Join two borrow graphs, so that self is mutated into a graph consisting of
    /// edges in both graphs
    pub fn join(&mut self, other: &Self) {
        for (n, to_neighbors) in &other.graph {
            self.graph
                .entry(*n)
                .and_modify(|e| {
                    e.extend(to_neighbors);
                })
                .or_insert_with(|| to_neighbors.clone());
        }

        for (n, from_neighbors) in &other.reverse_graph {
            self.reverse_graph
                .entry(*n)
                .and_modify(|e| {
                    e.extend(from_neighbors);
                })
                .or_insert_with(|| from_neighbors.clone());
        }

        self.moved_locals.extend(&other.moved_locals);
    }

    /// If the BorrowGraph is a subset of other
    pub fn is_subset(&self, other: &Self) -> bool {
        self.graph
            .keys()
            .all(|k| other.graph.contains_key(k) && self.graph[&k].is_subset(&other.graph[&k]))
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node: Node) {
        if let Node::Local(l) = node {
            self.moved_locals.remove(&l);
        }

        self.graph.remove(&node);
        if self.reverse_graph.contains_key(&node) {
            let from_neighbors = &self.reverse_graph[&node];
            for from in from_neighbors {
                self.graph.entry(*from).and_modify(|e| {
                    e.remove(&node);
                });
            }
        }
    }

    /// Find all the moved sink nodes, nodes that don't have any incoming edges
    pub fn find_sink_nodes(&mut self) -> BTreeSet<Node> {
        let mut res = BTreeSet::new();
        for l in &self.moved_locals {
            let n = Node::Local(*l);
            if !self.graph.contains_key(&n) || self.graph[&n].is_empty() {
                res.insert(n);
            }
        }
        res
    }

    /// Trim the borrow graph by iteratively deleting moved sink nodes from the graph
    /// Return the deleted nodes
    pub fn trim_graph(&mut self) -> BTreeSet<TempIndex> {
        let mut sink_nodes = self.find_sink_nodes();
        let mut trimmed_nodes = BTreeSet::new();
        while !sink_nodes.is_empty() {
            for n in sink_nodes {
                self.remove_node(n);
                if let Node::Local(l) = n {
                    trimmed_nodes.insert(l);
                }
            }
            sink_nodes = self.find_sink_nodes();
        }
        trimmed_nodes
    }
}

pub struct LifetimeAnalysis<'a> {
    local_types: &'a [SignatureToken],
}

#[derive(Clone, Debug)]
pub struct LifetimeState {
    borrow_graph: BorrowGraph,

    /// Mutable references that * just * go out of scope at the end of line CodeOffset
    dead_refs: BTreeMap<CodeOffset, BTreeSet<TempIndex>>,
}

impl LifetimeState {
    /// For each key in the maps, union the values corresponding to the key in two maps
    /// e.g., if one = {1: {2,3}, 2: {4}} and other = {1: {2,4}, 3: {5}},
    ///       then this function will mutate one into {1: {2,3,4}, 2: {4}, 3: {5}}
    fn dead_ref_join(
        one: &mut BTreeMap<CodeOffset, BTreeSet<TempIndex>>,
        other: &BTreeMap<CodeOffset, BTreeSet<TempIndex>>,
    ) {
        for (k, v) in other {
            one.entry(*k)
                .and_modify(|e| e.extend(v))
                .or_insert_with(|| v.clone());
        }
    }
}

impl AbstractDomain for LifetimeState {
    fn join(&mut self, other: &Self) -> JoinResult {
        Self::dead_ref_join(&mut self.dead_refs, &other.dead_refs);
        self.borrow_graph.join(&other.borrow_graph);

        let dead_refs_unchanged = self
            .dead_refs
            .keys()
            .all(|idx| other.dead_refs.get(&idx) == self.dead_refs.get(&idx));
        let borrow_graph_unchanged = self.borrow_graph.is_subset(&other.borrow_graph);
        if dead_refs_unchanged && borrow_graph_unchanged {
            JoinResult::Unchanged
        } else {
            JoinResult::Changed
        }
    }
}

impl<'a> LifetimeAnalysis<'a> {
    pub fn analyze(
        cfg: &StacklessControlFlowGraph,
        instrs: &[StacklessBytecode],
        local_types: &'a [SignatureToken],
    ) -> BTreeMap<CodeOffset, BTreeSet<TempIndex>> {
        let mut analyzer = Self { local_types };
        let initial_state = LifetimeState {
            borrow_graph: BorrowGraph::new(),
            dead_refs: BTreeMap::new(),
        };
        let state_map = analyzer.analyze_function(initial_state, &instrs, cfg);
        Self::post_process(state_map)
    }

    /// Union the set of dead references at each CodeOffset
    fn post_process(
        state_map: StateMap<LifetimeState>,
    ) -> BTreeMap<CodeOffset, BTreeSet<TempIndex>> {
        let mut res = BTreeMap::new();
        for (_, v) in state_map {
            LifetimeState::dead_ref_join(&mut res, &v.post.dead_refs);
        }
        res
    }
}

impl<'a> TransferFunctions for LifetimeAnalysis<'a> {
    type InstrType = StacklessBytecode;
    type State = LifetimeState;

    fn execute(
        &mut self,
        pre: &Self::State,
        instr: &Self::InstrType,
        idx: CodeOffset,
    ) -> Self::State {
        let mut after_state = pre.clone();

        match instr {
            MoveLoc(t, l) => {
                if self.local_types[*t].is_mutable_reference() {
                    after_state
                        .borrow_graph
                        .replace_local(Node::Local(*l), Node::Local(*t));
                }
            }
            CopyLoc(t, l) => {
                if self.local_types[*t].is_mutable_reference() {
                    after_state
                        .borrow_graph
                        .copy_local(Node::Local(*l), Node::Local(*t));
                }
            }
            StLoc(l, t) => {
                if self.local_types[*t].is_mutable_reference() {
                    after_state.borrow_graph.move_local(*l);
                    after_state
                        .borrow_graph
                        .replace_local(Node::Local(*t), Node::Local(*l));
                }
            }
            BorrowLoc(t, _) => {
                if self.local_types[*t].is_mutable_reference() {
                    after_state
                        .borrow_graph
                        .add_edge(Node::Root, Node::Local(*t));
                }
            }
            BorrowGlobal(t, _, _, _) => {
                if self.local_types[*t].is_mutable_reference() {
                    after_state
                        .borrow_graph
                        .add_edge(Node::Root, Node::Local(*t));
                }
            }
            BorrowField(dest, src, _, _) => {
                if self.local_types[*src].is_mutable_reference() {
                    after_state.borrow_graph.move_local(*src);
                }
                if self.local_types[*dest].is_mutable_reference() {
                    after_state
                        .borrow_graph
                        .add_edge(Node::Local(*src), Node::Local(*dest));
                }
            }
            FreezeRef(_, src) => {
                after_state.borrow_graph.move_local(*src);
            }
            WriteRef(t, _) => {
                after_state.borrow_graph.move_local(*t);
            }
            ReadRef(_, src) => {
                if self.local_types[*src].is_mutable_reference() {
                    after_state.borrow_graph.move_local(*src);
                }
            }
            Call(dest_vec, _, _, src_vec) => {
                let mut dest_mut_refs = dest_vec.clone();
                dest_mut_refs.retain(|d| self.local_types[*d].is_mutable_reference());
                let mut src_mut_refs = src_vec.clone();
                src_mut_refs.retain(|s| self.local_types[*s].is_mutable_reference());
                for s in src_mut_refs {
                    after_state.borrow_graph.move_local(s);
                    // this is over approximating right now
                    // we only need to add an edge if it's possible for d to come from s
                    // for example, if d is an address ref but s is LibraCoin ref then
                    // there is no way that d is borrowed from s
                    for d in &dest_mut_refs {
                        after_state
                            .borrow_graph
                            .add_edge(Node::Local(s), Node::Local(*d));
                    }
                }
            }
            Pop(t) => {
                after_state.borrow_graph.move_local(*t);
            }
            _ => {
                // Other instructions don't deal with mutable references
            }
        }

        // Dead refs are those newly trimmed from the graph
        after_state
            .dead_refs
            .insert(idx, after_state.borrow_graph.trim_graph());
        after_state
    }
}

impl<'a> DataflowAnalysis for LifetimeAnalysis<'a> {}
