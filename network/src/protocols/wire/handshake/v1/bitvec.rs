// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This library defines a BitVec struct that represents a bit vector.

use bytes::{BufMut, BytesMut};
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::ops::BitAnd;

// Every u8 is used as a bucket of 8 bits. Total max buckets = 256 / 8 = 32.
const BUCKET_SIZE: usize = 8;
const MAX_BUCKETS: usize = 32;

/// BitVec represents a bit vector that upports only 2 operations:
/// 1. Marking a position as set, and
/// 2. Checking if a position is set.
/// Internally, it stores a vector of u8's (as Bytes).
/// * The first 8 positions of the bit vector are encoded in the first element of the vector, the
///   next 8 are encoded in the second element, and so on.
/// * Each bit of a u8 is set to 1 if the position is set and to 0 if it's not.
/// * We only allow setting positions upto u8::MAX. As a result, the size of the inner vector is
///   limited to 32 (= 256 / 8).
/// * Once a bit has been set, it cannot be unset. As a result, the inner vector cannot shrink.
/// * The positions can be set in any order.
/// * A position can set more than once -- it remains set after the first time.
///
/// # Examples:
/// ```ignore
/// let mut bv = BitVec::default();
/// bv.set(2);
/// bv.set(5);
/// assert!(bv.is_set(2));
/// assert!(bv.is_set(5));
/// assert_eq!(false, bv.is_set(0));
///
/// // A bitwise AND of BitVec can be performed by using the `&` operator.
/// let bv1 = BitVec::default();
/// bv1.set(2);
/// bv1.set(3);
/// let bv2 = BitVec::default();
/// bv2.set(2);
/// let intersection = bv1 & bv2;
/// assert!(intersection.is_set(2));
/// assert_eq!(false, intersection.is_set(3));
/// ```
#[derive(Clone, Default, Debug, PartialEq, Serialize)]
pub struct BitVec {
    inner: BytesMut,
}

impl BitVec {
    // TODO(abhayb): Remove after migration to new wire format.
    #[allow(dead_code)]
    /// Sets the bit at position @pos.
    pub fn set(&mut self, pos: u8) {
        // This is optimised to: let bucket = pos << 3;
        let bucket: usize = pos as usize / BUCKET_SIZE;
        if self.inner.len() <= bucket {
            self.inner.resize(bucket + 1, 0);
        }
        // This is optimized to: let bucket_pos = pos | 0x07;
        let bucket_pos = pos as usize - (bucket * BUCKET_SIZE);
        self.inner[bucket] |= 0x01 << bucket_pos;
    }

    // TODO(abhayb): Remove after migration to new wire format.
    #[allow(dead_code)]
    /// Checks if the bit at position @pos is set.
    pub fn is_set(&self, pos: u8) -> bool {
        // This is optimised to: let bucket = pos << 3;
        let bucket: usize = pos as usize / BUCKET_SIZE;
        if self.inner.len() <= bucket {
            return false;
        }
        // This is optimized to: let bucket_pos = pos | 0x07;
        let bucket_pos = pos as usize - (bucket * BUCKET_SIZE);
        (self.inner[bucket] & (0x01 << bucket_pos)) != 0
    }
}

impl BitAnd for BitVec {
    type Output = BitVec;

    /// Returns a new BitVec that is a bitwise AND of two BitVecs.
    fn bitand(self, other: Self) -> Self {
        let len = std::cmp::min(self.inner.len(), other.inner.len());
        let mut ret = BitVec {
            inner: BytesMut::with_capacity(len),
        };
        for i in 0..len {
            ret.inner.put_u8(self.inner[i] & other.inner[i]);
        }
        ret
    }
}

// We impl custom deseriazation to ensure that the length of inner vector does not exceed 32 (=
// 256 / 8).
impl<'de> Deserialize<'de> for BitVec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = <BytesMut>::deserialize(deserializer)?;
        if v.len() > MAX_BUCKETS {
            return Err(D::Error::custom(format!("BitVec too long: {}", v.len())));
        }
        Ok(BitVec { inner: v })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use proptest::{arbitrary::any, collection::vec, prelude::*};

    #[test]
    fn test_empty() {
        let p = BitVec::default();
        for i in 0..std::u8::MAX {
            assert_eq!(false, p.is_set(i));
        }
    }

    #[test]
    fn test_extremes() {
        let mut p = BitVec::default();
        p.set(std::u8::MAX);
        p.set(0);
        assert!(p.is_set(std::u8::MAX));
        assert!(p.is_set(0));
        for i in 1..std::u8::MAX {
            assert_eq!(false, p.is_set(i));
        }
    }

    #[test]
    fn test_deserialization() {
        // First 4 bytes represent length.
        let mut bytes = [0u8; 50];
        bytes[0] = 46;
        assert!(lcs::from_bytes::<BitVec>(&bytes).is_err());
        let mut bytes = [0u8; 36];
        bytes[0] = 32;
        let bv = BitVec {
            inner: BytesMut::from([0u8; 32].as_ref()),
        };
        assert_eq!(Ok(bv), lcs::from_bytes::<BitVec>(&bytes));
    }

    // Constructs a bit vector by setting the positions specified in the argument vector. The
    // vector can have duplicates and need not be sorted.
    fn construct_bitvec(posns: &[u8]) -> BitVec {
        let mut bv = BitVec::default();
        posns.iter().for_each(|x| bv.set(*x));
        bv
    }

    // Proptest for ensuring is_set returns true iff corresponding position was set.
    proptest! {
        #[test]
        fn test_arbitrary(mut v in vec(any::<u8>(), 0..256)) {
            let bv = construct_bitvec(&v);
            // Sort and dedup the vector so we can iterate over its elements from smallest to largest.
            v.sort_unstable();
            v.dedup();
            let mut viter = v.into_iter().peekable();
            // Positions in bv should be set iff they are in v.
            for i in 0..std::u8::MAX {
                if viter.peek() == Some(&i) {
                    prop_assert!(bv.is_set(i));
                    viter.next();
                } else {
                    prop_assert_eq!(false, bv.is_set(i));
                }
            }

        }
    }

    // Test for bitwise AND operation on 2 bitvecs.
    proptest! {
        #[test]
        fn test_and(v1 in vec(any::<u8>(), 0..256), v2 in vec(any::<u8>(), 0..256)) {
            let bv1 = construct_bitvec(&v1);
            let bv2 = construct_bitvec(&v2);
            let intersection = bv1.clone() & bv2.clone();
            for i in 0..std::u8::MAX {
                if bv1.is_set(i) && bv2.is_set(i) {
                    prop_assert!(intersection.is_set(i));
                } else {
                    prop_assert_eq!(false, intersection.is_set(i));
                }
            }

        }
    }
}
