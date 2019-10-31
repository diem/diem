// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A map used for VM runtime caches. The data structures are highly specialized for the
//! VM runtime and are probably not useful outside of it.

mod arena;
mod cache_map;
#[cfg(test)]
mod unit_tests;

pub use arena::Arena;
pub use cache_map::{CacheMap, CacheRefMap};
