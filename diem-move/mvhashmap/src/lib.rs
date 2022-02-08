// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crossbeam::utils::CachePadded;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use std::{
    collections::{btree_map::BTreeMap},
    hash::Hash,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};

#[cfg(test)]
mod unit_tests;

const FLAG_DONE: usize = 0;
const FLAG_ESTIMATE: usize = 1;

pub(crate) struct WriteCell<V> {
    flag: AtomicUsize,
    retry_num: AtomicUsize,
    data: ArcSwap<Option<V>>,
}

impl<V> WriteCell<V> {
    pub fn new_from(flag: usize, retry_num: usize, data: Option<V>) -> WriteCell<V> {
        WriteCell {
            flag: AtomicUsize::new(flag),
            retry_num: AtomicUsize::new(retry_num),
            data: ArcSwap::from(Arc::new(data)),
        }
    }

    pub fn flag(&self) -> usize {
        self.flag.load( Ordering::SeqCst)
    }

    pub fn estimate(&self){
        self.flag.store(FLAG_ESTIMATE, Ordering::SeqCst);
    }
}

pub struct MVHashMap<K, V> {
    data: Arc<DashMap<K, BTreeMap<usize, CachePadded<WriteCell<V>>>>>,
}

impl<K: Hash + Clone + Eq, V: Clone> MVHashMap<K, V> {
    pub fn new() -> MVHashMap<K, V> {
        MVHashMap {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn write(
        &self,
        key: &K,
        txn_idx: usize,
        incarnation: usize,
        data: Option<V>,
    ) {
        let mut map = self.data.entry(key.clone()).or_insert(BTreeMap::new());
        map.insert(txn_idx, CachePadded::new(WriteCell::new_from(FLAG_DONE, incarnation, data)));
    }

    pub fn make_estimate(&self, key: &K, txn_idx: usize) {
        let map = self.data.get(key).unwrap();
        map.get(&txn_idx)
            .unwrap().estimate();
    }

    pub fn delete(&self, key: &K, txn_idx: usize) {
        //TODO: investigate logical deletion.
        let mut map = self.data.get_mut(key).unwrap();
        map.remove(&txn_idx);
    }

    // read may return Ok((Option<V>, txn_idx, incarnation)), Err(Some(txn_idx)) for a dependency
    // or Err(None) when no prior entry is found.
    // TODO: change to Error::NotInMap, etc.
    pub fn read(
        &self,
        key: &K,
        txn_idx: usize,
    ) -> Result<(Option<V>, usize, usize), Option<(usize, usize)>> {
        match self.data.get(key) {
            Some(tree) => {
                // Find the dependency
                let mut iter = tree.range(0..txn_idx);
                if let Some((entry_key, entry_val)) = iter.next_back() {
                    let flag = entry_val.flag();

                    if flag == FLAG_ESTIMATE {
                        // Found a dependency.
                        return Err(Some((
                            *entry_key,
                            entry_val.retry_num.load(Ordering::SeqCst),
                        )));
                    }

                    if flag == FLAG_DONE {
                        // The entry is populated, return its contents.
                        return Ok((
                            (**entry_val.data.load()).clone(),
                            *entry_key,
                            entry_val.retry_num.load(Ordering::SeqCst),
                        ));
                    }
                }
                return Err(None);
            }
            None => {
                return Err(None);
            }
        }
    }
}