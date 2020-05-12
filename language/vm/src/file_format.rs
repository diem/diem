// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Binary format for transactions and modules.
//!
//! This module provides a simple Rust abstraction over the binary format. That is the format of
//! modules stored on chain or the format of the code section of a transaction.
//!
//! `file_format_common.rs` provides the constant values for entities in the binary format.
//! (*The binary format is evolving so please come back here in time to check evolutions.*)
//!
//! Overall the binary format is structured in a number of sections:
//! - **Header**: this must start at offset 0 in the binary. It contains a blob that starts every
//! Libra binary, followed by the version of the VM used to compile the code, and last is the
//! number of tables present in this binary.
//! - **Table Specification**: it's a number of tuple of the form
//! `(table type, starting_offset, byte_count)`. The number of entries is specified in the
//! header (last entry in header). There can only be a single entry per table type. The
//! `starting offset` is from the beginning of the binary. Tables must cover the entire size of
//! the binary blob and cannot overlap.
//! - **Table Content**: the serialized form of the specific entries in the table. Those roughly
//! map to the structs defined in this module. Entries in each table must be unique.
//!
//! We have two formats: one for modules here represented by `CompiledModule`, another
//! for transaction scripts which is `CompiledScript`. Building those tables and passing them
//! to the serializer (`serializer.rs`) generates a binary of the form described. Vectors in
//! those structs translate to tables and table specifications.

use crate::{
    access::ModuleAccess, check_bounds::BoundsChecker, errors::VMResult, internals::ModuleIndex,
    IndexKind, SignatureTokenKind,
};
use libra_types::{
    account_address::AccountAddress,
    vm_error::{StatusCode, VMStatus},
};
use mirai_annotations::*;
use move_core_types::{
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use num_variants::NumVariants;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{collection::vec, prelude::*, strategy::BoxedStrategy};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use ref_cast::RefCast;

/// Generic index into one of the tables in the binary format.
pub type TableIndex = u16;

macro_rules! define_index {
    {
        name: $name: ident,
        kind: $kind: ident,
        doc: $comment: literal,
    } => {
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
        #[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
        #[doc=$comment]
        pub struct $name(pub TableIndex);

        /// Returns an instance of the given `Index`.
        impl $name {
            pub fn new(idx: TableIndex) -> Self {
                Self(idx)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl ModuleIndex for $name {
            const KIND: IndexKind = IndexKind::$kind;

            #[inline]
            fn into_index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

define_index! {
    name: ModuleHandleIndex,
    kind: ModuleHandle,
    doc: "Index into the `ModuleHandle` table.",
}
define_index! {
    name: StructHandleIndex,
    kind: StructHandle,
    doc: "Index into the `StructHandle` table.",
}
define_index! {
    name: FunctionHandleIndex,
    kind: FunctionHandle,
    doc: "Index into the `FunctionHandle` table.",
}
define_index! {
    name: FieldHandleIndex,
    kind: FieldHandle,
    doc: "Index into the `FieldHandle` table.",
}
define_index! {
    name: StructDefInstantiationIndex,
    kind: StructDefInstantiation,
    doc: "Index into the `StructInstantiation` table.",
}
define_index! {
    name: FunctionInstantiationIndex,
    kind: FunctionInstantiation,
    doc: "Index into the `FunctionInstantiation` table.",
}
define_index! {
    name: FieldInstantiationIndex,
    kind: FieldInstantiation,
    doc: "Index into the `FieldInstantiation` table.",
}
define_index! {
    name: IdentifierIndex,
    kind: Identifier,
    doc: "Index into the `Identifier` table.",
}
define_index! {
    name: AddressIdentifierIndex,
    kind: AddressIdentifier,
    doc: "Index into the `AddressIdentifier` table.",
}
define_index! {
    name: ConstantPoolIndex,
    kind: ConstantPool,
    doc: "Index into the `ConstantPool` table.",
}
define_index! {
    name: SignatureIndex,
    kind: Signature,
    doc: "Index into the `Signature` table.",
}
define_index! {
    name: StructDefinitionIndex,
    kind: StructDefinition,
    doc: "Index into the `StructDefinition` table.",
}
define_index! {
    name: FunctionDefinitionIndex,
    kind: FunctionDefinition,
    doc: "Index into the `FunctionDefinition` table.",
}

/// Index of a local variable in a function.
///
/// Bytecodes that operate on locals carry indexes to the locals of a function.
pub type LocalIndex = u8;
/// Max number of fields in a `StructDefinition`.
pub type MemberCount = u16;
/// Index into the code stream for a jump. The offset is relative to the beginning of
/// the instruction stream.
pub type CodeOffset = u16;

/// The pool of identifiers.
pub type IdentifierPool = Vec<Identifier>;
/// The pool of address identifiers (addresses used in ModuleHandles/ModuleIds).
/// Does not include runtime values. Those are placed in the `ConstantPool`
pub type AddressIdentifierPool = Vec<AccountAddress>;
/// The pool of `Constant` values
pub type ConstantPool = Vec<Constant>;
/// The pool of `TypeSignature` instances. Those are system and user types used and
/// their composition (e.g. &U64).
pub type TypeSignaturePool = Vec<TypeSignature>;
/// The pool of `Signature` instances. Every function definition must define the set of
/// locals used and their types.
pub type SignaturePool = Vec<Signature>;

// TODO: "<SELF>" only passes the validator for identifiers because it is special cased. Whenever
// "<SELF>" is removed, so should the special case in identifier.rs.
pub fn self_module_name() -> &'static IdentStr {
    IdentStr::ref_cast("<SELF>")
}

/// Index 0 into the LocalsSignaturePool, which is guaranteed to be an empty list.
/// Used to represent function/struct instantiation with no type arguments -- effectively
/// non-generic functions and structs.
pub const NO_TYPE_ARGUMENTS: SignatureIndex = SignatureIndex(0);

// HANDLES:
// Handles are structs that accompany opcodes that need references: a type reference,
// or a function reference (a field reference being available only within the module that
// defines the field can be a definition).
// Handles refer to both internal and external "entities" and are embedded as indexes
// in the instruction stream.
// Handles define resolution. Resolution is assumed to be by (name, signature)

/// A `ModuleHandle` is a reference to a MOVE module. It is composed by an `address` and a `name`.
///
/// A `ModuleHandle` uniquely identifies a code resource in the blockchain.
/// The `address` is a reference to the account that holds the code and the `name` is used as a
/// key in order to load the module.
///
/// Modules live in the *code* namespace of an LibraAccount.
///
/// Modules introduce a scope made of all types defined in the module and all functions.
/// Type definitions (fields) are private to the module. Outside the module a
/// Type is an opaque handle.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct ModuleHandle {
    /// Index into the `AddressIdentifierIndex`. Identifies module-holding account's address.
    pub address: AddressIdentifierIndex,
    /// The name of the module published in the code section for the account in `address`.
    pub name: IdentifierIndex,
}

/// A `StructHandle` is a reference to a user defined type. It is composed by a `ModuleHandle`
/// and the name of the type within that module.
///
/// A type in a module is uniquely identified by its name and as such the name is enough
/// to perform resolution.
///
/// The `StructHandle` is polymorphic: it can have type parameters in its fields and carries the
/// kind constraints for these type parameters (empty list for non-generic structs). It also
/// carries the kind (resource/copyable) of the struct itself so that the verifier can check
/// resource semantic without having to load the referenced type.
///
/// At link time kind checking is performed and an error is reported if there is a
/// mismatch with the definition.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct StructHandle {
    /// The module that defines the type.
    pub module: ModuleHandleIndex,
    /// The name of the type.
    pub name: IdentifierIndex,
    /// There are two ways for a type to have the Kind resource
    /// 1) If it has a type argument of resource
    /// 2) If it was declared as a resource
    /// These "declared" resources are referred to as *nominal resources*
    ///
    /// If `is_nominal_resource` is true, it is a *nominal resource*
    pub is_nominal_resource: bool,
    /// The type formals (identified by their index into the vec) and their kind constraints
    pub type_parameters: Vec<Kind>,
}

/// A `FunctionHandle` is a reference to a function. It is composed by a
/// `ModuleHandle` and the name and signature of that function within the module.
///
/// A function within a module is uniquely identified by its name. No overloading is allowed
/// and the verifier enforces that property. The signature of the function is used at link time to
/// ensure the function reference is valid and it is also used by the verifier to type check
/// function calls.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(params = "usize"))]
pub struct FunctionHandle {
    /// The module that defines the function.
    pub module: ModuleHandleIndex,
    /// The name of the function.
    pub name: IdentifierIndex,
    /// The list of arguments to the function.
    pub parameters: SignatureIndex,
    /// The list of return types.
    pub return_: SignatureIndex,
    /// The type formals (identified by their index into the vec) and their kind constraints
    pub type_parameters: Vec<Kind>,
}

/// A field access info (owner type and offset)
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct FieldHandle {
    pub owner: StructDefinitionIndex,
    pub field: MemberCount,
}

// DEFINITIONS:
// Definitions are the module code. So the set of types and functions in the module.

/// `StructFieldInformation` indicates whether a struct is native or has user-specified fields
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub enum StructFieldInformation {
    Native,
    Declared(Vec<FieldDefinition>),
}

//
// Instantiations
//
// Instantiations point to a generic handle and its instantiation.
// The instantiation can be partial.
// So, for example, `S<T, W>`, `S<u8, bool>`, `S<T, u8>`, `S<X<T>, address>` are all
// `StructInstantiation`s

/// A complete or partial instantiation of a generic struct
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct StructDefInstantiation {
    pub def: StructDefinitionIndex,
    pub type_parameters: SignatureIndex,
}

/// A complete or partial instantiation of a function
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct FunctionInstantiation {
    pub handle: FunctionHandleIndex,
    pub type_parameters: SignatureIndex,
}

/// A complete or partial instantiation of a field (or the type of it).
///
/// A `FieldInstantiation` points to a generic `FieldHandle` and the instantiation
/// of the owner type.
/// E.g. for `S<u8, bool>.f` where `f` is a field of any type, `instantiation`
/// would be `[u8, boo]`
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct FieldInstantiation {
    pub handle: FieldHandleIndex,
    pub type_parameters: SignatureIndex,
}

/// A `StructDefinition` is a type definition. It either indicates it is native or
// defines all the user-specified fields declared on the type.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct StructDefinition {
    /// The `StructHandle` for this `StructDefinition`. This has the name and the resource flag
    /// for the type.
    pub struct_handle: StructHandleIndex,
    /// Contains either
    /// - Information indicating the struct is native and has no accessible fields
    /// - Information indicating the number of fields and the start `FieldDefinition`s
    pub field_information: StructFieldInformation,
}

impl StructDefinition {
    pub fn declared_field_count(&self) -> Result<MemberCount, VMStatus> {
        match &self.field_information {
            // TODO we might want a more informative error here
            StructFieldInformation::Native => Err(VMStatus::new(StatusCode::LINKER_ERROR)
                .with_message("Looking for field in native structure".to_string())),
            StructFieldInformation::Declared(fields) => Ok(fields.len() as u16),
        }
    }
}
/// A `FieldDefinition` is the definition of a field: its name and the field type.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct FieldDefinition {
    /// The name of the field.
    pub name: IdentifierIndex,
    /// The type of the field.
    pub signature: TypeSignature,
}

/// A `FunctionDefinition` is the implementation of a function. It defines
/// the *prototype* of the function and the function body.

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(params = "usize"))]
pub struct FunctionDefinition {
    /// The prototype of the function (module, name, signature).
    pub function: FunctionHandleIndex,
    /// Flag to indicate if this function is public.
    pub is_public: bool,
    /// List of nominal resources (declared in this module) that the procedure might access
    /// Either through: BorrowGlobal, MoveFrom, or transitively through another procedure
    /// This list of acquires grants the borrow checker the ability to statically verify the safety
    /// of references into global storage
    ///
    /// Not in the signature as it is not needed outside of the declaring module
    ///
    /// Note, there is no SignatureIndex with each struct definition index, as global
    /// resources cannot currently take type arguments
    pub acquires_global_resources: Vec<StructDefinitionIndex>,
    /// Code for this function.
    #[cfg_attr(
        any(test, feature = "fuzzing"),
        proptest(strategy = "any_with::<CodeUnit>(params).prop_map(Some)")
    )]
    pub code: Option<CodeUnit>,
}

