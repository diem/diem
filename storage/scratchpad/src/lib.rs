// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate provides in-memory representation of Diem core data structures used by the executor.

mod sparse_merkle;

pub use crate::sparse_merkle::{AccountStatus, ProofRead, SparseMerkleTree};
