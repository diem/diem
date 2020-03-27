// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::Arena;
use std::{
    borrow::Borrow,
    collections::hash_map::{Entry, HashMap},
    hash::Hash,
    sync::Mutex,
};

/// The most common case of `CacheMap`, where references to stored values are handed out.
pub type CacheRefMap<'a, K, V> = CacheMap<'a, K, V, &'a V>;

/// A map custom designed for Move VM caches. Allocations are done in an Arena instead of directly
/// in a hash table, which allows for new entries to be added while existing entries are borrowed
/// out.
///
/// TODO: Entry-like API? Current one is somewhat awkward to use.
/// TODO: eviction -- how to do it safely?
/// TODO: should the map own the arena?
/// TODO: Mutex<HashMap> maybe need to be a smarter datastructure if it
/// affects benchmarking
pub struct CacheMap<'a, K, V, W> {
    alloc: &'a Arena<V>,
    map: Mutex<HashMap<K, W>>,
}

impl<'a, K, V, W> CacheMap<'a, K, V, W>
where
    K: Eq + Hash,
    W: Clone,
{
    #[inline]
    pub fn new(alloc: &'a Arena<V>) -> Self {
        Self {
            alloc,
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Get the value of the given key in the map.
    #[inline]
    pub fn get<Q: Eq + ?Sized>(&self, key: &Q) -> Option<W>
    where
        K: Borrow<Q>,
        Q: Hash + PartialEq,
    {
        let map = self.map.lock().unwrap();
        map.get(key).map(|value| (*value).clone())
    }

    /// Try inserting the value V if missing. The insert function is not called if the value is
    /// present.
    ///
    /// The first value is picked to avoid multiple cached results floating around. This assumes
    /// that the cache is immutable (i.e. there's no invalidation).
    ///
    /// Returns a reference to the inserted value.
    pub fn or_insert_with_transform<F, G>(&self, key: K, insert: F, transform: G) -> W
    where
        F: FnOnce() -> V,
        G: FnOnce(&'a V) -> W,
    {
        let mut ret: Option<W> = None;
        let ret_mut = &mut ret;
        let alloc = self.alloc;
        let mut map = self.map.lock().unwrap();
        let value = map.entry(key).or_insert_with(|| {
            let alloc_value: &'a V = alloc.alloc(insert());
            transform(alloc_value)
        });
        ret_mut.replace(value.clone());
        ret.expect("return value should always be initialized")
    }

    /// A version of insert_with_transform where the transform can fail. If it does then the value
    /// is not inserted into the map and is left allocated as garbage in the arena.
    pub fn or_insert_with_try_transform<F, G, E>(
        &self,
        key: K,
        insert: F,
        try_transform: G,
    ) -> Result<W, E>
    where
        F: FnOnce() -> V,
        G: FnOnce(&'a V) -> Result<W, E>,
    {
        let mut ret: Option<Result<W, E>> = None;
        let ret_mut = &mut ret;
        let mut map = self.map.lock().unwrap();
        let value = match map.entry(key) {
            Entry::Occupied(oe) => Ok(oe.get().clone()),
            Entry::Vacant(ve) => {
                let alloc_value: &'a V = self.alloc.alloc(insert());
                try_transform(alloc_value).and_then(|v| Ok(ve.insert(v).clone()))
            }
        };
        ret_mut.replace(value);
        ret.expect("return value should always be initialized")
    }
}

impl<'a, K, V> CacheRefMap<'a, K, V>
where
    K: Eq + Hash,
{
    /// Insert the value if not present. Discard the value if present.
    ///
    /// The first value is picked to avoid multiple cached results floating around. This assumes
    /// that the cache is immutable (i.e. there's no invalidation).
    ///
    /// Returns the address of the inserted value.
    #[inline]
    pub fn or_insert(&self, key: K, value: V) -> &'a V {
        self.or_insert_with_transform(key, move || value, |value_ref| value_ref)
    }

    #[inline]
    pub fn or_insert_with<F>(&self, key: K, insert: F) -> &'a V
    where
        F: FnOnce() -> V,
    {
        self.or_insert_with_transform(key, insert, |value_ref| value_ref)
    }
}

#[test]
fn cache_map_thread_safe() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<CacheRefMap<'_, String, String>>();
    assert_sync::<CacheRefMap<'_, String, String>>();
}