impl FunctionDefinition {
    /// Returns whether the FunctionDefinition is public.
    pub fn is_public(&self) -> bool {
        self.is_public
    }
    /// Returns whether the FunctionDefinition is native.
    pub fn is_native(&self) -> bool {
        self.code.is_none()
    }

    /// Function can be invoked outside of its declaring module.
    pub const PUBLIC: u8 = 0x1;
    /// A native function implemented in Rust.
    pub const NATIVE: u8 = 0x2;
}

// Signature
// A signature can be for a type (field, local) or for a function - return type: (arguments).
// They both go into the signature table so there is a marker that tags the signature.
// Signature usually don't carry a size and you have to read them to get to the end.

/// A type definition. `SignatureToken` allows the definition of the set of known types and their
/// composition.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
pub struct TypeSignature(pub SignatureToken);

// TODO: remove at some point or move it in the front end (language/compiler)
/// A `FunctionSignature` in internally used to create a unique representation of the overall
/// signature as need. Consider deprecated...
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(params = "usize"))]
pub struct FunctionSignature {
    /// The list of return types.
    #[cfg_attr(
        any(test, feature = "fuzzing"),
        proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")
    )]
    pub return_: Vec<SignatureToken>,
    /// The list of arguments to the function.
    #[cfg_attr(
        any(test, feature = "fuzzing"),
        proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")
    )]
    pub parameters: Vec<SignatureToken>,
    /// The type formals (identified by their index into the vec) and their kind constraints
    pub type_parameters: Vec<Kind>,
}

/// A `Signature` is the list of locals used by a function.
///
/// Locals include the arguments to the function from position `0` to argument `count - 1`.
/// The remaining elements are the type of each local.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(params = "usize"))]
pub struct Signature(
    #[cfg_attr(
        any(test, feature = "fuzzing"),
        proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")
    )]
    pub Vec<SignatureToken>,
);

impl Signature {
    /// Length of the `Signature`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the function has no locals (both arguments or locals).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Type parameters are encoded as indices. This index can also be used to lookup the kind of a
/// type parameter in the `FunctionHandle` and `StructHandle`.
pub type TypeParameterIndex = u16;

/// A `Kind` classifies types into sets with rules each set must follow.
///
/// Currently there are three kinds in Move: `All`, `Resource` and `Copyable`.
#[derive(Debug, Clone, Eq, Copy, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub enum Kind {
    /// Represents the super set of all types. The type might actually be a `Resource` or
    /// `Copyable` A type might be in this set if it is not known to be a `Resource` or
    /// `Copyable`
    ///   - This occurs when there is a type parameter with this kind as a constraint
    All,
    /// `Resource` types must follow move semantics and various resource safety rules, namely:
    /// - `Resource` values cannot be copied
    /// - `Resource` values cannot be popped, i.e. they must be used
    Resource,
    /// `Copyable` types do not need to follow the `Resource` rules.
    /// - `Copyable` values can be copied
    /// - `Copyable` values can be popped
    Copyable,
}

