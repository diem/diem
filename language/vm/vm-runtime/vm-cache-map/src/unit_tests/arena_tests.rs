// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::Arena;
use crossbeam::scope;
use proptest::{collection::vec, prelude::*};

proptest! {
    #[test]
    fn one_thread(strings in vec(".*", 0..50)) {
        let arena: Arena<String> = Arena::new();
        for string in strings {
            prop_assert_eq!(arena.alloc(string.clone()), &string);
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn many_threads(string_vecs in vec(vec(".*", 0..50), 0..16)) {
        let arena: Arena<String> = Arena::new();
        let arena_ref = &arena;
        let res = scope(|s| {
            for strings in &string_vecs {
                s.spawn(move |_| {
                    for string in strings {
                        prop_assert_eq!(arena_ref.alloc(string.clone()), string);
                    }
                    Ok(())
                });
            }
        });
        res.expect("threads should succeed");
    }
}
