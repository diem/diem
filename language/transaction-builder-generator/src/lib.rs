// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::language_storage::TypeTag;

/// Support for code-generation in Python 3
pub mod python3;
/// Support for code-generation in Rust
pub mod rust;

/// Useful error message.
fn type_not_allowed(type_tag: &TypeTag) -> ! {
    panic!(
        "Transaction scripts cannot take arguments of type {}.",
        type_tag
    );
}
