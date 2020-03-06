// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ast::{BinOp, CopyableVal_, Field_, QualifiedStructIdent, Type},
    location::*,
};
use libra_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

/// AST for the Move Prover specification language.

// .foo or [x + 1]
#[derive(PartialEq, Debug, Clone)]
pub enum FieldOrIndex {
    Field(Field_),
    Index(SpecExp),
}

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
    /// An access path rooted at `base` with nonempty offsets in `fields_or_indices`
    AccessPath {
        base: Box<StorageLocation>,
        fields_and_indices: Vec<FieldOrIndex>,
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
    Constant(CopyableVal_),
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
    /// Update expr (i := 1 inside [])
    Update(Box<SpecExp>, Box<SpecExp>),
    /// Value of expression evaluated in the state before function enter.
    Old(Box<SpecExp>),
    /// Call to a helper function.
    Call(String, Vec<SpecExp>),
}

/// A specification directive to be verified
#[derive(PartialEq, Debug, Clone)]
pub enum Condition_ {
    /// Postconditions
    Ensures(SpecExp),
    /// Preconditions
    Requires(SpecExp),
    /// If the given expression is true, the procedure *must* terminate in an aborting state
    AbortsIf(SpecExp),
    /// If the given expression is true, the procedure *must* terminate in a succeeding state
    SucceedsIf(SpecExp),
}

/// Specification directive with span.
pub type Condition = Spanned<Condition_>;

/// An invariant over a resource.
#[derive(PartialEq, Debug, Clone)]
pub struct Invariant_ {
    /// A free string (for now) which specifies the function of this invariant.
    pub modifier: String,

    /// An optional synthetic variable to which the below expression is assigned to.
    pub target: Option<String>,

    /// A specification expression.
    pub exp: SpecExp,
}

/// Invariant with span.
pub type Invariant = Spanned<Invariant_>;

/// A synthetic variable definition.
#[derive(PartialEq, Debug, Clone)]
pub struct SyntheticDefinition_ {
    pub name: Identifier,
    pub type_: Type,
}

/// Synthetic with span.
pub type SyntheticDefinition = Spanned<SyntheticDefinition_>;
