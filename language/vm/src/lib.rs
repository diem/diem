// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

#[macro_use]
extern crate mirai_annotations;

use std::fmt;

pub mod access;
pub mod check_bounds;
#[macro_use]
pub mod errors;
pub mod deserializer;
pub mod file_format;
pub mod file_format_common;
pub mod gas_schedule;
pub mod internals;
#[cfg(any(test, feature = "fuzzing"))]
pub mod proptest_types;
pub mod serializer;
pub mod transaction_metadata;
pub mod views;

#[cfg(test)]
mod unit_tests;

pub use file_format::CompiledModule;

/// Represents a kind of index -- useful for error messages.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexKind {
    ModuleHandle,
    StructHandle,
    FunctionHandle,
    FieldHandle,
    FunctionInstantiation,
    FieldInstantiation,
    StructDefinition,
    StructDefInstantiation,
    FunctionDefinition,
    FieldDefinition,
    Signature,
    Identifier,
    ByteArrayPool,
    AddressPool,
    LocalPool,
    CodeDefinition,
    TypeParameter,
    MemberCount,
}

impl IndexKind {
    pub fn variants() -> &'static [IndexKind] {
        use IndexKind::*;

        // XXX ensure this list stays up to date!
        &[
            ByteArrayPool,
            ModuleHandle,
            StructHandle,
            FunctionHandle,
            FieldHandle,
            StructDefInstantiation,
            FunctionInstantiation,
            FieldInstantiation,
            StructDefinition,
            FunctionDefinition,
            FieldDefinition,
            Signature,
            Identifier,
            AddressPool,
            LocalPool,
            CodeDefinition,
            TypeParameter,
            MemberCount,
        ]
    }
}

impl fmt::Display for IndexKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use IndexKind::*;

        let desc = match self {
            ModuleHandle => "module handle",
            StructHandle => "struct handle",
            FunctionHandle => "function handle",
            FieldHandle => "field handle",
            StructDefInstantiation => "struct instantiation",
            FunctionInstantiation => "function instantiation",
            FieldInstantiation => "field instantiation",
            StructDefinition => "struct definition",
            FunctionDefinition => "function definition",
            FieldDefinition => "field definition",
            Signature => "signature",
            Identifier => "identifier",
            ByteArrayPool => "byte_array pool",
            AddressPool => "address pool",
            LocalPool => "local pool",
            CodeDefinition => "code definition pool",
            TypeParameter => "type parameter",
            MemberCount => "field offset",
        };

        f.write_str(desc)
    }
}

// TODO: is this outdated?
/// Represents the kind of a signature token.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SignatureTokenKind {
    /// Any sort of owned value that isn't an array (Integer, Bool, Struct etc).
    Value,
    /// A reference.
    Reference,
    /// A mutable reference.
    MutableReference,
}

impl fmt::Display for SignatureTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SignatureTokenKind::*;

        let desc = match self {
            Value => "value",
            Reference => "reference",
            MutableReference => "mutable reference",
        };

        f.write_str(desc)
    }
}