impl Kind {
    /// Checks if the given kind is a sub-kind of another.
    #[inline]
    pub fn is_sub_kind_of(self, k: Kind) -> bool {
        use Kind::*;

        matches!((self, k), (_, All) | (Resource, Resource) | (Copyable, Copyable))
    }

    /// Helper function to determine the kind of a struct instance by taking the kind of a type
    /// argument and join it with the existing partial result.
    pub fn join(self, other: Kind) -> Kind {
        match (self, other) {
            (Kind::All, _) | (_, Kind::All) => Kind::All,
            (Kind::Resource, _) | (_, Kind::Resource) => Kind::Resource,
            (Kind::Copyable, Kind::Copyable) => Kind::Copyable,
        }
    }
}

/// A `SignatureToken` is a type declaration for a location.
///
/// Any location in the system has a TypeSignature.
/// A TypeSignature is also used in composed signatures.
///
/// A SignatureToken can express more types than the VM can handle safely, and correctness is
/// enforced by the verifier.
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SignatureToken {
    /// Boolean, `true` or `false`.
    Bool,
    /// Unsigned integers, 8 bits length.
    U8,
    /// Unsigned integers, 64 bits length.
    U64,
    /// Unsigned integers, 128 bits length.
    U128,
    /// Address, a 16 bytes immutable type.
    Address,
    /// Vector
    Vector(Box<SignatureToken>),
    /// MOVE user type, resource or copyable
    Struct(StructHandleIndex),
    StructInstantiation(StructHandleIndex, Vec<SignatureToken>),
    /// Reference to a type.
    Reference(Box<SignatureToken>),
    /// Mutable reference to a type.
    MutableReference(Box<SignatureToken>),
    /// Type parameter.
    TypeParameter(TypeParameterIndex),
}

/// An iterator to help traverse the `SignatureToken` in a non-recursive fashion to avoid
/// overflowing the stack.
///
/// Traversal order: root -> left -> right
pub struct SignatureTokenPreorderTraversalIter<'a> {
    stack: Vec<&'a SignatureToken>,
}

impl<'a> Iterator for SignatureTokenPreorderTraversalIter<'a> {
    type Item = &'a SignatureToken;

    fn next(&mut self) -> Option<Self::Item> {
        use SignatureToken::*;

        match self.stack.pop() {
            Some(tok) => {
                match tok {
                    Reference(inner_tok) | MutableReference(inner_tok) | Vector(inner_tok) => {
                        self.stack.push(inner_tok)
                    }

                    StructInstantiation(_, inner_toks) => {
                        self.stack.extend(inner_toks.iter().rev())
                    }

                    Bool | Address | U8 | U64 | U128 | Struct(_) | TypeParameter(_) => (),
                }
                Some(tok)
            }
            None => None,
        }
    }
}

/// `Arbitrary` for `SignatureToken` cannot be derived automatically as it's a recursive type.
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for SignatureToken {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
        use SignatureToken::*;

        let leaf = prop_oneof![
            Just(Bool),
            Just(U8),
            Just(U64),
            Just(U128),
            Just(Address),
            any::<StructHandleIndex>().prop_map(Struct),
            any::<TypeParameterIndex>().prop_map(TypeParameter),
        ];
        leaf.prop_recursive(
            8,  // levels deep
            16, // max size
            1,  // items per collection
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|token| Vector(Box::new(token))),
                    inner.clone().prop_map(|token| Reference(Box::new(token))),
                    inner.prop_map(|token| MutableReference(Box::new(token))),
                ]
            },
        )
        .boxed()
    }
}

impl std::fmt::Debug for SignatureToken {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SignatureToken::Bool => write!(f, "Bool"),
            SignatureToken::U8 => write!(f, "U8"),
            SignatureToken::U64 => write!(f, "U64"),
            SignatureToken::U128 => write!(f, "U128"),
            SignatureToken::Address => write!(f, "Address"),
            SignatureToken::Vector(boxed) => write!(f, "Vector({:?})", boxed),
            SignatureToken::Struct(idx) => write!(f, "Struct({:?})", idx),
            SignatureToken::StructInstantiation(idx, types) => {
                write!(f, "StructInstantiation({:?}, {:?})", idx, types)
            }
            SignatureToken::Reference(boxed) => write!(f, "Reference({:?})", boxed),
            SignatureToken::MutableReference(boxed) => write!(f, "MutableReference({:?})", boxed),
            SignatureToken::TypeParameter(idx) => write!(f, "TypeParameter({:?})", idx),
        }
    }
}

impl SignatureToken {
    /// Returns the "value kind" for the `SignatureToken`
    #[inline]
    pub fn signature_token_kind(&self) -> SignatureTokenKind {
        // TODO: SignatureTokenKind is out-dated. fix/update/remove SignatureTokenKind and see if
        // this function needs to be cleaned up
        use SignatureToken::*;

        match self {
            Reference(_) => SignatureTokenKind::Reference,
            MutableReference(_) => SignatureTokenKind::MutableReference,
            Bool
            | U8
            | U64
            | U128
            | Address
            | Struct(_)
            | StructInstantiation(_, _)
            | Vector(_) => SignatureTokenKind::Value,
            // TODO: This is a temporary hack to please the verifier. SignatureTokenKind will soon
            // be completely removed. `SignatureTokenView::kind()` should be used instead.
            TypeParameter(_) => SignatureTokenKind::Value,
        }
    }

    /// Returns `true` if the `SignatureToken` is a primitive type.
    pub fn is_primitive(&self) -> bool {
        use SignatureToken::*;
        match self {
            Bool | U8 | U64 | U128 | Address => true,
            Struct(_)
            | StructInstantiation(_, _)
            | Reference(_)
            | Vector(_)
            | MutableReference(_)
            | TypeParameter(_) => false,
        }
    }

    // Returns `true` if the `SignatureToken` is an integer type.
    pub fn is_integer(&self) -> bool {
        use SignatureToken::*;
        match self {
            U8 | U64 | U128 => true,
            Bool
            | Address
            | Vector(_)
            | Struct(_)
            | StructInstantiation(_, _)
            | Reference(_)
            | MutableReference(_)
            | TypeParameter(_) => false,
        }
    }

    /// Returns true if the `SignatureToken` is any kind of reference (mutable and immutable).
    pub fn is_reference(&self) -> bool {
        use SignatureToken::*;

        matches!(self, Reference(_) | MutableReference(_))
    }

    /// Returns true if the `SignatureToken` is a mutable reference.
    pub fn is_mutable_reference(&self) -> bool {
        use SignatureToken::*;

        matches!(self, MutableReference(_))
    }

    /// Set the index to this one. Useful for random testing.
    ///
    /// Panics if this token doesn't contain a struct handle.
    pub fn debug_set_sh_idx(&mut self, sh_idx: StructHandleIndex) {
        match self {
            SignatureToken::Struct(ref mut wrapped) => *wrapped = sh_idx,
            SignatureToken::Reference(ref mut token)
            | SignatureToken::MutableReference(ref mut token) => token.debug_set_sh_idx(sh_idx),
            other => panic!(
                "debug_set_sh_idx (to {}) called for non-struct token {:?}",
                sh_idx, other
            ),
        }
    }

    pub fn preorder_traversal(&self) -> SignatureTokenPreorderTraversalIter<'_> {
        SignatureTokenPreorderTraversalIter { stack: vec![self] }
    }
}

/// A `Constant` is a serialized value along with it's type. That type will be deserialized by the
/// loader/evauluator
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Constant {
    pub type_: SignatureToken,
    pub data: Vec<u8>,
}

