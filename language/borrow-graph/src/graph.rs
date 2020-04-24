// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    paths::{self, Path, PathSlice},
    references::*,
};
use mirai_annotations::{debug_checked_postcondition, debug_checked_precondition};
use std::collections::{BTreeMap, BTreeSet};

//**************************************************************************************************
// Definitions
//**************************************************************************************************

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BorrowGraph<Loc: Copy, Lbl: Clone + Ord>(BTreeMap<RefID, Ref<Loc, Lbl>>);

//**************************************************************************************************
// Impls
//**************************************************************************************************

impl<Loc: Copy, Lbl: Clone + Ord> BorrowGraph<Loc, Lbl> {
    /// creates an empty borrow graph
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// checks if the given reference is mutable or not
    pub fn is_mutable(&self, id: RefID) -> bool {
        self.0.get(&id).unwrap().mutable
    }

    /// Adds a new reference to the borrow graph
    /// Fails if the id is already in use
    pub fn new_ref(&mut self, id: RefID, mutable: bool) {
        assert!(self.0.insert(id, Ref::new(mutable)).is_none(), "{}", id.0)
    }

    /// Return the references borrowing the `id` reference
    /// The borrows are collected by first label in the borrow edge
    /// `BTreeMap<RefID, Loc>` represents all of the "full" or "epsilon" borrows (non field borrows)
    /// `BTreeMap<Lbl, BTreeMap<RefID, Loc>>)` represents the field borrows, collected over the
    /// first label
    pub fn borrowed_by(
        &self,
        id: RefID,
    ) -> (BTreeMap<RefID, Loc>, BTreeMap<Lbl, BTreeMap<RefID, Loc>>) {
        let borrowed_by = &self.0.get(&id).unwrap().borrowed_by;
        let mut full_borrows: BTreeMap<RefID, Loc> = BTreeMap::new();
        let mut field_borrows: BTreeMap<Lbl, BTreeMap<RefID, Loc>> = BTreeMap::new();
        for (borrower, edges) in &borrowed_by.0 {
            let borrower = *borrower;
            for edge in edges {
                match edge.path.get(0) {
                    None => full_borrows.insert(borrower, edge.loc),
                    Some(f) => field_borrows
                        .entry(f.clone())
                        .or_insert_with(BTreeMap::new)
                        .insert(borrower, edge.loc),
                };
            }
        }
        (full_borrows, field_borrows)
    }

    /// Return the edges between parent and child
    pub fn between_edges(&self, parent: RefID, child: RefID) -> Vec<(Loc, Path<Lbl>, bool)> {
        let edges = &self.0.get(&parent).unwrap().borrowed_by.0[&child];
        edges
            .iter()
            .map(|edge| (edge.loc, edge.path.clone(), edge.strong))
            .collect()
    }

    /// Return the outgoing edges from id
    pub fn out_edges(&self, id: RefID) -> Vec<(Loc, Path<Lbl>, bool, RefID)> {
        let mut returned_edges = vec![];
        let borrowed_by = &self.0.get(&id).unwrap().borrowed_by;
        for (borrower, edges) in &borrowed_by.0 {
            let borrower = *borrower;
            for edge in edges {
                returned_edges.push((edge.loc, edge.path.clone(), edge.strong, borrower));
            }
        }
        returned_edges
    }

    /// Return the incoming edges into id
    pub fn in_edges(&self, id: RefID) -> Vec<(Loc, RefID, Path<Lbl>, bool)> {
        let mut returned_edges = vec![];
        let borrows_from = &self.0.get(&id).unwrap().borrows_from;
        for src in borrows_from {
            for edge in self.between_edges(*src, id) {
                returned_edges.push((edge.0, *src, edge.1, edge.2));
            }
        }
        returned_edges
    }

    //**********************************************************************************************
    // Edges/Borrows
    //**********************************************************************************************

