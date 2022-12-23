// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::OnceCell;
use std::{
    cmp::{max, PartialOrd},
    collections::{btree_map::BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
};

#[cfg(test)]
mod unit_tests;

/// A structure that holds placeholders for each write to the database
//
//  The structure is created by one thread creating the scheduling, and
//  at that point it is used as a &mut by that single thread.
//
//  Then it is passed to all threads executing as a shared reference. At
//  this point only a single thread must write to any entry, and others
//  can read from it. Only entries are mutated using interior mutability,
//  but no entries can be added or deleted.
//

pub type Version = usize;

pub struct MVHashMap<K, V> {
    data: HashMap<K, BTreeMap<Version, WriteCell<V>>>,
}

#[derive(Debug)]
pub enum Error {
    // A write has been performed on an entry that is not in the possible_writes list.
    UnexpectedWrite,
}

#[cfg_attr(any(target_arch = "x86_64"), repr(align(128)))]
pub(crate) struct WriteCell<V>(OnceCell<Option<V>>);

impl<V: Debug> WriteCell<V> {
    pub fn new() -> WriteCell<V> {
        WriteCell(OnceCell::new())
    }

    pub fn is_assigned(&self) -> bool {
        self.0.get().is_some()
    }

    pub fn write(&self, v: V) {
        self.0
            .set(Some(v))
            .expect("WriteCell should be modified exactly once");
    }

    pub fn skip(&self) {
        self.0
            .set(None)
            .expect("WriteCell should be modified exactly once");
    }

    pub fn get(&self) -> Option<&Option<V>> {
        self.0.get()
    }
}

impl<K: Hash + Clone + Eq, V: Debug> MVHashMap<K, V> {
    /// Create the MVHashMap structure from a list of possible writes. Each element in the list
    /// indicates a key that could potentially be modified at its given version.
    ///
    /// Returns the MVHashMap, and the maximum number of writes that can write to one single key.
    pub fn new_from(possible_writes: Vec<(K, Version)>) -> (Self, usize) {
        let mut outer_map: HashMap<K, BTreeMap<Version, WriteCell<V>>> = HashMap::new();
        for (key, version) in possible_writes.into_iter() {
            outer_map
                .entry(key)
                .or_default()
                .insert(version, WriteCell::new());
        }
        let max_dependency_size = outer_map
            .values()
            .fold(0, |max_depth, btree_map| max(max_depth, btree_map.len()));

        (MVHashMap { data: outer_map }, max_dependency_size)
    }

    /// Get the number of keys in the MVHashMap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    fn get_entry(&self, key: &K, version: Version) -> Result<&WriteCell<V>, Error> {
        self.data
            .get(key)
            .ok_or(Error::UnexpectedWrite)?
            .get(&version)
            .ok_or(Error::UnexpectedWrite)
    }

    /// Write to `key` at `version`.
    /// Function will return an error if the write is not in the initial `possible_writes` list.
    pub fn write(&self, key: &K, version: Version, data: V) -> Result<(), Error> {
        // By construction there will only be a single writer, before the
        // write there will be no readers on the variable.
        // So it is safe to go ahead and write without any further check.

        let entry = self.get_entry(key, version)?;

        #[cfg(test)]
        {
            // Test the invariant holds
            if entry.is_assigned() {
                panic!("Cannot write twice to same entry.");
            }
        }

        entry.write(data);

        Ok(())
    }

    /// Skips writing to `key` at `version` if that entry hasn't been assigned.
    /// Function will return an error if the write is not in the initial `possible_writes` list.
    pub fn skip_if_not_set(&self, key: &K, version: Version) -> Result<(), Error> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self.get_entry(key, version)?;

        // Test the invariant holds
        if !entry.is_assigned() {
            entry.skip();
        }

        Ok(())
    }

    /// Skips writing to `key` at `version`.
    /// Function will return an error if the write is not in the initial `possible_writes` list.
    /// `skip` should only be invoked when `key` at `version` hasn't been assigned.
    pub fn skip(&self, key: &K, version: Version) -> Result<(), Error> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self.get_entry(key, version)?;

        #[cfg(test)]
        {
            // Test the invariant holds
            if entry.is_assigned() {
                panic!("Cannot write twice to same entry.");
            }
        }

        entry.skip();
        Ok(())
    }

    /// Get the value of `key` at `version`.
    /// Returns Ok(val) if such key is already assigned by previous transactions.
    /// Returns Err(None) if `version` is smaller than the write of all previous versions.
    /// Returns Err(Some(version)) if such key is dependent on the `version`-th transaction.
    pub fn read(&self, key: &K, version: Version) -> Result<&V, Option<Version>> {
        let tree = self.data.get(key).ok_or(None)?;

        let mut iter = tree.range(0..version);

        while let Some((entry_key, entry_val)) = iter.next_back() {
            if *entry_key < version {
                match entry_val.get() {
                    // Entry not yet computed, return the version that blocked this query.
                    None => return Err(Some(*entry_key)),
                    // Entry is skipped, go to previous version.
                    Some(None) => continue,
                    Some(Some(v)) => return Ok(v),
                }
            }
        }

        Err(None)
    }
}

const PARALLEL_THRESHOLD: usize = 1000;

impl<K, V> MVHashMap<K, V>
where
    K: PartialOrd + Send + Clone + Hash + Eq,
    V: Send + Debug,
{
    fn split_merge(
        num_cpus: usize,
        recursion_depth: usize,
        split: Vec<(K, Version)>,
    ) -> (usize, HashMap<K, BTreeMap<Version, WriteCell<V>>>) {
        if (1 << recursion_depth) > num_cpus || split.len() < PARALLEL_THRESHOLD {
            let mut data = HashMap::new();
            let mut max_len = 0;
            for (path, version) in split.into_iter() {
                let place = data.entry(path).or_insert_with(BTreeMap::new);
                place.insert(version, WriteCell::new());
                max_len = max(max_len, place.len());
            }
            (max_len, data)
        } else {
            // Partition the possible writes by keys and work on each partition in parallel.
            let pivot_address = split[split.len() / 2].0.clone();
            let (left, right): (Vec<_>, Vec<_>) =
                split.into_iter().partition(|(p, _)| *p < pivot_address);
            let ((m0, mut left_map), (m1, right_map)) = rayon::join(
                || Self::split_merge(num_cpus, recursion_depth + 1, left),
                || Self::split_merge(num_cpus, recursion_depth + 1, right),
            );
            left_map.extend(right_map);
            (max(m0, m1), left_map)
        }
    }

    /// Create the MVHashMap structure from a list of possible writes in parallel.
    pub fn new_from_parallel(possible_writes: Vec<(K, Version)>) -> (Self, usize) {
        let num_cpus = num_cpus::get();

        let (max_dependency_len, data) = Self::split_merge(num_cpus, 0, possible_writes);
        (MVHashMap { data }, max_dependency_len)
    }
}
