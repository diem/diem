// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::{collections::BTreeMap, fmt::Debug, iter::IntoIterator};

//**************************************************************************************************
// UniqueMap
//**************************************************************************************************

/// Unique wrapper around `BTreeMap` that throws on duplicate inserts
#[derive(Default, Clone, Debug)]
pub struct UniqueMap<K: TName, V>(pub(crate) BTreeMap<K::Key, (K::Loc, V)>);

impl<K: TName, V> UniqueMap<K, V> {
    pub fn new() -> Self {
        UniqueMap(BTreeMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, key: K, value: V) -> Result<(), (K, K::Loc)> {
        if let Some(old_loc) = self.get_loc(&key) {
            return Err((key, *old_loc));
        }
        let (loc, key_) = key.drop_loc();
        let old_value = self.0.insert(key_, (loc, value));
        assert!(old_value.is_none());
        Ok(())
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.contains_key_(&key.borrow().1)
    }

    pub fn contains_key_(&self, key_: &K::Key) -> bool {
        self.0.contains_key(key_)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_(&key.borrow().1)
    }

    pub fn get_(&self, key_: &K::Key) -> Option<&V> {
        self.0.get(key_).map(|loc_value| &loc_value.1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut_(&key.borrow().1)
    }

    pub fn get_mut_(&mut self, key_: &K::Key) -> Option<&mut V> {
        self.0.get_mut(key_).map(|loc_value| &mut loc_value.1)
    }

    pub fn get_loc(&self, key: &K) -> Option<&K::Loc> {
        self.get_loc_(&key.borrow().1)
    }

    pub fn get_loc_(&self, key_: &K::Key) -> Option<&K::Loc> {
        self.0.get(key_).map(|loc_value| &loc_value.0)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_(&key.borrow().1)
    }

    pub fn remove_(&mut self, key_: &K::Key) -> Option<V> {
        self.0.remove(key_).map(|loc_value| loc_value.1)
    }

    pub fn map<V2, F>(self, mut f: F) -> UniqueMap<K, V2>
    where
        F: FnMut(K, V) -> V2,
    {
        UniqueMap(
            self.0
                .into_iter()
                .map(|(k_, (loc, v))| {
                    let v2 = f(K::add_loc(loc, k_.clone()), v);
                    (k_, (loc, v2))
                })
                .collect(),
        )
    }

    pub fn filter_map<V2, F>(self, mut f: F) -> UniqueMap<K, V2>
    where
        F: FnMut(K, V) -> Option<V2>,
    {
        UniqueMap(
            self.0
                .into_iter()
                .filter_map(|(k_, (loc, v))| {
                    let v2_opt = f(K::add_loc(loc, k_.clone()), v);
                    v2_opt.map(|v2| (k_, (loc, v2)))
                })
                .collect(),
        )
    }

    pub fn ref_map<V2, F>(&self, mut f: F) -> UniqueMap<K, V2>
    where
        F: FnMut(K, &V) -> V2,
    {
        UniqueMap(
            self.0
                .iter()
                .map(|(k_, loc_v)| {
                    let loc = loc_v.0;
                    let v = &loc_v.1;
                    let k = K::add_loc(loc, k_.clone());
                    let v2 = f(k, v);
                    (k_.clone(), (loc, v2))
                })
                .collect(),
        )
    }

    pub fn union_with<F>(&self, other: &Self, mut f: F) -> Self
    where
        V: Clone,
        F: FnMut(&K, &V, &V) -> V,
    {
        let mut joined = Self::new();
        for (loc, k_, v1) in self.iter() {
            let k = K::add_loc(loc, k_.clone());
            let v = match other.get(&k) {
                None => v1.clone(),
                Some(v2) => f(&k, v1, v2),
            };
            assert!(joined.add(k, v).is_ok())
        }
        for (loc, k_, v2) in other.iter() {
            if !joined.contains_key_(&k_) {
                let k = K::add_loc(loc, k_.clone());
                assert!(joined.add(k, v2.clone()).is_ok())
            }
        }
        joined
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.into_iter()
    }

    pub fn key_cloned_iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.into_iter()
            .map(|(loc, k_, v)| (K::add_loc(loc, k_.clone()), v))
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.into_iter()
    }

    pub fn maybe_from_opt_iter(
        iter: impl Iterator<Item = Option<(K, V)>>,
    ) -> Option<Result<UniqueMap<K, V>, (K::Key, K::Loc, K::Loc)>> {
        // TODO remove collect in favor of more efficient impl
        Some(Self::maybe_from_iter(
            iter.collect::<Option<Vec<_>>>()?.into_iter(),
        ))
    }

    pub fn maybe_from_iter(
        iter: impl Iterator<Item = (K, V)>,
    ) -> Result<UniqueMap<K, V>, (K::Key, K::Loc, K::Loc)> {
        let mut m = Self::new();
        for (k, v) in iter {
            if let Err((k, old_loc)) = m.add(k, v) {
                let (loc, key_) = k.drop_loc();
                return Err((key_, loc, old_loc));
            }
        }
        Ok(m)
    }
}

impl<K: TName, V: PartialEq> PartialEq for UniqueMap<K, V> {
    fn eq(&self, other: &UniqueMap<K, V>) -> bool {
        self.iter()
            .all(|(_, k_, v1)| other.get_(&k_).map(|v2| v1 == v2).unwrap_or(false))
            && other.iter().all(|(_, k_, _)| self.contains_key_(&k_))
    }
}
impl<K: TName, V: Eq> Eq for UniqueMap<K, V> {}

//**************************************************************************************************
// IntoIter
//**************************************************************************************************

pub struct IntoIter<K: TName, V>(
    std::iter::Map<
        std::collections::btree_map::IntoIter<K::Key, (K::Loc, V)>,
        fn((K::Key, (K::Loc, V))) -> (K, V),
    >,
    usize,
);

impl<K: TName, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 0 {
            self.1 -= 1;
        }
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<K: TName, V> IntoIterator for UniqueMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        IntoIter(
            self.0.into_iter().map(|(k_, loc_v)| {
                let loc = loc_v.0;
                let v = loc_v.1;
                let k = K::add_loc(loc, k_);
                (k, v)
            }),
            len,
        )
    }
}