    /// Add a strong (exact) epsilon borrow from `parent_id` to `child_id`
    pub fn add_strong_borrow(&mut self, loc: Loc, parent_id: RefID, child_id: RefID) {
        self.factor(parent_id, loc, vec![], child_id)
    }

    /// Add a strong (exact) field borrow from `parent_id` to `child_id` at field `field`
    pub fn add_strong_field_borrow(
        &mut self,
        loc: Loc,
        parent_id: RefID,
        field: Lbl,
        child_id: RefID,
    ) {
        self.factor(parent_id, loc, vec![field], child_id)
    }

    /// Add a weak (prefix) epsilon borrow from `parent_id` to `child_id`
    /// i.e. `child_id` might be borrowing from ANY field in `parent_id`
    pub fn add_weak_borrow(&mut self, loc: Loc, parent_id: RefID, child_id: RefID) {
        self.add_path(parent_id, loc, false, vec![], child_id)
    }

    /// Add a weak (prefix) field borrow from `parent_id` to `child_id` at field `field`
    /// i.e. `child_id` might be borrowing from ANY field in `parent_id` rooted at `field`
    pub fn add_weak_field_borrow(
        &mut self,
        loc: Loc,
        parent_id: RefID,
        field: Lbl,
        child_id: RefID,
    ) {
        self.add_path(parent_id, loc, false, vec![field], child_id)
    }

    fn add_edge(&mut self, parent_id: RefID, edge: BorrowEdge<Loc, Lbl>, child_id: RefID) {
        assert!(parent_id != child_id);
        let parent = self.0.get_mut(&parent_id).unwrap();
        parent
            .borrowed_by
            .0
            .entry(child_id)
            .or_insert_with(BTreeSet::new)
            .insert(edge);
        let child = self.0.get_mut(&child_id).unwrap();
        child.borrows_from.insert(parent_id);
    }

    fn add_path(
        &mut self,
        parent_id: RefID,
        loc: Loc,
        strong: bool,
        path: Path<Lbl>,
        child_id: RefID,
    ) {
        let edge = BorrowEdge { strong, path, loc };
        self.add_edge(parent_id, edge, child_id)
    }

    fn factor(&mut self, parent_id: RefID, loc: Loc, path: Path<Lbl>, intermediate_id: RefID) {
        debug_checked_precondition!(self.check_invariant());
        let parent = self.0.get_mut(&parent_id).unwrap();
        let mut needs_factored = vec![];
        for (child_id, parent_to_child_edges) in &parent.borrowed_by.0 {
            for parent_to_child_edge in parent_to_child_edges {
                if paths::leq(&path, &parent_to_child_edge.path) {
                    let factored_edge = (*child_id, parent_to_child_edge.clone());
                    needs_factored.push(factored_edge);
                }
            }
        }

        let mut cleanup_ids = BTreeSet::new();
        for (child_id, parent_to_child_edge) in &needs_factored {
            let parent_to_child_edges = parent.borrowed_by.0.get_mut(child_id).unwrap();
            assert!(parent_to_child_edges.remove(parent_to_child_edge));
            if parent_to_child_edges.is_empty() {
                assert!(parent.borrowed_by.0.remove(child_id).is_some());
                cleanup_ids.insert(child_id);
            }
        }

        for child_id in cleanup_ids {
            assert!(self
                .0
                .get_mut(child_id)
                .unwrap()
                .borrows_from
                .remove(&parent_id));
        }

        for (child_id, parent_to_child_edge) in needs_factored {
            let (_, intermediate_to_child_suffix) = paths::factor(&path, parent_to_child_edge.path);
            self.add_path(
                intermediate_id,
                parent_to_child_edge.loc,
                parent_to_child_edge.strong,
                intermediate_to_child_suffix,
                child_id,
            )
        }
        self.add_path(
            parent_id,
            loc,
            /* strong */ true,
            path,
            intermediate_id,
        );
        debug_checked_postcondition!(self.check_invariant());
    }

    //**********************************************************************************************
    // Release
    //**********************************************************************************************

