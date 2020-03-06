// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Logic for generating valid stack states for native function calls.
//!
//! This implements a `StackAccessor` that generates random bytearrays of a user defined length. We
//! then use this ability to run the native functions with different bytearray lengths in the
//! generated synthesis binary.
use rand::{rngs::StdRng, Rng, SeedableRng};

/// A wrapper around data used to generate random valid bytearrays
pub struct StackAccessorMocker {
    gen: StdRng,
    /// The length of bytearrays that will be generated.
    pub hash_length: usize,
}

impl StackAccessorMocker {
    /// Builds a new stack accessor mocker. User is responsible for later setting the length of,
    /// and generating the underlying bytearray.
    pub fn new() -> Self {
        let seed: [u8; 32] = [0; 32];
        Self {
            gen: StdRng::from_seed(seed),
            hash_length: 1,
        }
    }

    /// Set the bytearray length that will be generated in calls to `next_bytearray`.
    pub fn set_hash_length(&mut self, len: usize) {
        self.hash_length = len;
    }

    /// Generage a fresh bytearray.
    pub fn next_bytearray(&mut self) -> Vec<u8> {
        let bytes: Vec<u8> = (0..self.hash_length)
            .map(|_| self.gen.gen::<u8>())
            .collect();
        bytes
    }
}

impl Default for StackAccessorMocker {
    fn default() -> Self {
        Self::new()
    }
}