//**************************************************************************************************
// Iter
//**************************************************************************************************

pub struct Iter<'a, K: TName, V>(
    std::iter::Map<
        std::collections::btree_map::Iter<'a, K::Key, (K::Loc, V)>,
        fn((&'a K::Key, &'a (K::Loc, V))) -> (K::Loc, &'a K::Key, &'a V),
    >,
    usize,
);

impl<'a, K: TName, V> Iterator for Iter<'a, K, V> {
    type Item = (K::Loc, &'a K::Key, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 0 {
            self.1 -= 1;
        }
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.1, Some(self.1))
    }
}

impl<'a, K: TName, V> IntoIterator for &'a UniqueMap<K, V> {
    type Item = (K::Loc, &'a K::Key, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let fix = |(k_, loc_v): (&'a K::Key, &'a (K::Loc, V))| -> (K::Loc, &'a K::Key, &'a V) {
            let loc = loc_v.0;
            let v = &loc_v.1;
            (loc, k_, v)
        };
        Iter(self.0.iter().map(fix), self.len())
    }
}

//**************************************************************************************************
// IterMut
//**************************************************************************************************

pub struct IterMut<'a, K: TName, V>(
    std::iter::Map<
        std::collections::btree_map::IterMut<'a, K::Key, (K::Loc, V)>,
        fn((&'a K::Key, &'a mut (K::Loc, V))) -> (K::Loc, &'a K::Key, &'a mut V),
    >,
    usize,
);

impl<'a, K: TName, V> Iterator for IterMut<'a, K, V> {
    type Item = (K::Loc, &'a K::Key, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 > 0 {
            self.1 -= 1;
        }
        self.0.next()
    }
}

impl<'a, K: TName, V> IntoIterator for &'a mut UniqueMap<K, V> {
    type Item = (K::Loc, &'a K::Key, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let len = self.len();
        let fix =
            |(k_, loc_v): (&'a K::Key, &'a mut (K::Loc, V))| -> (K::Loc, &'a K::Key, &'a mut V) {
                let loc = loc_v.0;
                let v = &mut loc_v.1;
                (loc, k_, v)
            };
        IterMut(self.0.iter_mut().map(fix), len)
    }
}
