// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proptest::sample::Index;
use std::iter::FromIterator;

/// A set of elements, each with an associated key, that grows over time.
///
/// This is called `GrowingSubset` because the universal set the subset grows from is provided
/// upfront. At any time, the items included form the *current subset*.
///
/// `GrowingSubset` integrates with `proptest` through the [`pick_item`][GrowingSubset::pick_item]
/// and [`pick_value`][GrowingSubset::pick_value] methods.
///
/// # Examples
///
/// ```
/// use diem_proptest_helpers::GrowingSubset;
/// let items = vec![(1, "a"), (3, "c"), (2, "b"), (2, "d")];
/// let mut subset: GrowingSubset<_, _> = items.into_iter().collect();
///
/// assert_eq!(subset.total_len(), 4);
/// assert_eq!(subset.len(), 0);
/// assert_eq!(subset.current(), &[]);
///
/// subset.advance_to(&2);
/// assert_eq!(subset.len(), 1);
/// assert_eq!(subset.current(), &[(1, "a")]);
///
/// subset.advance_to(&4);
/// assert_eq!(subset.len(), 4);
/// assert_eq!(subset.current(), &[(1, "a"), (2, "b"), (2, "d"), (3, "c")]);
///
/// subset.advance_to(&5);
/// assert_eq!(subset.len(), 4);
/// assert_eq!(subset.current(), &[(1, "a"), (2, "b"), (2, "d"), (3, "c")]);
/// ```
#[derive(Clone, Debug)]
pub struct GrowingSubset<Ix, T> {
    // `items` needs to be in ascending order by index -- constructors of `GrowingSubset` should
    // sort the list of items.
    items: Vec<(Ix, T)>,
    current_pos: usize,
}

/// Constructs a `GrowingSubset` from an iterator of (index, value) pairs.
///
/// The input does not need to be pre-sorted.
impl<Ix, T> FromIterator<(Ix, T)> for GrowingSubset<Ix, T>
where
    Ix: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Ix, T)>,
    {
        let mut items: Vec<_> = iter.into_iter().collect();
        // Sorting is required to construct a subset correctly.
        items.sort_by(|x, y| x.0.cmp(&y.0));
        Self {
            items,
            current_pos: 0,
        }
    }
}

impl<Ix, T> GrowingSubset<Ix, T>
where
    Ix: Ord,
{
    /// Returns the number of elements in the *current subset*.
    ///
    /// See [`total_len`](GrowingSubset::total_len) for the length of the universal set.
    pub fn len(&self) -> usize {
        self.current_pos
    }

    /// Returns `true` if the *current subset* contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total number of elements in the universal set.
    ///
    /// This remains fixed once the `GrowingSubset` has been created.
    pub fn total_len(&self) -> usize {
        self.items.len()
    }

    /// Returns a slice containing the items in the *current subset*.
    pub fn current(&self) -> &[(Ix, T)] {
        &self.items[0..self.current_pos]
    }

    /// Chooses an (index, value) pair from the *current subset* using the provided
    /// [`Index`](proptest::sample::Index) instance as the source of randomness.
    pub fn pick_item(&self, index: &Index) -> &(Ix, T) {
        index.get(self.current())
    }

    /// Chooses a value from the *current subset* using the provided
    /// [`Index`](proptest::sample::Index) instance as the source of randomness.
    pub fn pick_value(&self, index: &Index) -> &T {
        &self.pick_item(index).1
    }

    /// Advances the valid subset to the provided index. After the end of this, the *current subset*
    /// will contain all elements where the index is *less than* `to_idx`.
    ///
    /// If duplicate indexes exist, `advance_to` will cause all of the corresponding items to be
    /// included.
    ///
    /// It is expected that `advance_to` will be called with larger indexes over time.
    pub fn advance_to(&mut self, to_idx: &Ix) {
        let len = self.items.len();
        while self.current_pos < len {
            let (idx, _) = &self.items[self.current_pos];
            if idx >= to_idx {
                break;
            }
            self.current_pos += 1;
        }
    }
}
