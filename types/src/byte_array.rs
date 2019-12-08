// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use hex;
use serde::{Deserialize, Serialize};

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Default, Clone, Serialize, Deserialize)]
/// A struct that represents a ByteArray in Move.
pub struct ByteArray(Vec<u8>);

impl ByteArray {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn new(buf: Vec<u8>) -> Self {
        ByteArray(buf)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl std::fmt::Debug for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl std::fmt::Display for ByteArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "b\"{}\"", hex::encode(&self.0))
    }
}

impl std::ops::Index<usize> for ByteArray {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        std::ops::Index::index(&*self.0, index)
    }
}

/* TODO: Once we implement char as byte, then we can allow for Range Slicing of ByteArrays
impl std::ops::Index<std::ops::RangeToInclusive<usize>> for ByteArray {
    type Output = [u8];

    #[inline]
    fn index(&self, index: std::ops::RangeToInclusive<usize>) -> &Self::Output {
        std::ops::Index::index(&*self.0, index)
    }
}
*/