/// A `CodeUnit` is the body of a function. It has the function header and the instruction stream.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(params = "usize"))]
pub struct CodeUnit {
    /// List of locals type. All locals are typed.
    pub locals: SignatureIndex,
    /// Code stream, function body.
    #[cfg_attr(
        any(test, feature = "fuzzing"),
        proptest(strategy = "vec(any::<Bytecode>(), 0..=params)")
    )]
    pub code: Vec<Bytecode>,
}

/// `Bytecode` is a VM instruction of variable size. The type of the bytecode (opcode) defines
/// the size of the bytecode.
///
/// Bytecodes operate on a stack machine and each bytecode has side effect on the stack and the
/// instruction stream.
#[derive(Clone, Hash, Eq, NumVariants, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
#[cfg_attr(any(test, feature = "fuzzing"), proptest(no_params))]
#[num_variants = "NUM_INSTRUCTIONS"]
pub enum Bytecode {
    /// Pop and discard the value at the top of the stack.
    /// The value on the stack must be an copyable type.
    ///
    /// Stack transition:
    ///
    /// ```..., value -> ...```
    Pop,
    /// Return from function, possibly with values according to the return types in the
    /// function signature. The returned values are pushed on the stack.
    /// The function signature of the function being executed defines the semantic of
    /// the Ret opcode.
    ///
    /// Stack transition:
    ///
    /// ```..., arg_val(1), ..., arg_val(n) -> ..., return_val(1), ..., return_val(n)```
    Ret,
    /// Branch to the instruction at position `CodeOffset` if the value at the top of the stack
    /// is true. Code offsets are relative to the start of the instruction stream.
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value -> ...```
    BrTrue(CodeOffset),
    /// Branch to the instruction at position `CodeOffset` if the value at the top of the stack
    /// is false. Code offsets are relative to the start of the instruction stream.
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value -> ...```
    BrFalse(CodeOffset),
    /// Branch unconditionally to the instruction at position `CodeOffset`. Code offsets are
    /// relative to the start of the instruction stream.
    ///
    /// Stack transition: none
    Branch(CodeOffset),
    /// Push a U8 constant onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u8_value```
    LdU8(u8),
    /// Push a U64 constant onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    LdU64(u64),
    /// Push a U128 constant onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u128_value```
    LdU128(u128),
    /// Convert the value at the top of the stack into u8.
    ///
    /// Stack transition:
    ///
    /// ```..., integer_value -> ..., u8_value```
    CastU8,
    /// Convert the value at the top of the stack into u64.
    ///
    /// Stack transition:
    ///
    /// ```..., integer_value -> ..., u8_value```
    CastU64,
    /// Convert the value at the top of the stack into u128.
    ///
    /// Stack transition:
    ///
    /// ```..., integer_value -> ..., u128_value```
    CastU128,
    /// Push a `Constant` onto the stack. The value is loaded and deserialized (according to it's
    /// type) from the the `ConstantPool` via `ConstantPoolIndex`
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., value```
    LdConst(ConstantPoolIndex),
    /// Push `true` onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., true```
    LdTrue,
    /// Push `false` onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., false```
    LdFalse,
    /// Push the local identified by `LocalIndex` onto the stack. The value is copied and the
    /// local is still safe to use.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., value```
    CopyLoc(LocalIndex),
    /// Push the local identified by `LocalIndex` onto the stack. The local is moved and it is
    /// invalid to use from that point on, unless a store operation writes to the local before
    /// any read to that local.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., value```
    MoveLoc(LocalIndex),
    /// Pop value from the top of the stack and store it into the function locals at
    /// position `LocalIndex`.
    ///
    /// Stack transition:
    ///
    /// ```..., value -> ...```
    StLoc(LocalIndex),
    /// Call a function. The stack has the arguments pushed first to last.
    /// The arguments are consumed and pushed to the locals of the function.
    /// Return values are pushed on the stack and available to the caller.
    ///
    /// Stack transition:
    ///
    /// ```..., arg(1), arg(2), ...,  arg(n) -> ..., return_value(1), return_value(2), ...,
    /// return_value(k)```
    Call(FunctionHandleIndex),
    CallGeneric(FunctionInstantiationIndex),
    /// Create an instance of the type specified via `StructHandleIndex` and push it on the stack.
    /// The values of the fields of the struct, in the order they appear in the struct declaration,
    /// must be pushed on the stack. All fields must be provided.
    ///
    /// A Pack instruction must fully initialize an instance.
    ///
    /// Stack transition:
    ///
    /// ```..., field(1)_value, field(2)_value, ..., field(n)_value -> ..., instance_value```
    Pack(StructDefinitionIndex),
    PackGeneric(StructDefInstantiationIndex),
    /// Destroy an instance of a type and push the values bound to each field on the
    /// stack.
    ///
    /// The values of the fields of the instance appear on the stack in the order defined
    /// in the struct definition.
    ///
    /// This order makes Unpack<T> the inverse of Pack<T>. So `Unpack<T>; Pack<T>` is the identity
    /// for struct T.
    ///
    /// Stack transition:
    ///
    /// ```..., instance_value -> ..., field(1)_value, field(2)_value, ..., field(n)_value```
    Unpack(StructDefinitionIndex),
    UnpackGeneric(StructDefInstantiationIndex),
    /// Read a reference. The reference is on the stack, it is consumed and the value read is
    /// pushed on the stack.
    ///
    /// Reading a reference performs a copy of the value referenced. As such
    /// ReadRef cannot be used on a reference to a Resource.
    ///
    /// Stack transition:
    ///
    /// ```..., reference_value -> ..., value```
    ReadRef,
    /// Write to a reference. The reference and the value are on the stack and are consumed.
    ///
    ///
    /// The reference must be to an copyable type because Resources cannot be overwritten.
    ///
    /// Stack transition:
    ///
    /// ```..., value, reference_value -> ...```
    WriteRef,
    /// Convert a mutable reference to an immutable reference.
    ///
    /// Stack transition:
    ///
    /// ```..., reference_value -> ..., reference_value```
    FreezeRef,
    /// Load a mutable reference to a local identified by LocalIndex.
    ///
    /// The local must not be a reference.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., reference```
    MutBorrowLoc(LocalIndex),
    /// Load an immutable reference to a local identified by LocalIndex.
    ///
    /// The local must not be a reference.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., reference```
    ImmBorrowLoc(LocalIndex),
    /// Load a mutable reference to a field identified by `FieldHandleIndex`.
    /// The top of the stack must be a mutable reference to a type that contains the field
    /// definition.
    ///
    /// Stack transition:
    ///
    /// ```..., reference -> ..., field_reference```
    MutBorrowField(FieldHandleIndex),
    /// Load a mutable reference to a field identified by `FieldInstantiationIndex`.
    /// The top of the stack must be a mutable reference to a type that contains the field
    /// definition.
    ///
    /// Stack transition:
    ///
    /// ```..., reference -> ..., field_reference```
    MutBorrowFieldGeneric(FieldInstantiationIndex),
    /// Load an immutable reference to a field identified by `FieldHandleIndex`.
    /// The top of the stack must be a reference to a type that contains the field definition.
    ///
    /// Stack transition:
    ///
    /// ```..., reference -> ..., field_reference```
    ImmBorrowField(FieldHandleIndex),
    /// Load an immutable reference to a field identified by `FieldInstantiationIndex`.
    /// The top of the stack must be a reference to a type that contains the field definition.
    ///
    /// Stack transition:
    ///
    /// ```..., reference -> ..., field_reference```
    ImmBorrowFieldGeneric(FieldInstantiationIndex),
    /// Return a mutable reference to an instance of type `StructDefinitionIndex` published at the
    /// address passed as argument. Abort execution if such an object does not exist or if a
    /// reference has already been handed out.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., reference_value```
    MutBorrowGlobal(StructDefinitionIndex),
    MutBorrowGlobalGeneric(StructDefInstantiationIndex),
    /// Return an immutable reference to an instance of type `StructDefinitionIndex` published at
    /// the address passed as argument. Abort execution if such an object does not exist or if a
    /// reference has already been handed out.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., reference_value```
    ImmBorrowGlobal(StructDefinitionIndex),
    ImmBorrowGlobalGeneric(StructDefInstantiationIndex),
    /// Add the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of overflow.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Add,
    /// Subtract the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of underflow.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Sub,
    /// Multiply the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of overflow.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Mul,
    /// Perform a modulo operation on the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Mod,
    /// Divide the 2 u64 at the top of the stack and pushes the result on the stack.
    /// The operation aborts the transaction in case of "divide by 0".
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Div,
    /// Bitwise OR the 2 u64 at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    BitOr,
    /// Bitwise AND the 2 u64 at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    BitAnd,
    /// Bitwise XOR the 2 u64 at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Xor,
    /// Logical OR the 2 bool at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value(1), bool_value(2) -> ..., bool_value```
    Or,
    /// Logical AND the 2 bool at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value(1), bool_value(2) -> ..., bool_value```
    And,
    /// Logical NOT the bool at the top of the stack and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value -> ..., bool_value```
    Not,
    /// Compare for equality the 2 value at the top of the stack and pushes the
    /// result on the stack.
    /// The values on the stack cannot be resources or they will be consumed and so destroyed.
    ///
    /// Stack transition:
    ///
    /// ```..., value(1), value(2) -> ..., bool_value```
    Eq,
    /// Compare for inequality the 2 value at the top of the stack and pushes the
    /// result on the stack.
    /// The values on the stack cannot be resources or they will be consumed and so destroyed.
    ///
    /// Stack transition:
    ///
    /// ```..., value(1), value(2) -> ..., bool_value```
    Neq,
    /// Perform a "less than" operation of the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., bool_value```
    Lt,
    /// Perform a "greater than" operation of the 2 u64 at the top of the stack and pushes the
    /// result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., bool_value```
    Gt,
    /// Perform a "less than or equal" operation of the 2 u64 at the top of the stack and pushes
    /// the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., bool_value```
    Le,
    /// Perform a "greater than or equal" than operation of the 2 u64 at the top of the stack
    /// and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., bool_value```
    Ge,
    /// Abort execution with errorcode
    ///
    ///
    /// Stack transition:
    ///
    /// ```..., errorcode -> ...```
    Abort,
    /// Get the sender address from the transaction and pushes it on the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., address_value```
    GetTxnSenderAddress,
    /// Returns whether or not a given address has an object of type StructDefinitionIndex
    /// published already
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., bool_value```
    Exists(StructDefinitionIndex),
    ExistsGeneric(StructDefInstantiationIndex),
    /// Move the instance of type StructDefinitionIndex, at the address at the top of the stack.
    /// Abort execution if such an object does not exist.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., value```
    MoveFrom(StructDefinitionIndex),
    MoveFromGeneric(StructDefInstantiationIndex),
    /// Move the instance at the top of the stack to the address of the sender.
    /// Abort execution if an object of type StructDefinitionIndex already exists in address.
    ///
    /// Stack transition:
    ///
    /// ```..., value -> ...```
    MoveToSender(StructDefinitionIndex),
    MoveToSenderGeneric(StructDefInstantiationIndex),
    /// Shift the (second top value) left (top value) bits and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Shl,
    /// Shift the (second top value) right (top value) bits and pushes the result on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., u64_value(1), u64_value(2) -> ..., u64_value```
    Shr,
    /// No operation.
    ///
    /// Stack transition: none
    Nop,
}

