// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate provides in-memory representation of Libra core data structures used by the executor.

mod sparse_merkle;

pub use crate::sparse_merkle::{AccountStatus, ProofRead, SparseMerkleTree};
