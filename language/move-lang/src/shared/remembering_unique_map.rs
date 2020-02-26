// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::{unique_map::UniqueMap, *};
use std::{collections::BTreeSet, fmt, iter::IntoIterator};

//**************************************************************************************************
// UniqueMap
//**************************************************************************************************

/// wrapper around `UniqueMap` that remembers which values were asked for in `get`
#[derive(Clone)]
pub struct RememberingUniqueMap<K: TName + Ord, V> {
    map: UniqueMap<K, V>,
    gotten_keys: BTreeSet<K>,
}

#[allow(clippy::new_without_default)]
impl<K: TName, V> RememberingUniqueMap<K, V> {
    pub fn new() -> Self {
        RememberingUniqueMap {
            map: UniqueMap::new(),
            gotten_keys: BTreeSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn add(&mut self, key: K, value: V) -> Result<(), K::Loc> {
        self.map.add(key, value)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.gotten_keys.insert(key.clone());
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.gotten_keys.insert(key.clone());
        self.map.get_mut(key)
    }

    pub fn get_loc(&mut self, key: &K) -> Option<&K::Loc> {
        self.gotten_keys.insert(key.clone());
        self.map.get_loc(key)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.gotten_keys.remove(key);
        self.map.remove(key)
    }

    pub fn map<V2, F>(self, f: F) -> RememberingUniqueMap<K, V2>
    where
        F: FnMut(K, V) -> V2,
    {
        RememberingUniqueMap {
            map: self.map.map(f),
            gotten_keys: self.gotten_keys,
        }
    }

    pub fn ref_map<V2, F>(&self, f: F) -> RememberingUniqueMap<K, V2>
    where
        F: FnMut(K, &V) -> V2,
    {
        RememberingUniqueMap {
            map: self.map.ref_map(f),
            gotten_keys: self.gotten_keys.clone(),
        }
    }

    pub fn union_with<F>(&self, other: &Self, f: F) -> Self
    where
        V: Clone,
        F: FnMut(&K, &V, &V) -> V,
    {
        RememberingUniqueMap {
            map: self.map.union_with(&other.map, f),
            gotten_keys: self
                .gotten_keys
                .union(&other.gotten_keys)
                .cloned()
                .collect(),
        }
    }

    pub fn iter(&self) -> Iter<K, V> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        self.into_iter()
    }

    pub fn maybe_from_opt_iter(
        iter: impl Iterator<Item = Option<(K, V)>>,
    ) -> Option<Result<RememberingUniqueMap<K, V>, (K::Key, K::Loc, K::Loc)>> {
        let map_res = UniqueMap::maybe_from_opt_iter(iter)?;
        Some(map_res.map(|map| RememberingUniqueMap {
            map,
            gotten_keys: BTreeSet::new(),
        }))
    }

    pub fn maybe_from_iter(
        iter: impl Iterator<Item = (K, V)>,
    ) -> Result<RememberingUniqueMap<K, V>, (K::Key, K::Loc, K::Loc)> {
        let map = UniqueMap::maybe_from_iter(iter)?;
        Ok(RememberingUniqueMap {
            map,
            gotten_keys: BTreeSet::new(),
        })
    }

    pub fn remember(self) -> BTreeSet<K> {
        self.gotten_keys
    }
}

impl<K: TName, V: PartialEq> PartialEq for RememberingUniqueMap<K, V> {
    fn eq(&self, other: &RememberingUniqueMap<K, V>) -> bool {
        self.map == other.map && self.gotten_keys == other.gotten_keys
    }
}
impl<K: TName, V: Eq> Eq for RememberingUniqueMap<K, V> {}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl<K: TName + fmt::Debug, V: fmt::Debug> fmt::Debug for RememberingUniqueMap<K, V>
where
    K::Key: fmt::Debug,
    K::Loc: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RememberingUniqueMap {{ map: {:#?}, gotten_keys: {:#?} }}",
            self.map, self.gotten_keys
        )
    }
}

//**************************************************************************************************
// IntoIter
//**************************************************************************************************

pub struct IntoIter<K: TName, V>(unique_map::IntoIter<K, V>);

impl<K: TName, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<K: TName, V> IntoIterator for RememberingUniqueMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.map.into_iter())
    }
}

//**************************************************************************************************
// Iter
//**************************************************************************************************

pub struct Iter<'a, K: TName, V>(unique_map::Iter<'a, K, V>);

impl<'a, K: TName, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, K: TName, V> IntoIterator for &'a RememberingUniqueMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let m = &self.map;
        Iter(m.into_iter())
    }
}

//**************************************************************************************************
// IterMut
//**************************************************************************************************

pub struct IterMut<'a, K: TName, V>(unique_map::IterMut<'a, K, V>);

impl<'a, K: TName, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, K: TName, V> IntoIterator for &'a mut RememberingUniqueMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let m = &mut self.map;
        IterMut(m.into_iter())
    }
}
