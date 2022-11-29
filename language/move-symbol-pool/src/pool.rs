// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A pool of uniqued string data akin to a hash set.
//!
//! The string pool is represented as a statically sized contiguous array of
//! buckets. Each bucket represents space within which an entry in the pool
//! may be allocated. An entry is an intrusively-linked list of with two data:
//! a heap-allocated string, and the hash value of that string data.
//!
//! Insertion performance into the pool is comparable to a hash set, because the
//! underlying data structure is similar: a hashing function determines which
//! bucket in the array the string ought to be inserted into. The entries in
//! that bucket are iterated upon. If no entries match the string, a new
//! entry is appended to the end of the linked list of entries.
//!
//! * Why not use a [`HashSet`]? A set is dynamically resized as elements are
//!   added. Ideally converting a [`Symbol`] to its string value is as
//!   performant as dereferencing a pointer. But implementing the [`Symbol]` as
//!   a pointer would not be safe if the data being pointed to could be
//!   reallocated.
//! * Why not use a [`LinkedList`]? A linked list does not unique the elements.
//!   Ensuring the elements in the list are unique would require traversing the
//!   list, which would not be performant for large lists.
//! * Why not use a [`HashSet`] in conjunction with a [`LinkedList`]? This would
//!   be simpler to implement, but would involve storing the string data twice:
//!   once in the set for uniqueness checking, and once in the linked list to
//!   maintain a constant memory address for the string data. This pool
//!   implementation is more space-efficient.
//!
//! [`Symbol`]: crate::Symbol
//! [`HashSet`]: std::collections::HashSet
//! [`LinkedList`]: std::collections::LinkedList

use std::{
    borrow::Cow,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    ptr::NonNull,
};

/// The number of buckets in the pool's contiguous array.
const NB_BUCKETS: usize = 1 << 12; // 4096
/// A mask used to convert a string hash into the index of one of the contiguous buckets.
const BUCKET_MASK: u64 = NB_BUCKETS as u64 - 1;

/// A bucket is a space on the heap within which an entry may be allocated.
type Bucket = Option<Box<Entry>>;

/// A string in the pool.
pub(crate) struct Entry {
    pub(crate) string: Box<str>,
    hash: u64,
    next: Bucket,
}

/// A contiguous array of buckets.
pub(crate) struct Pool(pub(crate) Box<[Bucket; NB_BUCKETS]>);

impl Pool {
    /// Allocates a contiguous array of buckets on the heap. As strings are
    /// inserted into the pool, buckets in this array are filled with an entry.
    pub(crate) fn new() -> Self {
        let vec = std::mem::ManuallyDrop::new(vec![0_usize; NB_BUCKETS]);
        Self(unsafe { Box::from_raw(vec.as_ptr() as *mut [Bucket; NB_BUCKETS]) })
    }

    /// Computes the hash value of a string, which is used to determine both
    /// which top-level bucket contains an entry corresponding to the string,
    /// as well as a scalar value that can be used to quickly check whether
    /// two strings are not equal.
    fn hash(string: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        string.hash(&mut hasher);
        hasher.finish()
    }

    /// Given a string, returns its entry in the pool (adding it if it does not
    /// yet exist in the pool).
    pub(crate) fn insert(&mut self, string: Cow<str>) -> NonNull<Entry> {
        let hash = Self::hash(&string);
        // Access the top-level bucket in the pool's contiguous array that
        // contains the linked list of entries that contain the string.
        let bucket_index = (hash & BUCKET_MASK) as usize;
        let mut ptr: Option<&mut Box<Entry>> = self.0[bucket_index].as_mut();

        // Iterate over the entires in the bucket.
        while let Some(entry) = ptr.take() {
            // If we find the string we're looking for, don't add anything to
            // the pool. Instead, just return the existing entry.
            // NOTE: Strings with different hash values can't possibly be equal,
            // so comparing those hash values first ought to filter out unequal
            // strings faster than comparing the strings themselves.
            if entry.hash == hash && *entry.string == *string {
                return NonNull::from(&mut **entry);
            }
            ptr = entry.next.as_mut();
        }

        // The string doesn't exist in the pool yet; insert it at the head of
        // the linked list of entries.
        let mut entry = Box::new(Entry {
            string: string.into_owned().into_boxed_str(),
            hash,
            next: self.0[bucket_index].take(),
        });
        let ptr = NonNull::from(&mut *entry);

        // The bucket in the top-level contiguous array now points to the new
        // head of the linked list.
        self.0[bucket_index] = Some(entry);

        ptr
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::Pool;

    #[test]
    fn test_insert_identical_strings_have_the_same_entry() {
        let mut pool = Pool::new();
        let e1 = pool.insert(Cow::Borrowed("hi"));
        let e2 = pool.insert(Cow::Owned("hi".to_owned()));
        assert_eq!(e1, e2);
    }
}
