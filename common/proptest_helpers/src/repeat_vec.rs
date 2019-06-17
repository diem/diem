// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::pick_slice_idxs;
use proptest::sample::Index;

/// An efficient representation of a vector with repeated elements inserted.
///
/// Internally, this data structure stores one copy of each inserted element, along with data about
/// how many times each element is repeated.
///
/// This data structure does not do any sort of deduplication, so it isn't any sort of set (or
/// multiset).
///
/// This is useful for presenting a large logical vector for picking `proptest` indexes from.
///
/// # Examples
///
/// ```
/// use proptest_helpers::RepeatVec;
///
/// let mut repeat_vec = RepeatVec::new();
/// repeat_vec.extend("a", 10); // logically, insert "a" 10 times
/// repeat_vec.extend("b", 20); // logically, insert "b" 20 times
/// assert_eq!(repeat_vec.get(0), Some((&"a", 0))); // returns the "a" at logical position 0
/// assert_eq!(repeat_vec.get(5), Some((&"a", 5))); // returns the "a" at logical position 5
/// assert_eq!(repeat_vec.get(10), Some((&"b", 0))); // returns the "b" (offset 0) at logical position 10
/// assert_eq!(repeat_vec.get(20), Some((&"b", 10))); // returns the "b" (offset 10) at logical position 20
/// assert_eq!(repeat_vec.get(30), None); // past the end of the logical array
/// ```
///
/// The data structure doesn't care about whether the inserted items are equal or not.
///
/// ```
/// use proptest_helpers::RepeatVec;
///
/// let mut repeat_vec = RepeatVec::new();
/// repeat_vec.extend("a", 10); // logically, insert "a" 10 times
/// repeat_vec.extend("a", 20); // logically, insert "a" 20 times
/// assert_eq!(repeat_vec.get(0), Some((&"a", 0)));
/// assert_eq!(repeat_vec.get(5), Some((&"a", 5)));
/// assert_eq!(repeat_vec.get(10), Some((&"a", 0))); // This refers to the second "a".
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct RepeatVec<T> {
    // The first element of each tuple is the starting position for this item.
    items: Vec<(usize, T)>,
    len: usize,
}

impl<T> RepeatVec<T> {
    /// Creates a new, empty `RepeatVec`.
    pub fn new() -> Self {
        Self {
            items: vec![],
            len: 0,
        }
    }

    /// Returns the *logical* number of elements in this `RepeatVec`.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if this `RepeatVec` has no *logical* elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use proptest_helpers::RepeatVec;
    ///
    /// let mut repeat_vec = RepeatVec::new();
    ///
    /// // There are no elements in this RepeatVec.
    /// assert!(repeat_vec.is_empty());
    ///
    /// // Adding 0 logical copies of an element still means it's empty.
    /// repeat_vec.extend("a", 0);
    /// assert!(repeat_vec.is_empty());
    ///
    /// // Adding non-zero logical copies makes this vector not empty.
    /// repeat_vec.extend("b", 1);
    /// assert!(!repeat_vec.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Extends this `RepeatVec` by logically adding `size` copies of `item` to the end of it.
    pub fn extend(&mut self, item: T, size: usize) {
        self.items.push((self.len, item));
        self.len += size;
    }

    /// Returns the item at location `at`. The return value is a reference to the stored item, plus
    /// the offset from the start (logically, which copy of the item is being returned).
    pub fn get(&self, at: usize) -> Option<(&T, usize)> {
        if at >= self.len {
            return None;
        }
        match self.items.binary_search_by_key(&at, |(start, _)| *start) {
            Ok(exact_idx) => Some((&self.items[exact_idx].1, 0)),
            Err(start_idx) => {
                // start_idx can never be 0 because usize starts from 0 and items[0].0 is always 0.
                // So start_idx is always at least 1.
                let start_val = &self.items[start_idx - 1];
                let offset = at - start_val.0;
                Some((&start_val.1, offset))
            }
        }
    }

    /// Picks out elements uniformly randomly from this `RepeatVec`, using the provided
    /// [`Index`](proptest::sample::Index) instances as sources of randomness.
    pub fn pick_uniform(&self, indexes: &[impl AsRef<Index>]) -> Vec<(&T, usize)> {
        pick_slice_idxs(self.len(), indexes)
            .into_iter()
            .map(|idx| {
                self.get(idx)
                    .expect("indexes picked should always be in range")
            })
            .collect()
    }
}

// Note that RepeatVec cannot implement `std::ops::Index<usize>` because the return type of Index
// has to be a reference (in this case it would be &(T, usize)). But RepeatVec computes the result
// of get() (as (&T, usize)) instead of merely returning a reference. This is a subtle but
// important point.
