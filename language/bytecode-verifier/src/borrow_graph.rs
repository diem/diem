// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines the (acyclic) borrow graph for the type and memory safety analysis.
//! A node in the borrow graph represents an abstract reference.  Each edge in the borrow graph
//! is labeled with a (possibly empty) sequence of label elements.  A label element is either a
//! field index or a local index or a struct index.  An edge coming out of frame_root
//! (see abstract_state.rs) is labeled with a sequence beginning with a local or struct index
//! followed by zero or more field indices.  An edge coming out of a node different from frame_root
//! is labeled by a sequence of zero or more field indices.

//! An edge in the borrow graph from a node other than frame_root represents a prefix relationship
//! between the references represented by the source and sink of the edge.  There are two kinds of
//! edges---strong and weak.  A strong edge from node a to b labeled by sequence p indicates that
//! b is equal to the p-extension of a.  Instead, if the edge was weak, it indicates that b is an
//! extension of the p-extension of a.

use crate::nonce::Nonce;
use mirai_annotations::{
    checked_assume, checked_postcondition, checked_precondition, checked_verify,
};
use std::collections::{BTreeMap, BTreeSet};

/// The type of an edge
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EdgeType {
    Strong,
    Weak,
}

/// The label on an edge
pub type Label<T> = Vec<T>;

/// A labeled edge in borrow graph
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Edge<T> {
    edge_type: EdgeType,
    label: Label<T>,
    to: Nonce,
}

impl<T> Edge<T>
where
    T: Ord,
{
    pub fn is_prefix(&self, other: &Edge<T>) -> bool {
        self == other
            || (self.edge_type == EdgeType::Weak
                && other.label.starts_with(&self.label)
                && self.to == other.to)
    }
}

/// A borrow graph is represented as a map from a source nonce to the set of all edges
/// coming out of it.
#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct BorrowGraph<T>(BTreeMap<Nonce, BTreeSet<Edge<T>>>);

