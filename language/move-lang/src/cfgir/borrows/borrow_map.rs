// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//**************************************************************************************************
// Borrow Map
//**************************************************************************************************

use crate::shared::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefID(usize);

impl RefID {
    pub const fn new(x: usize) -> Self {
        RefID(x)
    }
}

pub type LocRefID = Spanned<RefID>;

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub type PathSlice = [String];
pub type Path = Vec<String>;

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
struct BorrowEdge_ {
    strong: bool,
    path: Path,
}
type BorrowEdge = Spanned<BorrowEdge_>;

#[derive(Clone, Debug, PartialEq)]
struct BorrowEdges(BTreeMap<RefID, BTreeSet<BorrowEdge>>);

#[derive(Clone, Debug, PartialEq)]
struct Ref {
    // x is borrowed by y
    borrowed_by: BorrowEdges,
    // y borrows from x
    borrows_from: BTreeSet<RefID>,
    // true if mutable, false otherwise
    mutable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorrowMap(BTreeMap<RefID, Ref>);

//**************************************************************************************************
// Impls
//**************************************************************************************************

fn leq(lhs: &PathSlice, rhs: &PathSlice) -> bool {
    lhs.len() <= rhs.len() && lhs.iter().zip(rhs).all(|(l, r)| l == r)
}

fn factor_path(lhs: &PathSlice, mut rhs: Path) -> (Path, Path) {
    assert!(leq(lhs, &rhs));
    let suffix = rhs.split_off(lhs.len());
    (rhs, suffix)
}

fn append_path(lhs: &PathSlice, rhs: &PathSlice) -> Path {
    let mut path = lhs.to_owned();
    path.append(&mut rhs.to_owned());
    path
}

impl BorrowEdges {
    fn new() -> Self {
        Self(BTreeMap::new())
    }
}

impl Ref {
    fn new(mutable: bool) -> Self {
        let borrowed_by = BorrowEdges::new();
        let borrows_from = BTreeSet::new();
        Self {
            borrowed_by,
            borrows_from,
            mutable,
        }
    }
}

impl BorrowMap {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn is_mutable(&self, id: RefID) -> bool {
        self.0.get(&id).unwrap().mutable
    }

    pub fn new_ref(&mut self, id: RefID, mutable: bool) {
        assert!(self.0.insert(id, Ref::new(mutable)).is_none(), "{}", id.0)
    }

    pub fn borrowed_by(
        &self,
        id: RefID,
    ) -> (BTreeSet<LocRefID>, BTreeMap<&str, BTreeSet<LocRefID>>) {
        let borrowed_by = &self.0.get(&id).unwrap().borrowed_by;
        let mut full_borrows: BTreeSet<LocRefID> = BTreeSet::new();
        let mut field_borrows: BTreeMap<&str, BTreeSet<LocRefID>> = BTreeMap::new();
        for (borrower, edges) in &borrowed_by.0 {
            let borrower = *borrower;
            let edges: &BTreeSet<BorrowEdge> = edges;
            for sp!(loc, edge_) in edges {
                match edge_.path.get(0) {
                    None => full_borrows.insert(sp(*loc, borrower)),
                    Some(f) => field_borrows
                        .entry(f)
                        .or_insert_with(BTreeSet::new)
                        .insert(sp(*loc, borrower)),
                };
            }
        }
        (full_borrows, field_borrows)
    }

    //**********************************************************************************************
    // Edges/Borrows
    //**********************************************************************************************