    /// Remove reference `id` from the graph
    /// Fixes any transitive borrows, so if `parent` borrowed by `id` borrowed by `child`
    /// After the release, `parent` borrowed by `child`
    pub fn release(&mut self, id: RefID) {
        debug_checked_precondition!(self.check_invariant());
        let Ref {
            borrowed_by,
            borrows_from,
            ..
        } = self.0.remove(&id).unwrap();
        for parent_ref_id in borrows_from.into_iter() {
            let parent = self.0.get_mut(&parent_ref_id).unwrap();
            let parent_edges = parent.borrowed_by.0.remove(&id).unwrap();
            for parent_edge in parent_edges {
                for (child_ref_id, child_edges) in &borrowed_by.0 {
                    for child_edge in child_edges {
                        self.splice_out_intermediate(
                            parent_ref_id,
                            &parent_edge,
                            *child_ref_id,
                            child_edge,
                        )
                    }
                }
            }
        }
        for child_ref_id in borrowed_by.0.keys() {
            let child = self.0.get_mut(&child_ref_id).unwrap();
            child.borrows_from.remove(&id);
        }
        debug_checked_postcondition!(self.check_invariant());
    }

    fn splice_out_intermediate(
        &mut self,
        parent_id: RefID,
        parent_to_intermediate: &BorrowEdge<Loc, Lbl>,
        child_id: RefID,
        intermediate_to_child: &BorrowEdge<Loc, Lbl>,
    ) {
        // dont add in an edge if releasing from a cycle
        if parent_id == child_id {
            return;
        }

        let path = if parent_to_intermediate.strong {
            paths::append(&parent_to_intermediate.path, &intermediate_to_child.path)
        } else {
            parent_to_intermediate.path.clone()
        };
        let strong = parent_to_intermediate.strong && intermediate_to_child.strong;
        let loc = intermediate_to_child.loc;
        let parent_to_child = BorrowEdge { strong, path, loc };
        self.add_edge(parent_id, parent_to_child, child_id)
    }

    //**********************************************************************************************
    // Subsumes/weakens
    //**********************************************************************************************

    /// checks if `self` covers `other`
    pub fn leq(&self, other: &Self) -> bool {
        self.unmatched_edges(other).is_empty()
    }

    fn unmatched_edges(&self, other: &Self) -> BTreeMap<RefID, BorrowEdges<Loc, Lbl>> {
        let mut unmatched_edges = BTreeMap::new();
        for (parent_id, other_ref) in &other.0 {
            let self_ref = &self.0[parent_id];
            let self_borrowed_by = &self_ref.borrowed_by.0;
            for (child_id, other_edges) in &other_ref.borrowed_by.0 {
                for other_edge in other_edges {
                    let found_match = self_borrowed_by
                        .get(child_id)
                        .map(|parent_to_child| {
                            parent_to_child
                                .iter()
                                .any(|self_edge| self_edge.leq(other_edge))
                        })
                        .unwrap_or(false);
                    if !found_match {
                        assert!(parent_id != child_id);
                        unmatched_edges
                            .entry(*parent_id)
                            .or_insert_with(BorrowEdges::new)
                            .0
                            .entry(*child_id)
                            .or_insert_with(BTreeSet::new)
                            .insert(other_edge.clone());
                    }
                }
            }
        }
        unmatched_edges
    }

    //**********************************************************************************************
    // Remap
    //**********************************************************************************************