pub const NUMBER_OF_NATIVE_FUNCTIONS: usize = 17;

impl ::std::fmt::Debug for Bytecode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Bytecode::Pop => write!(f, "Pop"),
            Bytecode::Ret => write!(f, "Ret"),
            Bytecode::BrTrue(a) => write!(f, "BrTrue({})", a),
            Bytecode::BrFalse(a) => write!(f, "BrFalse({})", a),
            Bytecode::Branch(a) => write!(f, "Branch({})", a),
            Bytecode::LdU8(a) => write!(f, "LdU8({})", a),
            Bytecode::LdU64(a) => write!(f, "LdU64({})", a),
            Bytecode::LdU128(a) => write!(f, "LdU128({})", a),
            Bytecode::CastU8 => write!(f, "CastU8"),
            Bytecode::CastU64 => write!(f, "CastU64"),
            Bytecode::CastU128 => write!(f, "CastU128"),
            Bytecode::LdConst(a) => write!(f, "LdConst({})", a),
            Bytecode::LdTrue => write!(f, "LdTrue"),
            Bytecode::LdFalse => write!(f, "LdFalse"),
            Bytecode::CopyLoc(a) => write!(f, "CopyLoc({})", a),
            Bytecode::MoveLoc(a) => write!(f, "MoveLoc({})", a),
            Bytecode::StLoc(a) => write!(f, "StLoc({})", a),
            Bytecode::Call(a) => write!(f, "Call({})", a),
            Bytecode::CallGeneric(a) => write!(f, "CallGeneric({})", a),
            Bytecode::Pack(a) => write!(f, "Pack({})", a),
            Bytecode::PackGeneric(a) => write!(f, "PackGeneric({})", a),
            Bytecode::Unpack(a) => write!(f, "Unpack({})", a),
            Bytecode::UnpackGeneric(a) => write!(f, "UnpackGeneric({})", a),
            Bytecode::ReadRef => write!(f, "ReadRef"),
            Bytecode::WriteRef => write!(f, "WriteRef"),
            Bytecode::FreezeRef => write!(f, "FreezeRef"),
            Bytecode::MutBorrowLoc(a) => write!(f, "MutBorrowLoc({})", a),
            Bytecode::ImmBorrowLoc(a) => write!(f, "ImmBorrowLoc({})", a),
            Bytecode::MutBorrowField(a) => write!(f, "MutBorrowField({:?})", a),
            Bytecode::MutBorrowFieldGeneric(a) => write!(f, "MutBorrowFieldGeneric({:?})", a),
            Bytecode::ImmBorrowField(a) => write!(f, "ImmBorrowField({:?})", a),
            Bytecode::ImmBorrowFieldGeneric(a) => write!(f, "ImmBorrowFieldGeneric({:?})", a),
            Bytecode::MutBorrowGlobal(a) => write!(f, "MutBorrowGlobal({:?})", a),
            Bytecode::MutBorrowGlobalGeneric(a) => write!(f, "MutBorrowGlobalGeneric({:?})", a),
            Bytecode::ImmBorrowGlobal(a) => write!(f, "ImmBorrowGlobal({:?})", a),
            Bytecode::ImmBorrowGlobalGeneric(a) => write!(f, "ImmBorrowGlobalGeneric({:?})", a),
            Bytecode::Add => write!(f, "Add"),
            Bytecode::Sub => write!(f, "Sub"),
            Bytecode::Mul => write!(f, "Mul"),
            Bytecode::Mod => write!(f, "Mod"),
            Bytecode::Div => write!(f, "Div"),
            Bytecode::BitOr => write!(f, "BitOr"),
            Bytecode::BitAnd => write!(f, "BitAnd"),
            Bytecode::Xor => write!(f, "Xor"),
            Bytecode::Shl => write!(f, "Shl"),
            Bytecode::Shr => write!(f, "Shr"),
            Bytecode::Or => write!(f, "Or"),
            Bytecode::And => write!(f, "And"),
            Bytecode::Not => write!(f, "Not"),
            Bytecode::Eq => write!(f, "Eq"),
            Bytecode::Neq => write!(f, "Neq"),
            Bytecode::Lt => write!(f, "Lt"),
            Bytecode::Gt => write!(f, "Gt"),
            Bytecode::Le => write!(f, "Le"),
            Bytecode::Ge => write!(f, "Ge"),
            Bytecode::Abort => write!(f, "Abort"),
            Bytecode::GetTxnSenderAddress => write!(f, "GetTxnSenderAddress"),
            Bytecode::Exists(a) => write!(f, "Exists({:?})", a),
            Bytecode::ExistsGeneric(a) => write!(f, "ExistsGeneric({:?})", a),
            Bytecode::MoveFrom(a) => write!(f, "MoveFrom({:?})", a),
            Bytecode::MoveFromGeneric(a) => write!(f, "MoveFromGeneric({:?})", a),
            Bytecode::MoveToSender(a) => write!(f, "MoveToSender({:?})", a),
            Bytecode::MoveToSenderGeneric(a) => write!(f, "MoveToSenderGeneric({:?})", a),
            Bytecode::Nop => write!(f, "Nop"),
        }
    }
}

