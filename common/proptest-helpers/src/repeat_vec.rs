// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::pick_slice_idxs;
use proptest::sample::Index;
use std::iter;

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
/// use diem_proptest_helpers::RepeatVec;
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
/// use diem_proptest_helpers::RepeatVec;
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
    // An invariant is that this doesn't have any zero-length elements.
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

    /// Creates a new, empty `RepeatVec` with the specified capacity to store physical elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
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
    /// use diem_proptest_helpers::RepeatVec;
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
        // Skip over zero-length elements to maintain the invariant on items.
        if size > 0 {
            self.items.push((self.len, item));
            self.len += size;
        }
    }

    /// Removes the item specified by the given *logical* index, shifting all elements after it to
    /// the left by updating start positions.
    ///
    /// Out of bounds indexes have no effect.
    pub fn remove(&mut self, index: usize) {
        self.remove_all(iter::once(index))
    }

    /// Removes the items specified by the given *logical* indexes, shifting all elements after them
    /// to the left by updating start positions.
    ///
    /// Ignores any out of bounds indexes.
    pub fn remove_all(&mut self, logical_indexes: impl IntoIterator<Item = usize>) {
        let mut logical_indexes: Vec<_> = logical_indexes.into_iter().collect();
        logical_indexes.sort_unstable();
        logical_indexes.dedup();
        self.remove_all_impl(logical_indexes)
    }

    fn remove_all_impl(&mut self, logical_indexes: Vec<usize>) {
        // # Notes
        //
        // * This looks pretty long and complicated, mostly because the logic is complex enough to
        //   require manual loops and iteration.
        // * This is unavoidably linear time in the number of physical elements. One way to make the
        //   constant factors smaller would be to implement a ChunkedRepeatVec<T>, which can be done
        //   by wrapping a RepeatVec<RepeatVec<T>> plus some glue logic. The idea would be similar
        //   to skip-lists. 2 levels should be enough, but 3 or more would work as well.

        // Separate function to minimize the amount of monomorphized code.
        let first = match logical_indexes.first() {
            Some(first) => *first,
            None => {
                // No indexes to remove, nothing to do.
                return;
            }
        };
        if first >= self.len() {
            // First index is out of bounds, nothing to do.
            return;
        }

        let first_idx = match self.binary_search(first) {
            Ok(exact_idx) => {
                // Logical copy 0 of the element at this position.
                exact_idx
            }
            Err(start_idx) => {
                // This is the physical index after the logical item. Start reasoning from the
                // previous index to match the Ok branch above.
                start_idx - 1
            }
        };

        // This serves two purposes -- it represents the number of elements to decrease by and
        // the current position in indexes.
        let mut decrease = 0;

        let new_items = {
            let mut items = self.items.drain(first_idx..).peekable();
            let mut new_items = vec![];

            while let Some((current_logical_idx_old, current_elem)) = items.next() {
                let current_logical_idx_new = current_logical_idx_old - decrease;

                let next_logical_idx_old = items.peek().map_or(self.len, |&(idx, _)| idx);

                // Remove all indexes until the next logical index or sorted_indexes runs out.
                while let Some(remove_idx) = logical_indexes.get(decrease) {
                    if *remove_idx < next_logical_idx_old {
                        decrease += 1;
                    } else {
                        break;
                    }
                }

                let next_logical_idx_new = next_logical_idx_old - decrease;
                assert!(
                    next_logical_idx_new >= current_logical_idx_new,
                    "too many items removed from next"
                );

                // Drop zero-length items to maintain invariant.
                if next_logical_idx_new > current_logical_idx_new {
                    new_items.push((current_logical_idx_new, current_elem));
                }
            }
            new_items
        };

        self.items.extend(new_items);
        self.len -= decrease;
    }

    /// Returns the item at location `at`. The return value is a reference to the stored item, plus
    /// the offset from the start (logically, which copy of the item is being returned).
    pub fn get(&self, at: usize) -> Option<(&T, usize)> {
        if at >= self.len {
            return None;
        }
        match self.binary_search(at) {
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

    /// Picks out indexes uniformly randomly from this `RepeatVec`, using the provided
    /// [`Index`](proptest::sample::Index) instances as sources of randomness.
    pub fn pick_uniform_indexes(&self, indexes: &[impl AsRef<Index>]) -> Vec<usize> {
        pick_slice_idxs(self.len(), indexes)
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

    #[inline]
    fn binary_search(&self, at: usize) -> Result<usize, usize> {
        self.items.binary_search_by_key(&at, |(start, _)| *start)
    }

    /// Check and assert the internal invariants for this RepeatVec.
    #[cfg(test)]
    pub(crate) fn assert_invariants(&self) {
        for window in self.items.windows(2) {
            let (idx1, idx2) = match window {
                [(idx1, _), (idx2, _)] => (*idx1, *idx2),
                _ => panic!("wrong window size"),
            };
            assert!(idx1 < idx2, "no zero-length elements");
        }
        match self.items.last() {
            Some(&(idx, _)) => {
                assert!(
                    idx < self.len,
                    "length must be greater than last element's start"
                );
            }
            None => {
                assert_eq!(self.len, 0, "empty RepeatVec");
            }
        }
    }
}

// Note that RepeatVec cannot implement `std::ops::Index<usize>` because the return type of Index
// has to be a reference (in this case it would be &(T, usize)). But RepeatVec computes the result
// of get() (as (&T, usize)) instead of merely returning a reference. This is a subtle but
// important point.