    /// Utility for remapping the reference ids according the `id_map` provided
    /// If it is not in the map, the id remains the same
    pub fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        debug_checked_precondition!(self.check_invariant());
        for info in self.0.values_mut() {
            info.remap_refs(id_map);
        }
        for (old, new) in id_map {
            if let Some(info) = self.0.remove(old) {
                self.0.insert(*new, info);
            }
        }
        debug_checked_postcondition!(self.check_invariant());
    }

    //**********************************************************************************************
    // Joins
    //**********************************************************************************************

    /// Joins other into self
    /// It adds only 'unmatched' edges from other into self, i.e. for any edge in other, if there
    /// is an edge in self that is <= than that edge, it is not added.
    pub fn join(&self, other: &Self) -> Self {
        debug_checked_precondition!(self.check_invariant());
        debug_checked_precondition!(other.check_invariant());
        debug_checked_precondition!(self.0.keys().all(|id| other.0.contains_key(id)));
        debug_checked_precondition!(other.0.keys().all(|id| self.0.contains_key(id)));

        let mut joined = self.clone();
        for (parent_id, unmatched_borrowed_by) in self.unmatched_edges(other) {
            for (child_id, unmatched_edges) in unmatched_borrowed_by.0 {
                for unmatched_edge in unmatched_edges {
                    joined.add_edge(parent_id, unmatched_edge, child_id);
                }
            }
        }
        debug_checked_postcondition!(joined.check_invariant());
        joined
    }

    //**********************************************************************************************
    // Consistency/Invariants
    //**********************************************************************************************

    fn check_invariant(&self) -> bool {
        self.id_consistency() && self.edge_consistency() && self.no_self_loops()
    }

    /// Checks at all ids in edges are contained in the borrow map itself, i.e. that each id
    /// corresponds to a reference
    fn id_consistency(&self) -> bool {
        let contains_id = |id| self.0.contains_key(id);
        self.0.values().all(|r| {
            r.borrowed_by.0.keys().all(contains_id) && r.borrows_from.iter().all(contains_id)
        })
    }

    /// Checks that for every edge in borrowed_by there is a flipped edge in borrows_from
    /// And vice versa
    //// i.e. verifies the "back edges" in the borrow graph
    fn edge_consistency(&self) -> bool {
        let parent_to_child_consistency =
            |cur_parent, child| self.0[child].borrows_from.contains(cur_parent);
        let child_to_parent_consistency =
            |cur_child, parent| self.0[parent].borrowed_by.0.contains_key(cur_child);
        self.0.iter().all(|(id, r)| {
            r.borrowed_by
                .0
                .keys()
                .all(|c| parent_to_child_consistency(id, c))
                && r.borrows_from
                    .iter()
                    .all(|p| child_to_parent_consistency(id, p))
        })
    }

    /// Checks that no reference borrows from itself
    fn no_self_loops(&self) -> bool {
        self.0.iter().all(|(id, r)| {
            r.borrowed_by.0.keys().all(|to_id| id != to_id)
                && r.borrows_from.iter().all(|from_id| id != from_id)
        })
    }

    //**********************************************************************************************
    // Util
    //**********************************************************************************************

    /// Checks if the current reference is in the graph
    pub fn contains_id(&self, ref_id: RefID) -> bool {
        self.0.contains_key(&ref_id)
    }

    /// Returns all ref ids in the map
    pub fn all_refs(&self) -> BTreeSet<RefID> {
        self.0.keys().cloned().collect()
    }

    /// Prints out a view of the borrow graph
    #[allow(dead_code)]
    pub fn display(&self)
    where
        Lbl: std::fmt::Display,
    {
        fn path_to_string<Lbl: std::fmt::Display>(p: &PathSlice<Lbl>) -> String {
            p.iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(".")
        }

        for (id, ref_info) in &self.0 {
            if ref_info.borrowed_by.0.is_empty() && ref_info.borrows_from.is_empty() {
                println!("{}", id.0);
            }
            for (borrower, edges) in &ref_info.borrowed_by.0 {
                for edge in edges {
                    let edisp = if edge.strong { "=" } else { "-" };
                    println!(
                        "{} {}{}{}> {}",
                        id.0,
                        edisp,
                        path_to_string(&edge.path),
                        edisp,
                        borrower.0,
                    );
                }
            }
            for parent in &ref_info.borrows_from {
                println!("{} <- {}", parent.0, id.0);
            }
        }
    }
}