    fn add_edge(&mut self, parent_id: RefID, edge: BorrowEdge, child_id: RefID) {
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

    fn add_path(&mut self, parent_id: RefID, loc: Loc, strong: bool, path: Path, child_id: RefID) {
        let edge = sp(loc, BorrowEdge_ { strong, path });
        self.add_edge(parent_id, edge, child_id)
    }

    fn factor(
        &mut self,
        parent_id: RefID,
        loc: Loc,
        strong: bool,
        path: Path,
        intermediate_id: RefID,
    ) {
        let parent = self.0.get_mut(&parent_id).unwrap();
        let mut needs_factored = vec![];
        for (child_id, parent_to_child_edges) in &parent.borrowed_by.0 {
            for parent_to_child_edge in parent_to_child_edges {
                if leq(&path, &parent_to_child_edge.value.path) {
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

        for (child_id, sp!(cloc, parent_to_child_edge_)) in needs_factored {
            let (_, intermediate_to_child_suffix) = factor_path(&path, parent_to_child_edge_.path);
            self.add_path(
                intermediate_id,
                cloc,
                parent_to_child_edge_.strong,
                intermediate_to_child_suffix,
                child_id,
            )
        }
        self.add_path(parent_id, loc, strong, path, intermediate_id);
    }

    pub fn add_borrow(&mut self, loc: Loc, parent_id: RefID, child_id: RefID) {
        self.factor(parent_id, loc, false, vec![], child_id)
    }

    pub fn add_copy(&mut self, loc: Loc, parent_id: RefID, copier_id: RefID) {
        self.factor(parent_id, loc, true, vec![], copier_id)
    }

    pub fn add_field_borrow(&mut self, loc: Loc, parent_id: RefID, field: String, child_id: RefID) {
        self.factor(parent_id, loc, true, vec![field], child_id)
    }

    pub fn add_weak_field_borrow(
        &mut self,
        loc: Loc,
        parent_id: RefID,
        field: String,
        child_id: RefID,
    ) {
        self.factor(parent_id, loc, false, vec![field], child_id)
    }

    //**********************************************************************************************
    // Release
    //**********************************************************************************************

    fn splice_out_intermediate(
        &mut self,
        parent_id: RefID,
        sp!(_, parent_to_intermediate_): &BorrowEdge,
        child_id: RefID,
        sp!(loc, intermediate_to_child_): &BorrowEdge,
    ) {
        let path = if parent_to_intermediate_.strong {
            append_path(&parent_to_intermediate_.path, &intermediate_to_child_.path)
        } else {
            parent_to_intermediate_.path.clone()
        };
        let strong = parent_to_intermediate_.strong && intermediate_to_child_.strong;
        let parent_to_child = sp(*loc, BorrowEdge_ { strong, path });
        self.add_edge(parent_id, parent_to_child, child_id)
    }

    pub fn release(&mut self, id: RefID) {
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
    }

    //**********************************************************************************************
    // Util
    //**********************************************************************************************

    pub fn contains_id(&self, ref_id: RefID) -> bool {
        self.0.contains_key(&ref_id)
    }

    pub fn all_refs(&self) -> BTreeSet<RefID> {
        self.0.keys().cloned().collect()
    }

    #[allow(dead_code)]
    pub fn display(&self) {
        fn path_to_string(p: &[String]) -> String {
            if p.is_empty() {
                return "".to_string();
            }

            let mut res = p.get(0).unwrap().to_string();
            for idx in 1..p.len() {
                res = format!("{}.{}", res, p.get(idx).unwrap());
            }
            res
        }

        for (id, ref_info) in &self.0 {
            if ref_info.borrowed_by.0.is_empty() && ref_info.borrows_from.is_empty() {
                println!("{}", id.0);
            }
            for (borrower, edges) in &ref_info.borrowed_by.0 {
                for sp!(_, edge_) in edges {
                    let edisp = if edge_.strong { "=" } else { "-" };
                    println!(
                        "{} {}{}{}> {}",
                        id.0,
                        edisp,
                        path_to_string(&edge_.path),
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

//**********************************************************************************************
// Remap
//**********************************************************************************************

pub fn remap_set(set: &mut BTreeSet<RefID>, id_map: &BTreeMap<RefID, RefID>) {
    for (old, new) in id_map {
        if set.remove(&old) {
            set.insert(*new);
        }
    }
}

impl BorrowEdges {
    fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        for (old, new) in id_map {
            if let Some(edges) = self.0.remove(old) {
                self.0.insert(*new, edges);
            }
        }
    }
}

impl Ref {
    fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        self.borrowed_by.remap_refs(id_map);
        remap_set(&mut self.borrows_from, id_map)
    }
}

impl BorrowMap {
    pub fn remap_refs(&mut self, id_map: &BTreeMap<RefID, RefID>) {
        for info in self.0.values_mut() {
            info.remap_refs(id_map);
        }
        for (old, new) in id_map {
            if let Some(info) = self.0.remove(old) {
                self.0.insert(*new, info);
            }
        }
    }
}

//**********************************************************************************************
// Joins
//**********************************************************************************************

pub fn union_sets<T: Ord + Clone>(s1: &BTreeSet<T>, s2: &BTreeSet<T>) -> BTreeSet<T> {
    s1.union(&s2).cloned().collect()
}

pub fn union_map<K: Ord + Clone, V: Clone, F: FnMut(&K, &V, &V) -> V>(
    m1: &BTreeMap<K, V>,
    m2: &BTreeMap<K, V>,
    mut f: F,
) -> BTreeMap<K, V> {
    let mut joined = BTreeMap::new();
    for (k, v1) in m1 {
        let v = match m2.get(k) {
            None => v1.clone(),
            Some(v2) => f(k, v1, v2),
        };
        assert!(joined.insert(k.clone(), v).is_none())
    }
    for (k, v2) in m2.iter() {
        if !joined.contains_key(&k) {
            assert!(joined.insert(k.clone(), v2.clone()).is_none())
        }
    }
    joined
}

impl BorrowEdges {
    fn join(&self, other: &Self) -> Self {
        Self(union_map(&self.0, &other.0, |_, s1, s2| union_sets(s1, s2)))
    }
}

impl Ref {
    fn join(&self, other: &Self) -> Self {
        let borrowed_by = self.borrowed_by.join(&other.borrowed_by);
        let borrows_from = union_sets(&self.borrows_from, &other.borrows_from);
        assert!(
            self.mutable == other.mutable,
            "ICE type checking or freezing failed"
        );
        let mutable = self.mutable;

        Self {
            borrowed_by,
            borrows_from,
            mutable,
        }
    }
}

impl BorrowMap {
    pub fn join(&self, other: &Self) -> Self {
        Self(union_map(&self.0, &other.0, |_, v1, v2| v1.join(v2)))
    }
}
