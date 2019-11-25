// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::Arena;
use chashmap::CHashMap;
use std::{borrow::Borrow, hash::Hash};

/// The most common case of `CacheMap`, where references to stored values are handed out.
pub type CacheRefMap<'a, K, V> = CacheMap<'a, K, V, &'a V>;

/// A map custom designed for the VM runtime caches. Allocations are done in an Arena instead of
/// directly in a hash table, which allows for new entries to be added while existing entries are
/// borrowed out.
///
/// TODO: Entry-like API? Current one is somewhat awkward to use.
/// TODO: eviction -- how to do it safely?
/// TODO: should the map own the arena?
pub struct CacheMap<'a, K, V, W> {
    alloc: &'a Arena<V>,
    map: CHashMap<K, W>,
}

impl<'a, K, V, W> CacheMap<'a, K, V, W>
where
    K: Hash + PartialEq,
    W: Clone,
{
    #[inline]
    pub fn new(alloc: &'a Arena<V>) -> Self {
        Self {
            alloc,
            map: CHashMap::new(),
        }
    }

    /// Get the value of the given key in the map.
    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<W>
    where
        K: Borrow<Q>,
        Q: Hash + PartialEq,
    {
        self.map.get(key).map(|value| (*value).clone())
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
        self.map.alter(key, move |value| match value {
            Some(value) => {
                ret_mut.replace(value.clone());
                Some(value)
            }
            None => {
                let alloc_value: &'a V = self.alloc.alloc(insert());
                let value = transform(alloc_value);
                ret_mut.replace(value.clone());
                Some(value)
            }
        });
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
        self.map.alter(key, move |value| match value {
            Some(value) => {
                ret_mut.replace(Ok(value.clone()));
                Some(value)
            }
            None => {
                let alloc_value: &'a V = self.alloc.alloc(insert());
                let res = try_transform(alloc_value);
                let (cloned_res, stored_value) = match res {
                    Ok(value) => (Ok(value.clone()), Some(value)),
                    Err(err) => (Err(err), None),
                };
                ret_mut.replace(cloned_res);
                stored_value
            }
        });
        ret.expect("return value should always be initialized")
    }
}

impl<'a, K, V> CacheRefMap<'a, K, V>
where
    K: Hash + PartialEq,
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
