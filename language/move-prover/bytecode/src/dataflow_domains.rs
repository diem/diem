// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines traits and representations of domains used in dataflow analysis.

use im::{ordmap, ordset, OrdMap, OrdSet};
use itertools::Itertools;
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    ops::{Deref, DerefMut},
};

// ================================================================================================
// Abstract Domains

/// Represents the abstract outcome of a join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinResult {
    /// The left operand subsumes the right operand: L union R == L.
    Unchanged,
    /// The left operand does not subsume the right one and was changed as part of the join.
    Changed,
}

impl JoinResult {
    /// Build the least upper bound of two join results, where `Unchanged` is bottom element of the
    /// semilattice.
    pub fn combine(self, other: JoinResult) -> JoinResult {
        use JoinResult::*;
        match (self, other) {
            (Unchanged, Unchanged) => Unchanged,
            _ => Changed,
        }
    }
}

/// A trait to be implemented by domains which support a join.
pub trait AbstractDomain {
    fn join(&mut self, other: &Self) -> JoinResult;
}

// ================================================================================================
// Predefined Domain Types

// As the underlying implementation of the below types we use the collections from the `im`(mutable)
// crate (`im::OrdSet` and `im::OrdMap`), a representation which supports structure sharing.
// This is important because in data flow analysis we often refine a set or map
// value in each step of the analysis, e.g. adding a single element to a larger collection, while
// the original collection still need to be available at the previous program point.

// ------------------------------------------------------------------------------------------------
// Set Type

/// Implements a set domain.
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct SetDomain<E: Ord + Clone>(OrdSet<E>);

impl<E: Ord + Clone> Default for SetDomain<E> {
    fn default() -> Self {
        Self(OrdSet::default())
    }
}

impl<E: Ord + Clone> From<OrdSet<E>> for SetDomain<E> {
    fn from(ord_set: OrdSet<E>) -> Self {
        Self(ord_set)
    }
}

impl<E: Ord + Clone> AsRef<OrdSet<E>> for SetDomain<E> {
    fn as_ref(&self) -> &OrdSet<E> {
        &self.0
    }
}

impl<E: Ord + Clone> Borrow<OrdSet<E>> for SetDomain<E> {
    fn borrow(&self) -> &OrdSet<E> {
        self.as_ref()
    }
}

impl<E: Ord + Clone> Deref for SetDomain<E> {
    type Target = OrdSet<E>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Ord + Clone> DerefMut for SetDomain<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: Ord + Clone + Debug> Debug for SetDomain<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<E: Ord + Clone> std::iter::FromIterator<E> for SetDomain<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        let mut s = SetDomain::default();
        for e in iter {
            s.insert(e);
        }
        s
    }
}

impl<E: Ord + Clone> std::iter::IntoIterator for SetDomain<E> {
    type Item = E;
    type IntoIter = im::ordset::ConsumingIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E: Ord + Clone> AbstractDomain for SetDomain<E> {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut change = JoinResult::Unchanged;
        for e in other.iter() {
            if self.insert(e.clone()).is_none() {
                change = JoinResult::Changed;
            }
        }
        change
    }
}

impl<E: Ord + Clone> From<BTreeSet<E>> for SetDomain<E> {
    fn from(s: BTreeSet<E>) -> Self {
        s.into_iter().collect()
    }
}

impl<E: Ord + Clone> SetDomain<E> {
    pub fn singleton(e: E) -> Self {
        ordset!(e).into()
    }

    /// Implements set difference, which is not following standard APIs for rust sets in OrdSet
    pub fn difference<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &'a E> {
        self.iter().filter(move |e| !other.contains(e))
    }

    /// Implements is_disjoint which is not available in OrdSet
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.iter().all(move |e| !other.contains(e))
    }
}

// ------------------------------------------------------------------------------------------------
// Map Type

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct MapDomain<K: Ord, V: AbstractDomain>(OrdMap<K, V>);

impl<K: Ord + Clone, V: AbstractDomain + Clone> Default for MapDomain<K, V> {
    fn default() -> Self {
        Self(OrdMap::default())
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> From<OrdMap<K, V>> for MapDomain<K, V> {
    fn from(ord_map: OrdMap<K, V>) -> Self {
        Self(ord_map)
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> AsRef<OrdMap<K, V>> for MapDomain<K, V> {
    fn as_ref(&self) -> &OrdMap<K, V> {
        &self.0
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> Borrow<OrdMap<K, V>> for MapDomain<K, V> {
    fn borrow(&self) -> &OrdMap<K, V> {
        self.as_ref()
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> Deref for MapDomain<K, V> {
    type Target = OrdMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> DerefMut for MapDomain<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<K: Ord + Clone + Debug, V: AbstractDomain + Clone + Debug> Debug for MapDomain<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> std::iter::FromIterator<(K, V)>
    for MapDomain<K, V>
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut s = MapDomain::default();
        for (k, v) in iter {
            s.insert(k, v);
        }
        s
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> std::iter::IntoIterator for MapDomain<K, V> {
    type Item = (K, V);
    type IntoIter = im::ordmap::ConsumingIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> AbstractDomain for MapDomain<K, V> {
    fn join(&mut self, other: &Self) -> JoinResult {
        let mut change = JoinResult::Unchanged;
        for (k, v) in other.iter() {
            change = change.combine(self.insert_join(k.clone(), v.clone()));
        }
        change
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> From<BTreeMap<K, V>> for MapDomain<K, V> {
    fn from(m: BTreeMap<K, V>) -> Self {
        m.into_iter().collect()
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone> MapDomain<K, V> {
    /// Construct a singleton map.
    pub fn singleton(k: K, v: V) -> MapDomain<K, V> {
        (ordmap! {k => v}).into()
    }

    /// Join `v` with self[k] if `k` is bound, insert `v` otherwise
    pub fn insert_join(&mut self, k: K, v: V) -> JoinResult {
        let mut change = JoinResult::Unchanged;
        self.0
            .entry(k)
            .and_modify(|old_v| {
                change = old_v.join(&v);
            })
            .or_insert_with(|| {
                change = JoinResult::Changed;
                v
            });
        change
    }
}

impl<K: Ord + Clone, V: AbstractDomain + Clone + PartialEq> MapDomain<K, V> {
    /// Updates the values in the range of the map using the given function. Notice
    /// that with other kind of map representations we would use `iter_mut` for this,
    /// but this is not available in OrdMap for obvious reasons (because entries are shared),
    /// so we need to use this pattern here instead.
    pub fn update_values(&mut self, mut f: impl FnMut(&mut V)) {
        // Commpute the key-values which actually changed. If the change is small, we preserve
        // structure sharing.
        let new_values = self
            .iter()
            .filter_map(|(k, v)| {
                let mut v_new = v.clone();
                f(&mut v_new);
                if v != &v_new {
                    Some((k.clone(), v_new))
                } else {
                    None
                }
            })
            .collect_vec();
        self.extend(new_values.into_iter());
    }
}
