// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A global, uniqued cache of strings that is never purged.
//!
//! This module provides storage for strings that are meant to remain in use for
//! the entire running duration of a program. Strings that are stored in this
//! global, static cache are never evicted, and so the memory consumed by them
//! can only ever grow.
//!
//! The strings can be accessed via the [`Symbol`] type, which acts like a
//! pointer to the underlying cached string value.
//!
//! NOTE: If you're looking for a `#[forbid(unsafe_code)]` attribute here, you
//! won't find one: symbol-pool (and its inspiration, servo/string-cache) uses
//! `unsafe` Rust in order to store and dereference `Symbol` pointers to
//! strings.
//!
//! [`Symbol`]: crate::move_symbol_pool::Symbol

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashSet, fmt, num::NonZeroU64, sync::Mutex};

type SymbolData = Box<str>;
static SYMBOL_POOL: Lazy<Mutex<HashSet<SymbolData>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Represents a string that has been cached.
///
/// A `Symbol` represents a string; it is not the string data itself. As such
/// this representation can implement `Copy` and other traits that some string
/// types cannot.
///
/// The strings that `Symbol` types represent are added to the global cache as
/// the `Symbol` are created.
///
/// ```
///# use crate::move_symbol_pool::Symbol;
/// let s1 = Symbol::from("hi"); // "hi" is stored in the global cache
/// let s2 = Symbol::from("hi"); // "hi" is already stored, cache does not grow
/// assert_eq!(s1, s2);
/// ```
///
/// Use the method [`as_str()`] to access the string value that a `Symbol`
/// represents. `Symbol` also implements the [`Display`] trait, so it can be
/// printed as an ordinary string would. This makes it easier to use with
/// crates that print strings to a terminal, such as codespan.
///
/// ```
///# use crate::move_symbol_pool::Symbol;
/// let message = format!("{} {}",
///     Symbol::from("hello").as_str(),
///     Symbol::from("world"));
/// assert_eq!(message, "hello world");
/// ```
///
/// [`as_str()`]: crate::move_symbol_pool::Symbol::as_str
/// [`Display`]: std::fmt::Display
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Symbol(NonZeroU64);

impl Symbol {
    pub fn as_str(&self) -> &str {
        let ptr = self.0.get() as *const SymbolData;
        unsafe { &(*ptr) }
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        let mut pool = SYMBOL_POOL.lock().expect("could not acquire lock on pool");
        pool.insert(s.to_owned().into_boxed_str());
        let interned = pool.get(s).expect("string must be in pool");
        let ptr: *const SymbolData = &*interned;
        Symbol(NonZeroU64::new(ptr as u64).expect("pointer to interned string cannot be null"))
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Symbol) -> Ordering {
        if self.0 == other.0 {
            Ordering::Equal
        } else {
            self.as_str().cmp(other.as_str())
        }
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Symbol) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[test]
fn test_from() {
    Symbol::from("this shouldn't panic");
}

#[test]
fn test_as_str() {
    let s = Symbol::from("hi");
    assert_eq!(s.as_str(), "hi");
}

#[test]
fn test_display() {
    assert_eq!(format!("{}", Symbol::from("hola")), "hola");
}

#[test]
fn test_ord() {
    assert!(Symbol::from("aardvark") < Symbol::from("bear"));
    assert!(Symbol::from("bear") <= Symbol::from("bear"));
    assert!(Symbol::from("cat") > Symbol::from("bear"));
    assert!(Symbol::from("dog") >= Symbol::from("cat"));
}

#[test]
fn test_two_identical_strings_have_identical_addresses() {
    // When two identical strings are added to the symbol pool, they should be
    // deduplicated, resulting in only one string being allocated on the heap.
    let sym_one = Symbol::from("/path/to/file.move");
    let sym_two = Symbol::from("/path/to/file.move");
    // Verify that the memory address of both strings are equal.
    assert_eq!(sym_one.0, sym_two.0);
}
