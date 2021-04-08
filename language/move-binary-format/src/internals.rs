// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Types meant for use by other parts of this crate, and by other crates that are designed to
//! work with the internals of these data structures.

use crate::IndexKind;

/// Represents a module index.
pub trait ModuleIndex {
    const KIND: IndexKind;

    fn into_index(self) -> usize;
}
