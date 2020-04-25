// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, SystemTime},
};

struct ValueInfo<V> {
    value: V,
    ttl: SystemTime,
}

pub struct TtlCache<K, V> {
    capacity: usize,
    default_timeout: Duration,
    data: HashMap<K, ValueInfo<V>>,
    ttl_index: BTreeMap<SystemTime, K>,
}

impl<K, V> TtlCache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + std::clone::Clone,
{
    pub fn new(capacity: usize, default_timeout: Duration) -> Self {
        Self {
            capacity,
            default_timeout,
            data: HashMap::new(),
            ttl_index: BTreeMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key).map(|v| &v.value)
    }

    pub fn insert(&mut self, key: K, value: V) {
        // remove old entry if it exists
        match self.data.get(&key) {
            Some(info) => {
                self.ttl_index.remove(&info.ttl);
            }
            None => {
                // remove oldest entry if cache is still full
                if self.data.len() == self.capacity {
                    let first_entry = self.ttl_index.keys().next().cloned();
                    if let Some(tst) = first_entry {
                        if let Some(key) = self.ttl_index.remove(&tst) {
                            self.data.remove(&key);
                        }
                    }
                }
            }
        }

        // insert new one
        if let Some(expiration_time) = SystemTime::now().checked_add(self.default_timeout) {
            self.ttl_index.insert(expiration_time.clone(), key.clone());
            let value_info = ValueInfo {
                value,
                ttl: expiration_time,
            };
            self.data.insert(key, value_info);
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        match self.data.remove(&key) {
            Some(info) => {
                self.ttl_index.remove(&info.ttl);
                Some(info.value)
            }
            None => None,
        }
    }

    pub fn gc(&mut self, gc_time: SystemTime) {
        // remove expired entries
        let mut active = self.ttl_index.split_off(&gc_time);
        for key in self.ttl_index.values() {
            self.data.remove(key);
        }
        self.ttl_index.clear();
        self.ttl_index.append(&mut active);
    }

    #[cfg(test)]
    pub fn size(&self) -> usize {
        self.data.len()
    }
}
