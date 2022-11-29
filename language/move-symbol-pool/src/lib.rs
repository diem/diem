// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A global, uniqued cache of strings that is never purged. Inspired by
//! [servo/string-cache].
//!
//! This module provides storage for strings that are meant to remain in use for
//! the entire running duration of a program. Strings that are stored in this
//! global, static cache are never evicted, and so the memory consumed by them
//! can only ever grow.
//!
//! The strings can be accessed via the [`Symbol`] type, which acts as a pointer
//! to the underlying string data.
//!
//! NOTE: If you're looking for a `#[forbid(unsafe_code)]` attribute here, you
//! won't find one: symbol-pool (and its inspiration, servo/string-cache) uses
//! `unsafe` Rust in order to store and dereference `Symbol` pointers to
//! strings.
//!
//! [servo/string-cache]: https://github.com/servo/string-cache
//! [`Symbol`]: crate::Symbol

mod pool;
pub mod symbol;

use once_cell::sync::Lazy;
use pool::Pool;
use std::sync::Mutex;

pub use symbol::Symbol;

/// The global, unique cache of strings.
pub(crate) static SYMBOL_POOL: Lazy<Mutex<Pool>> = Lazy::new(|| Mutex::new(Pool::new()));

#[cfg(test)]
mod tests {
    use crate::{Pool, Symbol, SYMBOL_POOL};
    use std::mem::replace;

    #[test]
    fn test_serialization() {
        // Internally, a Symbol behaves like a pointer. Naively serializing it
        // as an address in the pool is incorrect, as it may be serialized by
        // one process with its own pool, and deserialized by another process
        // with a different pool.
        let s = Symbol::from("serialize me!");
        let serialized = serde_json::to_string(&s).unwrap();

        // Artificially reset the pool for testing purposes. The address pointed
        // to by the Symbol is now no longer valid.
        let _ = replace(&mut SYMBOL_POOL.lock().unwrap().0, Pool::new().0);

        // Below, test that deserialization still succeeds.
        let deserialized: Symbol = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.as_str(), "serialize me!");
    }
}
