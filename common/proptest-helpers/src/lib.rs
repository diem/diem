// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[cfg(test)]
mod unit_tests;

mod growing_subset;
mod repeat_vec;
mod value_generator;

pub use crate::{
    growing_subset::GrowingSubset, repeat_vec::RepeatVec, value_generator::ValueGenerator,
};

use crossbeam::thread;
use proptest::sample::Index as PropIndex;
use proptest_derive::Arbitrary;
use std::{
    any::Any,
    collections::BTreeSet,
    ops::{Deref, Index as OpsIndex},
};

/// Creates a new thread with a larger stack size.
///
/// Generating some proptest values can overflow the stack. This allows test authors to work around
/// this limitation.
///
/// This is expected to be used with closure-style proptest invocations:
///
/// ```
/// use proptest::prelude::*;
/// use diem_proptest_helpers::with_stack_size;
///
/// with_stack_size(4 * 1024 * 1024, || proptest!(|(x in 0usize..128)| {
///     // assertions go here
///     prop_assert!(x >= 0 && x < 128);
/// }));
/// ```
pub fn with_stack_size<'a, F, T>(size: usize, f: F) -> Result<T, Box<dyn Any + 'static + Send>>
where
    F: FnOnce() -> T + Send + 'a,
    T: Send + 'a,
{
    thread::scope(|s| {
        let handle = s.builder().stack_size(size).spawn(|_| f()).map_err(|err| {
            let any: Box<dyn Any + 'static + Send> = Box::new(err);
            any
        })?;
        handle.join()
    })?
}

/// Given a maximum value `max` and a list of [`Index`](proptest::sample::Index) instances, picks
/// integers in the range `[0, max)` uniformly randomly and without duplication.
///
/// If `indexes_len` is greater than `max`, all indexes will be returned.
///
/// This function implements [Robert Floyd's F2
/// algorithm](https://blog.acolyer.org/2018/01/30/a-sample-of-brilliance/) for sampling without
/// replacement.
pub fn pick_idxs<T, P>(max: usize, indexes: &T, indexes_len: usize) -> Vec<usize>
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
pub fn pick_slice_idxs(max: usize, indexes: &[impl AsRef<PropIndex>]) -> Vec<usize> {
    pick_idxs(max, indexes, indexes.len())
}

/// Wrapper for `proptest`'s [`Index`][proptest::sample::Index] that allows `AsRef` to work.
///
/// There is no blanket `impl<T> AsRef<T> for T`, so `&[PropIndex]` doesn't work with
/// `&[impl AsRef<PropIndex>]` (unless an impl gets added upstream). `Index` does.
#[derive(Arbitrary, Clone, Copy, Debug)]
pub struct Index(PropIndex);

impl AsRef<PropIndex> for Index {
    fn as_ref(&self) -> &PropIndex {
        &self.0
    }
}

impl Deref for Index {
    type Target = PropIndex;

    fn deref(&self) -> &PropIndex {
        &self.0
    }
}
