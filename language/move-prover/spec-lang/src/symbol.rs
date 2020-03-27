// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Contains definitions of symbols -- internalized strings which support fast hashing and
//! comparison.

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt,
    fmt::{Error, Formatter},
    rc::Rc,
};

/// Representation of a symbol.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct Symbol(usize);

impl Symbol {
    pub fn display<'a>(&'a self, pool: &'a SymbolPool) -> SymbolDisplay<'a> {
        SymbolDisplay { sym: self, pool }
    }
}

/// A helper to support symbols in formatting.
pub struct SymbolDisplay<'a> {
    sym: &'a Symbol,
    pool: &'a SymbolPool,
}

impl<'a> fmt::Display for SymbolDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&self.pool.string(*self.sym))
    }
}

/// A pool of symbols. Allows to lookup a symbol by a string representation, and discover
/// the string representation of an existing symbol. This struct does not need be mutable
/// for operations on it, which is important so references to it can be freely passed around.
#[derive(Debug)]
pub struct SymbolPool {
    inner: RefCell<InnerPool>,
}

#[derive(Debug)]
struct InnerPool {
    strings: Vec<Rc<String>>,
    lookup: HashMap<Rc<String>, usize>,
}

impl SymbolPool {
    /// Creates a new SymbolPool.
    pub fn new() -> SymbolPool {
        SymbolPool {
            inner: RefCell::new(InnerPool {
                strings: vec![],
                lookup: HashMap::new(),
            }),
        }
    }

    /// Looks up a symbol by its string representation. If a symbol with this representation
    /// already exists, it will be returned, otherwise a new one will be created in the
    /// pool. The implementation uses internally a RefCell for storing symbols, so the pool
    /// does not need to be mutable.
    pub fn make(&self, s: &str) -> Symbol {
        let mut pool = self.inner.borrow_mut();
        let key = Rc::new(s.to_string());
        if let Some(n) = pool.lookup.get(&key) {
            return Symbol(*n);
        }
        let new_sym = pool.strings.len();
        pool.strings.push(key.clone());
        pool.lookup.insert(key, new_sym);
        Symbol(new_sym)
    }

    /// Returns the string representation of this symbol, as an rc'ed string to avoid copies.
    /// If the past symbol was not created from this pool, a runtime error may happen (or a wrong
    /// string will be returned).
    pub fn string(&self, sym: Symbol) -> Rc<String> {
        self.inner.borrow().strings[sym.0].clone()
    }
}

impl Default for SymbolPool {
    fn default() -> Self {
        Self::new()
    }
}
