// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::GrowingSubset;
use proptest::{collection::vec, prelude::*, sample::Index as PropIndex};

proptest! {
    #[test]
    fn max_len(inputs in vec((0..1000usize, 0..100_000usize), 1..5000)) {
        let len = inputs.len();
        let max_idx = 1 + inputs.iter().map(|x| x.0).max().expect("inputs has length at least 1");

        let mut growing_subset: GrowingSubset<_, _> = inputs.into_iter().collect();
        assert_eq!(len, growing_subset.total_len());
        growing_subset.advance_to(&max_idx);
        prop_assert_eq!(len, growing_subset.len());
    }

    #[test]
    fn pick_valid(
        inputs in vec((0..1000usize, 0..100_000usize), 1..5000),
        // queries goes up to 1100 so that advance_to past the end is tested.
        mut queries in vec((0..1100usize, any::<PropIndex>()), 0..500),
    ) {
        // Sort the queries so that indexes are advanced in order.
        queries.sort_by_key(|x| x.0);

        let mut growing_subset: GrowingSubset<_, _> = inputs.into_iter().collect();
        for (to_idx, index) in queries {
            growing_subset.advance_to(&to_idx);
            if growing_subset.is_empty() {
                continue;
            }
            let (picked_idx, _) = growing_subset.pick_item(&index);
            prop_assert!(*picked_idx < to_idx);
        }
    }
}
