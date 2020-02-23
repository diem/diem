// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module implements the RefID type used for borrow checking in the abstract interpreter.
//! A RefID instance represents an arbitrary reference or access path.
//! The integer inside a RefID is meaningless; only equality and borrow relationships are
//! meaningful.
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct RefID(usize);

impl RefID {
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
