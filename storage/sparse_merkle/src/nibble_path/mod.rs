// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! NibblePath library simplify operations with nibbles in a compact format for modified sparse
//! Merkle tree by providing powerful iterators advancing by either bit or nibble.

#[cfg(test)]
mod nibble_path_test;

use crate::ROOT_NIBBLE_HEIGHT;
use std::{fmt, iter::FromIterator};

/// NibblePath defines a path in Merkle tree in the unit of nibble (4 bits)
#[derive(Clone, Eq, PartialEq)]
pub struct NibblePath {
    /// the underlying bytes that stores the path, 2 nibbles per byte. If the number of nibbles is
    /// odd, the second half of the last byte must be 0.
    bytes: Vec<u8>,
    /// Indicates the total number of nibbles in bytes. Either `bytes.len() * 2 - 1` or
    /// `bytes.len() * 2`.
    num_nibbles: usize,
}

/// Support debug format by concatenating nibbles literally. For example, [0x12, 0xa0] with 3
/// nibbles will be printed as "12a".
impl fmt::Debug for NibblePath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.nibbles().map(|x| write!(f, "{:x}", x)).collect()
    }
}

/// Convert a vector of bytes into `NibblePath` using the lower 4 bits of each byte as nibble.
impl FromIterator<u8> for NibblePath {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        let mut bytes = vec![];
        let mut count = 0;
        for nibble in iter {
            assert!(nibble < 16);
            if count % 2 == 0 {
                bytes.push(nibble << 4);
            } else {
                *bytes.last_mut().expect("Cannot be None") |= nibble;
            }
            count += 1;
        }
        if count % 2 == 0 {
            NibblePath::new(bytes)
        } else {
            NibblePath::new_odd(bytes)
        }
    }
}

impl NibblePath {
    /// Creates a new `NibblePath` from a vector of bytes assuming each byte has 2 nibbles.
    pub fn new(bytes: Vec<u8>) -> Self {
        let num_nibbles = bytes.len() * 2;
        assert!(num_nibbles <= ROOT_NIBBLE_HEIGHT);
        NibblePath { bytes, num_nibbles }
    }

    /// Similar to `new()` but assumes that the bytes have one less nibble.
    pub fn new_odd(bytes: Vec<u8>) -> Self {
        assert_eq!(
            bytes.last().expect("Should have odd number of nibbles.") & 0x0f,
            0,
            "Last nibble must be 0."
        );
        let num_nibbles = bytes.len() * 2 - 1;
        assert!(num_nibbles <= ROOT_NIBBLE_HEIGHT);
        NibblePath { bytes, num_nibbles }
    }

    /// Get the i-th bit.
    fn get_bit(&self, i: usize) -> bool {
        assert!(i / 4 < self.num_nibbles);
        let pos = i / 8;
        let bit = 7 - i % 8;
        ((self.bytes[pos] >> bit) & 1) != 0
    }

    /// Get the i-th nibble, stored at lower 4 bits
    fn get_nibble(&self, i: usize) -> u8 {
        assert!(i < self.num_nibbles);
        (self.bytes[i / 2] >> (if i % 2 == 1 { 0 } else { 4 })) & 0xf
    }

    /// Get a bit iterator iterates over the whole nibble path.
    pub fn bits(&self) -> BitIterator {
        BitIterator {
            nibble_path: self,
            pos: (0..self.num_nibbles * 4),
        }
    }

    /// Get a nibble iterator iterates over the whole nibble path.
    pub fn nibbles(&self) -> NibbleIterator {
        NibbleIterator::new(self, 0, self.num_nibbles)
    }

    /// Get the total number of nibbles stored.
    pub fn num_nibbles(&self) -> usize {
        self.num_nibbles
    }

    /// Get the underlying bytes storing nibbles.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub(crate) trait Peekable: Iterator {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item>;
}

/// BitIterator iterates a nibble path by bit.
pub struct BitIterator<'a> {
    nibble_path: &'a NibblePath,
    pos: std::ops::Range<usize>,
}

