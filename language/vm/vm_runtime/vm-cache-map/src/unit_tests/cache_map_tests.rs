// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Arena, CacheRefMap};
use crossbeam::scope;
use proptest::{
    collection::{hash_map, vec},
    prelude::*,
};
use rand::{seq::SliceRandom, thread_rng};

const NUM_THREADS: usize = 8;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn or_insert(kv_pairs in hash_map(".*", ".*", 0..100)) {
        let arena = Arena::new();
        let map = CacheRefMap::new(&arena);
        for (key, value) in kv_pairs {
            prop_assert_eq!(map.get(&key), None);
            prop_assert_eq!(map.or_insert(key.clone(), value.clone()), &value);
            prop_assert_eq!(map.get(&key), Some(&value));
        }
    }

    #[test]
    fn or_insert_duplicates(kv_lists in hash_map(".*", vec(".*", 1..4), 0..100)) {
        let arena = Arena::new();
        let map = CacheRefMap::new(&arena);
        for (key, values) in kv_lists {
            let first = values[0].clone();
            prop_assert_eq!(map.get(&key), None);
            prop_assert_eq!(map.or_insert(key.clone(), first.clone()), &first);
            prop_assert_eq!(map.get(&key), Some(&first));

            // Further values for the same key should be ignored.
            for value in values.into_iter().skip(1) {
                prop_assert_eq!(map.or_insert_with(key.clone(), || value.clone()), &first);
                prop_assert_eq!(map.or_insert(key.clone(), value), &first);
            }
        }
    }

    #[test]
    fn or_insert_many_threads(kv_lists in hash_map(".*", vec(".*", NUM_THREADS), 0..50)) {
        // Try inserting to the list concurrently with NUM_THREADS threads.
        let arena: Arena<String> = Arena::new();
        let map: CacheRefMap<String, String> = CacheRefMap::new(&arena);
        let map_ref = &map;

        let mut kv_thread_lists: Vec<Vec<(String, String)>> = vec![vec![]; NUM_THREADS];
        for (key, values) in &kv_lists {
            for (idx, value) in values.iter().enumerate() {
                kv_thread_lists[idx].push((key.clone(), value.to_string()));
            }
        }

        // Shuffle the lists so each thread gets a chance to insert the first value.
        let mut rng = thread_rng();
        for kv_pairs in &mut kv_thread_lists {
            kv_pairs.shuffle(&mut rng);
        }

        let res = scope(move |s| {
            for kv_pairs in kv_thread_lists {
                s.spawn(move |_| {
                    for (key, value) in kv_pairs {
                        // This is nondeterministic so can't really be compared.
                        map_ref.or_insert(key, value);
                    }
                    Ok::<_, ()>(())
                });
            }
        });
        res.expect("threads should succeed");

        // The final value for each key should be one of the values in kv_lists.
        for (key, values) in &kv_lists {
            let cached_value = map.get(key).expect("at least one value should have been cached");
            prop_assert!(values.contains(cached_value));
        }
    }
}
