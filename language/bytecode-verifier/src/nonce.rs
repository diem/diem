// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the Nonce type used for borrow checking in the abstract interpreter.
//! A Nonce instance represents an arbitrary reference or access path.
//! The integer inside a Nonce is meaningless; only equality and borrow relationships are
//! meaningful.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Nonce(usize);

impl Nonce {
    pub fn new(n: usize) -> Self {
        Self(n)
    }

    pub fn is(self, n: usize) -> bool {
        self.0 == n
    }

    pub fn inner(self) -> usize {
        self.0
    }
}