impl<'a> Peekable for BitIterator<'a> {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item> {
        if self.pos.start < self.pos.end {
            Some(self.nibble_path.get_bit(self.pos.start))
        } else {
            None
        }
    }
}

/// BitIterator spits out a boolean each time. True/false denotes 1/0.
impl<'a> Iterator for BitIterator<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        self.pos
            .next()
            .and_then(|i| Some(self.nibble_path.get_bit(i)))
    }
}

/// Support iterating bits in reversed order.
impl<'a> DoubleEndedIterator for BitIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.pos
            .next_back()
            .and_then(|i| Some(self.nibble_path.get_bit(i)))
    }
}

/// NibbleIterator iterates a nibble path by nibble.
pub struct NibbleIterator<'a> {
    /// The underlying nibble path that stores the nibbles
    nibble_path: &'a NibblePath,

    /// The current index, `pos.start`, will bump by 1 after calling `next()` until `pos.start ==
    /// pos.end`.
    pos: std::ops::Range<usize>,

    /// The start index of the iterator. At the beginning, `pos.start == start`. [start, pos.end)
    /// defines the range of `nibble_path` this iterator iterates over. `nibble_path` refers to
    /// the entire underlying buffer but the range may only be partial.
    start: usize,
}

/// NibbleIterator spits out a byte each time. Each byte must be in range [0, 16).
impl<'a> Iterator for NibbleIterator<'a> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        self.pos
            .next()
            .and_then(|i| Some(self.nibble_path.get_nibble(i)))
    }
}

impl<'a> Peekable for NibbleIterator<'a> {
    /// Returns the `next()` value without advancing the iterator.
    fn peek(&self) -> Option<Self::Item> {
        if self.pos.start < self.pos.end {
            Some(self.nibble_path.get_nibble(self.pos.start))
        } else {
            None
        }
    }
}

impl<'a> NibbleIterator<'a> {
    fn new(nibble_path: &'a NibblePath, start: usize, end: usize) -> Self {
        Self {
            nibble_path,
            pos: (start..end),
            start,
        }
    }

    /// Returns a nibble iterator that iterates all visited nibbles.
    pub fn visited_nibbles(&self) -> NibbleIterator<'a> {
        Self::new(self.nibble_path, self.start, self.pos.start)
    }

    /// Returns a nibble iterator that iterates all remaining nibbles.
    pub fn remaining_nibbles(&self) -> NibbleIterator<'a> {
        Self::new(self.nibble_path, self.pos.start, self.pos.end)
    }

    /// Turn it into a `BitIterator`.
    pub fn bits(&self) -> BitIterator<'a> {
        BitIterator {
            nibble_path: self.nibble_path,
            pos: (self.pos.start * 4..self.pos.end * 4),
        }
    }

    /// Cut and return the range of the underlying `nibble_path` that this iterator is iterating
    /// over as a new `NibblePath`
    pub fn get_nibble_path(&self) -> NibblePath {
        self.visited_nibbles()
            .chain(self.remaining_nibbles())
            .collect()
    }

    /// Get the number of nibbles that this iterator covers.
    pub fn num_nibbles(&self) -> usize {
        self.pos.end - self.start
    }

    /// Return `true` if the iteration is over.
    pub fn is_finished(&self) -> bool {
        self.peek().is_none()
    }
}

/// Advance both iterators if their next nibbles are the same until either reaches the end or
/// the find a mismatch. Return the number of matched nibbles.
pub(crate) fn skip_common_prefix<'a, 'b, I1: 'a, I2: 'b>(x: &'a mut I1, y: &mut I2) -> usize
where
    I1: Iterator + Peekable,
    I2: Iterator + Peekable,
    <I1 as Iterator>::Item: std::cmp::PartialEq<<I2 as Iterator>::Item>,
{
    let mut count = 0;
    loop {
        let x_peek = x.peek();
        let y_peek = y.peek();
        if x_peek.is_none()
            || y_peek.is_none()
            || x_peek.expect("cannot be none") != y_peek.expect("cannot be none")
        {
            break;
        }
        count += 1;
        x.next();
        y.next();
    }
    count
}