impl<T> BorrowGraph<T>
where
    T: Ord,
    T: Clone,
{
    /// creates a new empty borrow graph
    pub fn new() -> Self {
        BorrowGraph(BTreeMap::new())
    }

    /// adds a fresh nonce
    pub fn add_nonce(&mut self, nonce: Nonce) {
        checked_precondition!(!self.0.contains_key(&nonce));
        self.0.insert(nonce, BTreeSet::new());
    }

    /// adds a weak edge
    pub fn add_weak_edge(&mut self, from: Nonce, label: Label<T>, to: Nonce) {
        checked_precondition!(self.0.contains_key(&from));
        checked_precondition!(self.0.contains_key(&to));
        checked_precondition!(self.0[&to].is_empty());
        let new_edge = Edge {
            edge_type: EdgeType::Weak,
            label,
            to,
        };
        self.0.get_mut(&from).unwrap().insert(new_edge);
    }

    /// adds a strong edge and factors other edges coming out of `from` with respect to the new edge
    pub fn add_strong_edge(&mut self, from: Nonce, label: Label<T>, to: Nonce) {
        checked_precondition!(self.0.contains_key(&from));
        checked_precondition!(self.0.contains_key(&to));
        checked_precondition!(self.0[&to].is_empty());
        checked_precondition!(label.len() <= 1);

        let new_edge = Edge {
            edge_type: EdgeType::Strong,
            label: label.clone(),
            to,
        };
        self.0.remove(&to).unwrap();
        let from_edge_set = self.0.remove(&from).unwrap();
        if label.is_empty() {
            let new_from_edge_set = {
                let mut x = BTreeSet::new();
                x.insert(new_edge);
                x
            };
            self.0.insert(from, new_from_edge_set);
            self.0.insert(to, from_edge_set);
        } else {
            let (mut new_to_edges, mut new_from_edge_set) = Self::split(from_edge_set, |x| {
                !x.label.is_empty() && x.label[0] == label[0]
            });
            new_from_edge_set.insert(new_edge);
            new_to_edges.iter_mut().for_each(|x| {
                x.label.remove(0);
            });
            self.0.insert(from, new_from_edge_set);
            self.0.insert(to, new_to_edges.into_iter().collect());
        }
    }

    /// removes `nonce` and appropriately concatenates each incoming edge with each outgoing edge of `nonce`
    pub fn remove_nonce(&mut self, nonce: Nonce) {
        checked_assume!(self.invariant());
        let nonce_edge_set = self.0.remove(&nonce).unwrap();
        let removed_edges = {
            let mut x = BTreeMap::new();
            for (n, es) in &self.0 {
                x.insert(
                    n.clone(),
                    es.iter()
                        .filter(|x| x.to == nonce)
                        .cloned()
                        .collect::<Vec<_>>(),
                );
            }
            x
        };
        for (n, es) in &removed_edges {
            es.iter().for_each(|removed_edge| {
                let n_edge_set_ref = self.0.get_mut(n).unwrap();
                n_edge_set_ref.remove(removed_edge);
                nonce_edge_set.iter().for_each(|nonce_edge| {
                    // Avoid adding self edges in the case of cycles
                    if n == &nonce_edge.to {
                        return;
                    }
                    if removed_edge.edge_type == EdgeType::Strong {
                        let mut new_label = vec![];
                        new_label.append(&mut removed_edge.label.clone());
                        new_label.append(&mut nonce_edge.label.clone());
                        let edge = Edge {
                            edge_type: nonce_edge.edge_type,
                            label: new_label,
                            to: nonce_edge.to,
                        };
                        n_edge_set_ref.insert(edge);
                    } else {
                        let edge = Edge {
                            edge_type: EdgeType::Weak,
                            label: removed_edge.label.clone(),
                            to: nonce_edge.to,
                        };
                        n_edge_set_ref.insert(edge);
                    }
                });
            });
        }
        checked_verify!(self.invariant());
        checked_postcondition!(!self.0.contains_key(&nonce));
    }

    /// renames nonces in `self` according to `nonce_map`
    pub fn rename_nonces(&self, nonce_map: BTreeMap<Nonce, Nonce>) -> Self {
        checked_assume!(self.invariant());
        let mut new_graph = BTreeMap::new();
        for (n, es) in &self.0 {
            new_graph.insert(
                nonce_map[n],
                es.iter()
                    .map(|x| Edge {
                        edge_type: x.edge_type,
                        label: x.label.clone(),
                        to: nonce_map[&x.to],
                    })
                    .collect::<BTreeSet<_>>(),
            );
        }
        let new_borrow_graph = BorrowGraph(new_graph);
        checked_verify!(self.invariant());
        checked_verify!(new_borrow_graph.invariant());
        new_borrow_graph
    }

    /// checks if `self` covers `other`
    pub fn abstracts(&self, other: &Self) -> bool {
        self.unmatched_edges(other).values().all(|es| es.is_empty())
    }

    /// joins `other` into `self`
    pub fn join(&mut self, other: &Self) {
        for (n, es) in self.unmatched_edges(other) {
            self.0.get_mut(&n).unwrap().extend(es);
        }
    }

    /// gets all nonces that are targets of outgoing edges from `nonce`
    pub fn all_borrows(&self, nonce: Nonce) -> BTreeSet<Nonce> {
        checked_precondition!(self.0.contains_key(&nonce));
        self.0[&nonce].iter().map(|x| x.to).collect()
    }

    /// gets all nonces that are targets of outgoing edges from `nonce` that are labeled with the empty label
    pub fn nil_borrows(&self, nonce: Nonce) -> BTreeSet<Nonce> {
        checked_precondition!(self.0.contains_key(&nonce));
        self.0[&nonce]
            .iter()
            .filter(|x| x.label.is_empty())
            .map(|x| x.to)
            .collect()
    }

    /// gets all nonces that are targets of outgoing edges from `nonce` that are consistent with `label_elem`
    pub fn consistent_borrows(&self, nonce: Nonce, label_elem: T) -> BTreeSet<Nonce> {
        checked_precondition!(self.0.contains_key(&nonce));
        self.0[&nonce]
            .iter()
            .filter(|x| x.label.is_empty() || x.label[0] == label_elem)
            .map(|x| x.to)
            .collect()
    }

    /// split `edge_set` based on `pred` without cloning entries in `edge_set`
    fn split<F>(edge_set: BTreeSet<Edge<T>>, pred: F) -> (Vec<Edge<T>>, BTreeSet<Edge<T>>)
    where
        F: Copy,
        F: FnOnce(&Edge<T>) -> bool,
    {
        let mut pred_true = Vec::new();
        let mut pred_false = BTreeSet::new();
        for edge in edge_set.into_iter() {
            if pred(&edge) {
                pred_true.push(edge);
            } else {
                pred_false.insert(edge);
            }
        }
        (pred_true, pred_false)
    }

    fn unmatched_edges(&self, other: &Self) -> BTreeMap<Nonce, BTreeSet<Edge<T>>> {
        let mut unmatched_edges = BTreeMap::new();
        for (n, other_edges) in &other.0 {
            unmatched_edges.insert(n.clone(), BTreeSet::new());
            for other_edge in other_edges {
                let found_match = self.0[n]
                    .iter()
                    .any(|self_edge| self_edge.is_prefix(other_edge));
                if !found_match {
                    unmatched_edges
                        .get_mut(n)
                        .unwrap()
                        .insert(other_edge.clone());
                }
            }
        }
        unmatched_edges
    }

    fn invariant(&self) -> bool {
        self.0
            .values()
            .flatten()
            .all(|edge| self.0.contains_key(&edge.to))
            && self
                .0
                .iter()
                .all(|(n, edges)| edges.iter().all(|edge| n != &edge.to))
    }
}
