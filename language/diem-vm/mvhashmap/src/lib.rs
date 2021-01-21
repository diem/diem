// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use once_cell::sync::OnceCell;
use std::{
    cmp::{max, PartialOrd},
    collections::{btree_map::BTreeMap, HashMap},
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
};

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

const FLAG_UNASSIGNED: usize = 0;
const FLAG_DONE: usize = 2;
const FLAG_SKIP: usize = 3;

pub struct MVHashMap<K, V> {
    data: HashMap<K, BTreeMap<Version, WriteVersionValue<V>>>,
}

#[cfg_attr(any(target_arch = "x86_64"), repr(align(128)))]
pub(crate) struct WriteVersionValue<V> {
    flag: AtomicUsize,
    data: OnceCell<Option<V>>,
}

impl<V> WriteVersionValue<V> {
    pub fn new() -> WriteVersionValue<V> {
        WriteVersionValue {
            flag: AtomicUsize::new(FLAG_UNASSIGNED),
            data: OnceCell::new(),
        }
    }
}

impl<K: Hash + Clone + Eq, V: Clone> MVHashMap<K, V> {
    pub fn new() -> MVHashMap<K, V> {
        MVHashMap {
            data: HashMap::new(),
        }
    }

    pub fn new_from(possible_writes: Vec<(K, Version)>) -> (usize, MVHashMap<K, V>) {
        let mut map: HashMap<K, BTreeMap<Version, WriteVersionValue<V>>> = HashMap::new();
        for (key, version) in possible_writes.into_iter() {
            map.entry(key)
                .or_default()
                .insert(version, WriteVersionValue::new());
        }
        (
            map.values()
                .fold(0, |max_depth, map| max(max_depth, map.len())),
            MVHashMap { data: map },
        )
    }

    pub fn get_change_set(&self) -> Vec<(K, Option<V>)> {
        let mut change_set = Vec::with_capacity(self.data.len());
        for (k, _) in self.data.iter() {
            let val = self.read(k, usize::MAX).unwrap();
            change_set.push((k.clone(), val.clone()));
        }
        change_set
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn write(&self, key: &K, version: Version, data: Option<V>) -> Result<(), ()> {
        // By construction there will only be a single writer, before the
        // write there will be no readers on the variable.
        // So it is safe to go ahead and write without any further check.
        // Then update the flag to enable reads.

        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        #[cfg(test)]
        {
            // Test the invariant holds
            let flag = entry.flag.load(Ordering::Acquire);
            if flag != FLAG_UNASSIGNED {
                panic!("Cannot write twice to same entry.");
            }
        }

        entry.data.set(data).map_err(|_| ())?;

        entry.flag.store(FLAG_DONE, Ordering::Release);
        Ok(())
    }

    pub fn skip_if_not_set(&self, key: &K, version: Version) -> Result<(), ()> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        // Test the invariant holds
        let flag = entry.flag.load(Ordering::Acquire);
        if flag == FLAG_UNASSIGNED {
            entry.flag.store(FLAG_SKIP, Ordering::Release);
        }

        Ok(())
    }

    pub fn skip(&self, key: &K, version: Version) -> Result<(), ()> {
        // We only write or skip once per entry
        // So it is safe to go ahead and just do it.
        let entry = self
            .data
            .get(key)
            .ok_or_else(|| ())?
            .get(&version)
            .ok_or_else(|| ())?;

        #[cfg(test)]
        {
            // Test the invariant holds
            let flag = entry.flag.load(Ordering::Acquire);
            if flag != FLAG_UNASSIGNED {
                panic!("Cannot write twice to same entry.");
            }
        }

        entry.flag.store(FLAG_SKIP, Ordering::Release);
        Ok(())
    }

    pub fn read(&self, key: &K, version: Version) -> Result<&Option<V>, Option<Version>> {
        // Get the smaller key
        let tree = self.data.get(key).ok_or_else(|| None)?;

        let mut iter = tree.range(0..version);

        while let Some((entry_key, entry_val)) = iter.next_back() {
            if *entry_key < version {
                let flag = entry_val.flag.load(Ordering::Acquire);

                // Return this key, must wait.
                if flag == FLAG_UNASSIGNED {
                    return Err(Some(*entry_key));
                }

                // If we are to skip this entry, pick the next one
                if flag == FLAG_SKIP {
                    continue;
                }

                // The entry is populated so return its contents
                if flag == FLAG_DONE {
                    return Ok(entry_val.data.get().unwrap());
                }

                unreachable!();
            }
        }

        Err(None)
    }
}

impl<K, V> MVHashMap<K, V>
where
    K: PartialOrd + Send + Clone + Hash + Eq,
    V: Send,
{
    fn split_merge(
        num_cpus: usize,
        num: usize,
        split: Vec<(K, Version)>,
    ) -> (usize, HashMap<K, BTreeMap<Version, WriteVersionValue<V>>>) {
        if ((2 << num) > num_cpus) || split.len() < 1000 {
            let mut data = HashMap::new();
            let mut max_len = 0;
            for (path, version) in split.into_iter() {
                let place = data.entry(path).or_insert(BTreeMap::new());
                place.insert(version, WriteVersionValue::new());
                max_len = max(max_len, place.len());
            }
            (max_len, data)
        } else {
            let pivot_address = split[split.len() / 2].0.clone();
            let (left, right): (Vec<_>, Vec<_>) =
                split.into_iter().partition(|(p, _)| *p < pivot_address);
            let ((m0, mut left_map), (m1, right_map)) = rayon::join(
                || Self::split_merge(num_cpus, num + 1, left),
                || Self::split_merge(num_cpus, num + 1, right),
            );
            left_map.extend(right_map);
            (max(m0, m1), left_map)
        }
    }

    pub fn new_from_parallel(possible_writes: Vec<(K, Version)>) -> (usize, MVHashMap<K, V>) {
        let num_cpus = num_cpus::get();

        let (max_dependency_depth, data) = Self::split_merge(num_cpus, 0, possible_writes);
        (max_dependency_depth, MVHashMap { data })
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn create_write_read_placeholder_struct() {
        let ap1 = b"/foo/b".to_vec();
        let ap2 = b"/foo/c".to_vec();

        let data = vec![(ap1.clone(), 10), (ap2.clone(), 10), (ap2.clone(), 20)];

        let (max_dep, mvtbl) = MVHashMap::new_from(data);

        assert_eq!(2, max_dep);

        assert_eq!(2, mvtbl.len());

        // Reads that should go the the DB return Err(None)
        let r1 = mvtbl.read(&ap1, 5);
        assert_eq!(Err(None), r1);

        // Reads at a version return the previous versions, not this
        // version.
        let r1 = mvtbl.read(&ap1, 10);
        assert_eq!(Err(None), r1);

        // Check reads into non-ready structs return the Err(entry)

        // Reads at a higher version return the previous version
        let r1 = mvtbl.read(&ap1, 15);
        assert_eq!(Err(Some(10)), r1);

        // Writes populate the entry
        mvtbl.write(&ap1, 10, Some(vec![0, 0, 0])).unwrap();

        // Subsequent higher reads read this entry
        let r1 = mvtbl.read(&ap1, 15);
        assert_eq!(Ok(&Some(vec![0, 0, 0])), r1);

        // Set skip works
        assert!(mvtbl.skip(&ap1, 20).is_err());

        // Higher reads skip this entry
        let r1 = mvtbl.read(&ap1, 25);
        assert_eq!(Ok(&Some(vec![0, 0, 0])), r1);
    }
}
