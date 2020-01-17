// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::ast::{BinOp, CopyableVal, Field, QualifiedStructIdent, Type};
use libra_types::account_address::AccountAddress;

/// AST for the Move Prover specification language. Just postconditions for now

/// A location that can store a value
#[derive(PartialEq, Debug, Clone)]
pub enum StorageLocation {
    /// A formal of the current procedure
    Formal(String),
    /// A resource of type `type_` stored in global storage at `address`
    GlobalResource {
        type_: QualifiedStructIdent,
        type_actuals: Vec<Type>,
        address: Box<StorageLocation>,
    },
    /// An access path rooted at `base` with nonempty offsets in `fields`
    AccessPath {
        base: Box<StorageLocation>,
        fields: Vec<Field>,
    },
    /// Sender address for the current transaction
    TxnSenderAddress,
    /// Account address constant
    Address(AccountAddress),
    /// The ith return value of the current procedure
    Ret(u8),
    // TODO: useful constants like U64_MAX
}

/// An expression in the specification language
#[derive(PartialEq, Debug, Clone)]
pub enum SpecExp {
    /// A Move constant
    Constant(CopyableVal),
    /// A spec language storage location
    StorageLocation(StorageLocation),
    /// Lifting the Move exists operator to a storage location
    GlobalExists {
        type_: QualifiedStructIdent,
        type_actuals: Vec<Type>,
        address: StorageLocation,
    },
    /// Dereference of a storage location (written *s)
    Dereference(StorageLocation),
    /// Reference to a storage location (written &s)
    Reference(StorageLocation),
    /// Negation of a boolean expression (written !e),
    Not(Box<SpecExp>),
    /// Binary operators also suported by Move
    Binop(Box<SpecExp>, BinOp, Box<SpecExp>),
    // TODO: binary operators not supported by Move like implies and iff
    /// Value of expression evaluated in the state before function enter.
    Old(Box<SpecExp>),
}

/// A specification directive to be verified
#[derive(PartialEq, Debug, Clone)]
pub enum Condition {
    /// Postconditions
    Ensures(SpecExp),
    /// Preconditions
    Requires(SpecExp),
    /// If the given expression is true, the procedure *must* terminate in an aborting state
    AbortsIf(SpecExp),
    /// If the given expression is true, the procedure *must* terminate in a succeeding state
    SucceedsIf(SpecExp),
}
