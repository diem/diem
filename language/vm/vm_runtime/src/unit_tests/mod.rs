// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use types::identifier::Identifier;

mod identifier_prop_tests;
mod module_cache_tests;
mod runtime_tests;

// Helper methods for identifiers in tests.

fn ident(name: impl Into<Box<str>>) -> Identifier {
    Identifier::new(name).unwrap()
}

fn idents(names: impl IntoIterator<Item = &'static str>) -> Vec<Identifier> {
    names.into_iter().map(ident).collect()
}