impl Bytecode {
    /// Return true if this bytecode instruction always branches
    pub fn is_unconditional_branch(&self) -> bool {
        matches!(self, Bytecode::Ret | Bytecode::Abort | Bytecode::Branch(_))
    }

    /// Return true if the branching behavior of this bytecode instruction depends on a runtime
    /// value
    pub fn is_conditional_branch(&self) -> bool {
        matches!(self, Bytecode::BrFalse(_) | Bytecode::BrTrue(_))
    }

    /// Returns true if this bytecode instruction is either a conditional or an unconditional branch
    pub fn is_branch(&self) -> bool {
        self.is_conditional_branch() || self.is_unconditional_branch()
    }

    /// Returns the offset that this bytecode instruction branches to, if any.
    /// Note that return and abort are branch instructions, but have no offset.
    pub fn offset(&self) -> Option<&CodeOffset> {
        match self {
            Bytecode::BrFalse(offset) | Bytecode::BrTrue(offset) | Bytecode::Branch(offset) => {
                Some(offset)
            }
            _ => None,
        }
    }

    /// Return the successor offsets of this bytecode instruction.
    pub fn get_successors(pc: CodeOffset, code: &[Bytecode]) -> Vec<CodeOffset> {
        checked_precondition!(
            // The program counter could be added to at most twice and must remain
            // within the bounds of the code.
            pc <= u16::max_value() - 2 && (pc as usize) < code.len(),
            "Program counter out of bounds"
        );
        let bytecode = &code[pc as usize];
        let mut v = vec![];

        if let Some(offset) = bytecode.offset() {
            v.push(*offset);
        }

        let next_pc = pc + 1;
        if next_pc >= code.len() as CodeOffset {
            return v;
        }

        if !bytecode.is_unconditional_branch() && !v.contains(&next_pc) {
            // avoid duplicates
            v.push(pc + 1);
        }

        // always give successors in ascending order
        if v.len() > 1 && v[0] > v[1] {
            v.swap(0, 1);
        }

        v
    }
}

// Note that this doesn't derive either `Arbitrary` or `Default` while `CompiledScriptMut` does.
// That's because a CompiledScript is guaranteed to be valid while a CompiledScriptMut isn't.
/// Contains the main function to execute and its dependencies.
///
/// A CompiledScript does not have definition tables because it can only have a `main(args)`.
/// A CompiledScript defines the constant pools (string, address, signatures, etc.), the handle
/// tables (external code references) and it has a `main` definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledScript(CompiledScriptMut);

/// A mutable version of `CompiledScript`. Converting to a `CompiledScript` requires this to pass
/// the bounds checker.
#[derive(Clone, Default, Eq, PartialEq, Debug)]
pub struct CompiledScriptMut {
    /// Handles to all modules referenced.
    pub module_handles: Vec<ModuleHandle>,
    /// Handles to external/imported types.
    pub struct_handles: Vec<StructHandle>,
    /// Handles to external/imported functions.
    pub function_handles: Vec<FunctionHandle>,

    /// Function instantiations.
    pub function_instantiations: Vec<FunctionInstantiation>,

    pub signatures: SignaturePool,

    /// All identifiers used in this transaction.
    pub identifiers: IdentifierPool,
    /// All address identifiers used in this transaction.
    pub address_identifiers: AddressIdentifierPool,
    /// Constant pool. The constant values used in the transaction.
    pub constant_pool: ConstantPool,

    pub type_parameters: Vec<Kind>,

    pub parameters: SignatureIndex,

    pub code: CodeUnit,
}

impl CompiledScript {
    /// Returns the index of `main` in case a script is converted to a module.
    pub const MAIN_INDEX: FunctionDefinitionIndex = FunctionDefinitionIndex(0);

    /// Returns a reference to the inner `CompiledScriptMut`.
    pub fn as_inner(&self) -> &CompiledScriptMut {
        &self.0
    }

    /// Converts this instance into the inner `CompiledScriptMut`. Converting back to a
    /// `CompiledScript` would require it to be verified again.
    pub fn into_inner(self) -> CompiledScriptMut {
        self.0
    }

    /// Converts a `CompiledScript` into a `CompiledModule` for code that wants a uniform view of
    /// both.
    ///
    /// If a `CompiledScript` has been bounds checked, the corresponding `CompiledModule` can be
    /// assumed to pass the bounds checker as well.
    #[allow(deprecated)]
    pub fn into_module(self) -> (ScriptConversionInfo, CompiledModule) {
        let (info, m) = self.0.into_module();
        (info, CompiledModule(m))
    }
}

pub struct ScriptConversionInfo {
    // If a dummy address was added.
    added_dummy_addr: bool,
    // If an empty signature was added.
    added_dummy_sig: bool,
    // If the <SELF> identifier was added.
    added_self_ident: bool,
    // If a self module handle was added.
    added_self_module_handle: bool,
}

impl CompiledScriptMut {
    /// Converts this instance into `CompiledScript` after verifying it for basic internal
    /// consistency. This includes bounds checks but no others.
    #[allow(deprecated)]
    pub fn freeze(self) -> VMResult<CompiledScript> {
        let (info, fake_module) = self.into_module();
        Ok(fake_module.freeze()?.into_script(info))
    }

