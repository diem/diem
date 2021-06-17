// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::sample::Index as PropIndex;
use std::{collections::BTreeSet, ops::Index as OpsIndex};

/// Given a maximum value `max` and a list of [`Index`](proptest::sample::Index) instances, picks
/// integers in the range `[0, max)` uniformly randomly and without duplication.
///
/// If `indexes_len` is greater than `max`, all indexes will be returned.
///
/// This function implements [Robert Floyd's F2
/// algorithm](https://blog.acolyer.org/2018/01/30/a-sample-of-brilliance/) for sampling without
/// replacement.
pub(crate) fn pick_idxs<T, P>(max: usize, indexes: &T, indexes_len: usize) -> Vec<usize>
where
    T: OpsIndex<usize, Output = P> + ?Sized,
    P: AsRef<PropIndex>,
{
    // See https://blog.acolyer.org/2018/01/30/a-sample-of-brilliance/ (the F2 algorithm)
    // for a longer explanation. This is a variant that works with zero-indexing.
    let mut selected = BTreeSet::new();
    let to_select = indexes_len.min(max);
    for (iter_idx, choice) in ((max - to_select)..max).enumerate() {
        // "RandInt(1, J)" in the original algorithm means a number between 1
        // and choice, both inclusive. `PropIndex::index` picks a number between 0 and
        // whatever's passed in, with the latter exclusive. Pass in "+1" to ensure the same
        // range of values is picked from. (This also ensures that if choice is 0 then `index`
        // doesn't panic.
        let idx = indexes[iter_idx].as_ref().index(choice + 1);
        if !selected.insert(idx) {
            selected.insert(choice);
        }
    }
    selected.into_iter().collect()
}

/// Given a maximum value `max` and a slice of [`Index`](proptest::sample::Index) instances, picks
/// integers in the range `[0, max)` uniformly randomly and without duplication.
///
/// If the number of `Index` instances is greater than `max`, all indexes will be returned.
///
/// This function implements [Robert Floyd's F2
/// algorithm](https://blog.acolyer.org/2018/01/30/a-sample-of-brilliance/) for sampling without
/// replacement.
#[inline]
pub(crate) fn pick_slice_idxs(max: usize, indexes: &[impl AsRef<PropIndex>]) -> Vec<usize> {
    pick_idxs(max, indexes, indexes.len())
}
