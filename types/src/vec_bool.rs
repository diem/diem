// SPDX-License-Identifier: Apache-2.0

use crate::nibble::nibble_path::NibblePath;

/// An space efficient Vec<Bool>, with each bit representing one boolean value.
///
/// This is used to represent the position of an node in a binary tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VecBool {
    len: usize,
    buckets: Vec<u8>,
}

impl VecBool {
    const BUCKET_LEN: usize = u8::BITS as usize;

    pub fn new() -> Self {
        Self {
            len: 0,
            buckets: Vec::new(),
        }
    }

    pub fn push(&mut self, bit: bool) {
        let pos = Self::bit_within_bucket(self.len);
        if pos == 0 {
            self.buckets.push(0);
        }
        let bucket = self.buckets.last_mut().expect("Last bucket must exist.");
        if bit {
            *bucket |= Self::bit_mask(pos)
        }
        self.len += 1;
    }

    pub fn with_pushed(mut self, bit: bool) -> Self {
        self.push(bit);
        self
    }

    pub fn pop(&mut self) -> Option<bool> {
        if self.is_empty() {
            return None;
        }

        let pos = Self::bit_within_bucket(self.len - 1);
        let bucket = self.buckets.last_mut().expect("Last bucket must exist.");
        let ret = Some(Self::get_in_bucket(*bucket, pos));
        *bucket &= !Self::bit_mask(pos);
        if pos == 0 {
            self.buckets.pop();
        }
        self.len -= 1;
        ret
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.len);

        let bucket = Self::bucket_index(index);
        let pos = Self::bit_within_bucket(index);

        Self::get_in_bucket(self.buckets[bucket], pos)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn try_to_nibble_path(&self) -> Option<NibblePath> {
        assert!(self.len <= 256);

        if self.len() % 4 == 0 {
            if self.len() % 8 == 0 {
                Some(NibblePath::new(self.buckets.clone()))
            } else {
                Some(NibblePath::new_odd(self.buckets.clone()))
            }
        } else {
            None
        }
    }

    fn bucket_index(idx_in_bits: usize) -> usize {
        (idx_in_bits / Self::BUCKET_LEN) as usize
    }

    fn bit_within_bucket(idx_in_bits: usize) -> usize {
        idx_in_bits % Self::BUCKET_LEN
    }

    fn get_in_bucket(bucket: u8, bit_in_bucket: usize) -> bool {
        bucket & Self::bit_mask(bit_in_bucket) != 0
    }

    fn bit_mask(bit_in_bucket: usize) -> u8 {
        assert!(bit_in_bucket < Self::BUCKET_LEN);

        1 << (Self::BUCKET_LEN - 1 - bit_in_bucket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{collection::vec, prelude::*};

    impl Arbitrary for VecBool {
        type Strategy = BoxedStrategy<Self>;
        type Parameters = ();

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            vec(any::<u8>(), 0..=32)
                .prop_flat_map(|buckets| {
                    let ignored_trailing_bits = if buckets.is_empty() {
                        0..=0
                    } else {
                        0..=std::cmp::min(7, buckets.last().unwrap().trailing_zeros() as usize)
                    };
                    (Just(buckets), ignored_trailing_bits)
                })
                .prop_filter("over length", |(buckets, ignored_trailing_bits)| {
                    buckets.len() < 32 || *ignored_trailing_bits > 0
                })
                .prop_map(|(buckets, ignored_trailing_bites)| {
                    let len = buckets.len() * 8 - ignored_trailing_bites;
                    Self { buckets, len }
                })
                .boxed()
        }
    }

    proptest! {
        #[test]
        fn test_push_pop(bits in any::<VecBool>()) {
            let mut cloned = bits.clone();
            let mut vec_bool = Vec::new();

            while let Some(bit) = cloned.pop() {
                vec_bool.push(bit);
            }

            let mut new = VecBool::new();
            vec_bool.into_iter().rev().for_each(|bit| new.push(bit));

            assert_eq!(new, bits)
        }

        #[test]
        fn test_at(bits in any::<VecBool>()) {
            let mut new = VecBool::new();

            for i in 0..bits.len() {
                new.push(bits.get(i))
            }

            assert_eq!(new, bits)
        }

        #[test]
        fn test_pop_clean(mut bits in any::<VecBool>().prop_filter("empty", |bits| !bits.is_empty())) {
            let mut new = VecBool::new();

            for i in 0..bits.len()-1 {
                new.push(bits.get(i))
            }

            bits.pop();

            assert_eq!(new, bits);
        }
    }
}