    /// Converts a `CompiledScriptMut` to a `CompiledModule` for code that wants a uniform view
    /// of both.
    ///
    /// This also produces a `ScriptConversionInfo`, which will be required to convert the
    /// module back into a script.
    ///
    /// TODO: rewrite things that depend on this and get this removed.
    #[deprecated(
        note = "This function is deprecated and will be removed soon. Please do not introduce new dependencies."
    )]
    pub fn into_module(mut self) -> (ScriptConversionInfo, CompiledModuleMut) {
        // Add the "<SELF>" identifier if it isn't present.
        //
        // Note: When adding an element to the table, in theory it is possible for the index
        // to overflow. This will not be a problem if we get rid of the script/module conversion.
        let (added_self_ident, self_ident_idx) = match self
            .identifiers
            .iter()
            .position(|ident| ident.as_ident_str() == self_module_name())
        {
            Some(idx) => (false, IdentifierIndex::new(idx as u16)),
            None => {
                let idx = IdentifierIndex::new(self.identifiers.len() as u16);
                self.identifiers
                    .push(Identifier::new(self_module_name().to_string()).unwrap());
                (true, idx)
            }
        };

        // Add a dummy adress if none exists.
        let dummy_addr = AccountAddress::new([0xff; AccountAddress::LENGTH]);
        let (added_dummy_addr, dummy_addr_idx) = match self
            .address_identifiers
            .iter()
            .position(|addr| addr == &dummy_addr)
        {
            Some(idx) => (false, AddressIdentifierIndex::new(idx as u16)),
            None => {
                let idx = AddressIdentifierIndex::new(self.address_identifiers.len() as u16);
                self.address_identifiers.push(dummy_addr);
                (true, idx)
            }
        };

        // Add a self module handle.
        let (added_self_module_handle, self_module_handle_idx) =
            match self.module_handles.iter().position(|handle| {
                handle.address == dummy_addr_idx && handle.name == self_ident_idx
            }) {
                Some(idx) => (false, ModuleHandleIndex::new(idx as u16)),
                None => {
                    let idx = ModuleHandleIndex::new(self.module_handles.len() as u16);
                    self.module_handles.push(ModuleHandle {
                        address: dummy_addr_idx,
                        name: self_ident_idx,
                    });
                    (true, idx)
                }
            };

        // Find the index to the empty signature [].
        // Create one if it doesn't exist.
        let (added_dummy_sig, return_sig_idx) =
            match self.signatures.iter().position(|sig| sig.0.is_empty()) {
                Some(idx) => (false, SignatureIndex::new(idx as u16)),
                None => {
                    let idx = SignatureIndex::new(self.signatures.len() as u16);
                    self.signatures.push(Signature(vec![]));
                    (true, idx)
                }
            };

        // Create a function handle for the main function.
        let main_handle_idx = FunctionHandleIndex::new(self.function_handles.len() as u16);
        self.function_handles.push(FunctionHandle {
            module: self_module_handle_idx,
            name: self_ident_idx,
            parameters: self.parameters,
            return_: return_sig_idx,
            type_parameters: self.type_parameters,
        });

        // Create a function definition for the main function.
        let main_def = FunctionDefinition {
            function: main_handle_idx,
            is_public: true,
            acquires_global_resources: vec![],
            code: Some(self.code),
        };

        let info = ScriptConversionInfo {
            added_dummy_addr,
            added_dummy_sig,
            added_self_ident,
            added_self_module_handle,
        };

        let m = CompiledModuleMut {
            module_handles: self.module_handles,
            self_module_handle_idx,
            struct_handles: self.struct_handles,
            function_handles: self.function_handles,
            field_handles: vec![],

            struct_def_instantiations: vec![],
            function_instantiations: self.function_instantiations,
            field_instantiations: vec![],

            signatures: self.signatures,

            identifiers: self.identifiers,
            address_identifiers: self.address_identifiers,
            constant_pool: self.constant_pool,

            struct_defs: vec![],
            function_defs: vec![main_def],
        };

        (info, m)
    }
}

/// A `CompiledModule` defines the structure of a module which is the unit of published code.
///
/// A `CompiledModule` contains a definition of types (with their fields) and functions.
/// It is a unit of code that can be used by transactions or other modules.
///
/// A module is published as a single entry and it is retrieved as a single blob.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledModule(CompiledModuleMut);

/// A mutable version of `CompiledModule`. Converting to a `CompiledModule` requires this to pass
/// the bounds checker.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CompiledModuleMut {
    /// Handles to external modules and self.
    pub module_handles: Vec<ModuleHandle>,
    /// Handle to self.
    pub self_module_handle_idx: ModuleHandleIndex,
    /// Handles to external and internal types.
    pub struct_handles: Vec<StructHandle>,
    /// Handles to external and internal functions.
    pub function_handles: Vec<FunctionHandle>,
    /// Handles to fields.
    pub field_handles: Vec<FieldHandle>,

    /// Struct instantiations.
    pub struct_def_instantiations: Vec<StructDefInstantiation>,
    /// Function instantiations.
    pub function_instantiations: Vec<FunctionInstantiation>,
    /// Field instantiations.
    pub field_instantiations: Vec<FieldInstantiation>,

    /// Locals signature pool. The signature for all locals of the functions defined in
    /// the module.
    pub signatures: SignaturePool,

    /// All identifiers used in this module.
    pub identifiers: IdentifierPool,
    /// All address identifiers used in this module.
    pub address_identifiers: AddressIdentifierPool,
    /// Constant pool. The constant values used in the module.
    pub constant_pool: ConstantPool,

    /// Types defined in this module.
    pub struct_defs: Vec<StructDefinition>,
    /// Function defined in this module.
    pub function_defs: Vec<FunctionDefinition>,
}

// Need a custom implementation of Arbitrary because as of proptest-derive 0.1.1, the derivation
// doesn't work for structs with more than 10 fields.
#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for CompiledScriptMut {
    type Strategy = BoxedStrategy<Self>;
    /// The size of the compiled script.
    type Parameters = usize;

    fn arbitrary_with(size: Self::Parameters) -> Self::Strategy {
        (
            (
                vec(any::<ModuleHandle>(), 0..=size),
                vec(any::<StructHandle>(), 0..=size),
                vec(any::<FunctionHandle>(), 0..=size),
            ),
            vec(any_with::<Signature>(size), 0..=size),
            (
                vec(any::<Identifier>(), 0..=size),
                vec(any::<AccountAddress>(), 0..=size),
            ),
            vec(any::<Kind>(), 0..=size),
            any::<SignatureIndex>(),
            any::<CodeUnit>(),
        )
            .prop_map(
                |(
                    (module_handles, struct_handles, function_handles),
                    signatures,
                    (identifiers, address_identifiers),
                    type_parameters,
                    parameters,
                    code,
                )| {
                    // TODO actual constant generation
                    CompiledScriptMut {
                        module_handles,
                        struct_handles,
                        function_handles,
                        function_instantiations: vec![],
                        signatures,
                        identifiers,
                        address_identifiers,
                        constant_pool: vec![],
                        type_parameters,
                        parameters,
                        code,
                    }
                },
            )
            .boxed()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for CompiledModuleMut {
    type Strategy = BoxedStrategy<Self>;
    /// The size of the compiled module.
    type Parameters = usize;

    fn arbitrary_with(size: Self::Parameters) -> Self::Strategy {
        (
            (
                vec(any::<ModuleHandle>(), 0..=size),
                vec(any::<StructHandle>(), 0..=size),
                vec(any::<FunctionHandle>(), 0..=size),
            ),
            any::<ModuleHandleIndex>(),
            vec(any_with::<Signature>(size), 0..=size),
            (
                vec(any::<Identifier>(), 0..=size),
                vec(any::<AccountAddress>(), 0..=size),
            ),
            (
                vec(any::<StructDefinition>(), 0..=size),
                vec(any_with::<FunctionDefinition>(size), 0..=size),
            ),
        )
            .prop_map(
                |(
                    (module_handles, struct_handles, function_handles),
                    self_module_handle_idx,
                    signatures,
                    (identifiers, address_identifiers),
                    (struct_defs, function_defs),
                )| {
                    // TODO actual constant generation
                    CompiledModuleMut {
                        module_handles,
                        struct_handles,
                        function_handles,
                        self_module_handle_idx,
                        field_handles: vec![],
                        struct_def_instantiations: vec![],
                        function_instantiations: vec![],
                        field_instantiations: vec![],
                        signatures,
                        identifiers,
                        address_identifiers,
                        constant_pool: vec![],
                        struct_defs,
                        function_defs,
                    }
                },
            )
            .boxed()
    }
}

impl CompiledModuleMut {
    /// Returns the count of a specific `IndexKind`
    pub fn kind_count(&self, kind: IndexKind) -> usize {
        match kind {
            IndexKind::ModuleHandle => self.module_handles.len(),
            IndexKind::StructHandle => self.struct_handles.len(),
            IndexKind::FunctionHandle => self.function_handles.len(),
            IndexKind::FieldHandle => self.field_handles.len(),
            IndexKind::StructDefInstantiation => self.struct_def_instantiations.len(),
            IndexKind::FunctionInstantiation => self.function_instantiations.len(),
            IndexKind::FieldInstantiation => self.field_instantiations.len(),
            IndexKind::StructDefinition => self.struct_defs.len(),
            IndexKind::FunctionDefinition => self.function_defs.len(),
            IndexKind::Signature => self.signatures.len(),
            IndexKind::Identifier => self.identifiers.len(),
            IndexKind::AddressIdentifier => self.address_identifiers.len(),
            IndexKind::ConstantPool => self.constant_pool.len(),
            // XXX these two don't seem to belong here
            other @ IndexKind::LocalPool
            | other @ IndexKind::CodeDefinition
            | other @ IndexKind::FieldDefinition
            | other @ IndexKind::TypeParameter
            | other @ IndexKind::MemberCount => unreachable!("invalid kind for count: {:?}", other),
        }
    }

    /// Converts this instance into `CompiledModule` after verifying it for basic internal
    /// consistency. This includes bounds checks but no others.
    pub fn freeze(self) -> VMResult<CompiledModule> {
        BoundsChecker::new(&self).verify()?;
        Ok(CompiledModule(self))
    }
}

impl CompiledModule {
    /// Returns a reference to the inner `CompiledModuleMut`.
    pub fn as_inner(&self) -> &CompiledModuleMut {
        &self.0
    }

    /// Converts this instance into the inner `CompiledModuleMut`. Converting back to a
    /// `CompiledModule` would require it to be verified again.
    pub fn into_inner(self) -> CompiledModuleMut {
        self.0
    }

    /// Returns the number of items of a specific `IndexKind`.
    pub fn kind_count(&self, kind: IndexKind) -> usize {
        precondition!(match kind {
            IndexKind::LocalPool
            | IndexKind::CodeDefinition
            | IndexKind::FieldDefinition
            | IndexKind::TypeParameter
            | IndexKind::MemberCount => false,
            _ => true,
        });
        self.as_inner().kind_count(kind)
    }

    /// Returns the code key of `module_handle`
    pub fn module_id_for_handle(&self, module_handle: &ModuleHandle) -> ModuleId {
        ModuleId::new(
            *self.address_identifier_at(module_handle.address),
            self.identifier_at(module_handle.name).to_owned(),
        )
    }

    /// Returns the code key of `self`
    pub fn self_id(&self) -> ModuleId {
        self.module_id_for_handle(self.self_handle())
    }

    /// This function should only be called on an instance of CompiledModule obtained by invoking
    /// into_module on some instance of CompiledScript. This function is the inverse of
    /// into_module, i.e., script.into_module().into_script() == script.
    #[deprecated(
        note = "This function is deprecated and will be removed soon. Please do not introduce new dependencies."
    )]
    pub fn into_script(self, conv_info: ScriptConversionInfo) -> CompiledScript {
        let mut inner = self.into_inner();
        precondition!(!inner.function_defs.is_empty());
        let main = inner.function_defs.pop().unwrap();

        if conv_info.added_dummy_addr {
            inner.address_identifiers.pop().unwrap();
        }
        if conv_info.added_dummy_sig {
            inner.signatures.pop().unwrap();
        }
        if conv_info.added_self_ident {
            inner.identifiers.pop().unwrap();
        }
        if conv_info.added_self_module_handle {
            inner.module_handles.pop().unwrap();
        }
        let main_handle = inner.function_handles.pop().unwrap();

        CompiledScript(CompiledScriptMut {
            module_handles: inner.module_handles,
            struct_handles: inner.struct_handles,
            function_handles: inner.function_handles,

            function_instantiations: inner.function_instantiations,

            signatures: inner.signatures,

            identifiers: inner.identifiers,
            address_identifiers: inner.address_identifiers,
            constant_pool: inner.constant_pool,

            type_parameters: main_handle.type_parameters,
            parameters: main_handle.parameters,
            code: main.code.unwrap(),
        })
    }
}

/// Return the simplest module that will pass the bounds checker
pub fn empty_module() -> CompiledModuleMut {
    CompiledModuleMut {
        module_handles: vec![ModuleHandle {
            address: AddressIdentifierIndex(0),
            name: IdentifierIndex(0),
        }],
        self_module_handle_idx: ModuleHandleIndex(0),
        identifiers: vec![self_module_name().to_owned()],
        address_identifiers: vec![AccountAddress::default()],
        constant_pool: vec![],
        function_defs: vec![],
        struct_defs: vec![],
        struct_handles: vec![],
        function_handles: vec![],
        field_handles: vec![],
        struct_def_instantiations: vec![],
        function_instantiations: vec![],
        field_instantiations: vec![],
        signatures: vec![Signature(vec![])],
    }
}

/// Create the following module which is convenient in tests:
/// // module <SELF> {
/// //     struct Bar { x: u64 }
/// //
/// //     foo() {
/// //     }
/// // }
pub fn basic_test_module() -> CompiledModuleMut {
    let mut m = empty_module();

    m.function_handles.push(FunctionHandle {
        module: ModuleHandleIndex(0),
        name: IdentifierIndex(m.identifiers.len() as u16),
        parameters: SignatureIndex(0),
        return_: SignatureIndex(0),
        type_parameters: vec![],
    });
    m.identifiers
        .push(Identifier::new("foo".to_string()).unwrap());

    m.function_defs.push(FunctionDefinition {
        function: FunctionHandleIndex(0),
        is_public: false,
        acquires_global_resources: vec![],
        code: Some(CodeUnit {
            locals: SignatureIndex(0),
            code: vec![],
        }),
    });

    m.struct_handles.push(StructHandle {
        module: ModuleHandleIndex(0),
        name: IdentifierIndex(m.identifiers.len() as u16),
        is_nominal_resource: false,
        type_parameters: vec![],
    });
    m.identifiers
        .push(Identifier::new("Bar".to_string()).unwrap());

    m.struct_defs.push(StructDefinition {
        struct_handle: StructHandleIndex(0),
        field_information: StructFieldInformation::Declared(vec![FieldDefinition {
            name: IdentifierIndex(m.identifiers.len() as u16),
            signature: TypeSignature(SignatureToken::U64),
        }]),
    });
    m.identifiers
        .push(Identifier::new("x".to_string()).unwrap());

    m
}

/// Create a dummy module to wrap the bytecode program in local@code
pub fn dummy_procedure_module(code: Vec<Bytecode>) -> CompiledModule {
    let mut module = empty_module();
    let mut code_unit = CodeUnit::default();
    code_unit.code = code;
    let mut fun_def = FunctionDefinition::default();
    fun_def.code = Some(code_unit);

    let fun_handle = FunctionHandle {
        module: ModuleHandleIndex(0),
        name: IdentifierIndex(0),
        parameters: SignatureIndex(0),
        return_: SignatureIndex(0),
        type_parameters: vec![],
    };

    module.function_handles.push(fun_handle);
    module.function_defs.push(fun_def);
    module.freeze().unwrap()
}

/// Return a simple script that contains only a return in the main()
pub fn empty_script() -> CompiledScriptMut {
    CompiledScriptMut {
        module_handles: vec![],
        struct_handles: vec![],
        function_handles: vec![],

        function_instantiations: vec![],

        signatures: vec![Signature(vec![])],

        identifiers: vec![],
        address_identifiers: vec![],
        constant_pool: vec![],

        type_parameters: vec![],
        parameters: SignatureIndex(0),
        code: CodeUnit {
            locals: SignatureIndex(0),
            code: vec![Bytecode::Ret],
        },
    }
}
