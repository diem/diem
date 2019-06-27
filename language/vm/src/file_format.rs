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
    access::ModuleAccess, check_bounds::BoundsChecker, errors::VerificationError,
    internals::ModuleIndex, IndexKind, SignatureTokenKind,
};
use proptest::{collection::vec, prelude::*, strategy::BoxedStrategy};
use proptest_derive::Arbitrary;
use types::{account_address::AccountAddress, byte_array::ByteArray, language_storage::ModuleId};

/// Generic index into one of the tables in the binary format.
pub type TableIndex = u16;

macro_rules! define_index {
    {
        name: $name: ident,
        kind: $kind: ident,
        doc: $comment: literal,
    } => {
        #[derive(Arbitrary, Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[proptest(no_params)]
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
    name: StringPoolIndex,
    kind: StringPool,
    doc: "Index into the `StringPool` table.",
}
define_index! {
    name: ByteArrayPoolIndex,
    kind: ByteArrayPool,
    doc: "Index into the `ByteArrayPool` table.",
}
define_index! {
    name: AddressPoolIndex,
    kind: AddressPool,
    doc: "Index into the `AddressPool` table.",
}
define_index! {
    name: TypeSignatureIndex,
    kind: TypeSignature,
    doc: "Index into the `TypeSignature` table.",
}
define_index! {
    name: FunctionSignatureIndex,
    kind: FunctionSignature,
    doc: "Index into the `FunctionSignature` table.",
}
define_index! {
    name: LocalsSignatureIndex,
    kind: LocalsSignature,
    doc: "Index into the `LocalsSignature` table.",
}
define_index! {
    name: StructDefinitionIndex,
    kind: StructDefinition,
    doc: "Index into the `StructDefinition` table.",
}
define_index! {
    name: FieldDefinitionIndex,
    kind: FieldDefinition,
    doc: "Index into the `FieldDefinition` table.",
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

/// The pool of identifiers and string literals.
pub type StringPool = Vec<String>;
/// The pool of `ByteArray` literals.
pub type ByteArrayPool = Vec<ByteArray>;
/// The pool of `AccountAddress` literals.
///
/// Code references have a literal addresses in `ModuleHandle`s. Literal references to data in
/// the blockchain are also published here.
pub type AddressPool = Vec<AccountAddress>;
/// The pool of `TypeSignature` instances. Those are system and user types used and
/// their composition (e.g. &U64).
pub type TypeSignaturePool = Vec<TypeSignature>;
/// The pool of `FunctionSignature` instances.
pub type FunctionSignaturePool = Vec<FunctionSignature>;
/// The pool of `LocalsSignature` instances. Every function definition must define the set of
/// locals used and their types.
pub type LocalsSignaturePool = Vec<LocalsSignature>;

/// Name of the placeholder module. Every compiled script has an entry that
/// refers to itself in its module handle list. This is the name of that script.
pub const SELF_MODULE_NAME: &str = "<SELF>";

// HANDLES:
// Handles are structs that accompany opcodes that need references: a type reference,
// or a function reference (a field reference being available only within the module that
// defrines the field can be a definition).
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
#[derive(Arbitrary, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[proptest(no_params)]
pub struct ModuleHandle {
    /// Index into the `AddressPool`. Identifies the account that holds the module.
    pub address: AddressPoolIndex,
    /// The name of the module published in the code section for the account in `address`.
    pub name: StringPoolIndex,
}

/// A `StructHandle` is a reference to a user defined type. It is composed by a `ModuleHandle`
/// and the name of the type within that module.
///
/// A type in a module is uniquely identified by its name and as such the name is enough
/// to perform resolution.
///
/// The `StructHandle` also carries the type *kind* (resource/unrestricted) so that the verifier
/// can check resource semantic without having to load the referenced type.
/// At link time a check of the kind is performed and an error is reported if there is a
/// mismatch with the definition.
#[derive(Arbitrary, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[proptest(no_params)]
pub struct StructHandle {
    /// The module that defines the type.
    pub module: ModuleHandleIndex,
    /// The name of the type.
    pub name: StringPoolIndex,
    /// Whether the type is a resource or an unrestricted type.
    pub is_resource: bool,
}

/// A `FunctionHandle` is a reference to a function. It is composed by a
/// `ModuleHandle` and the name and signature of that function within the module.
///
/// A function within a module is uniquely identified by its name. No overloading is allowed
/// and the verifier enforces that property. The signature of the function is used at link time to
/// ensure the function reference is valid and it is also used by the verifier to type check
/// function calls.
#[derive(Arbitrary, Clone, Debug, Eq, Hash, PartialEq)]
#[proptest(no_params)]
pub struct FunctionHandle {
    /// The module that defines the function.
    pub module: ModuleHandleIndex,
    /// The name of the function.
    pub name: StringPoolIndex,
    /// The signature of the function.
    pub signature: FunctionSignatureIndex,
}

// DEFINITIONS:
// Definitions are the module code. So the set of types and functions in the module.

/// A `StructDefinition` is a user type definition. It defines all the fields declared on the type.
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
#[proptest(no_params)]
pub struct StructDefinition {
    /// The `StructHandle` for this `StructDefinition`. This has the name and the resource flag
    /// for the type.
    pub struct_handle: StructHandleIndex,
    /// The number of fields in this type.
    pub field_count: MemberCount,
    /// The starting index for the fields of this type. `FieldDefinition`s for each type must
    /// be consecutively stored in the `FieldDefinition` table.
    pub fields: FieldDefinitionIndex,
}

/// A `FieldDefinition` is the definition of a field: the type the field is defined on,
/// its name and the field type.
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
#[proptest(no_params)]
pub struct FieldDefinition {
    /// The type (resource or unrestricted) the field is defined on.
    pub struct_: StructHandleIndex,
    /// The name of the field.
    pub name: StringPoolIndex,
    /// The type of the field.
    pub signature: TypeSignatureIndex,
}

/// A `FunctionDefinition` is the implementation of a function. It defines
/// the *prototype* of the function and the function body.
#[derive(Arbitrary, Clone, Debug, Default, Eq, PartialEq)]
#[proptest(params = "usize")]
pub struct FunctionDefinition {
    /// The prototype of the function (module, name, signature).
    pub function: FunctionHandleIndex,
    /// Flags for this function (private, public, native, etc.)
    pub flags: u8,
    /// Code for this function.
    #[proptest(strategy = "any_with::<CodeUnit>(params)")]
    pub code: CodeUnit,
}

impl FunctionDefinition {
    /// Returns whether the FunctionDefinition is public.
    pub fn is_public(&self) -> bool {
        self.flags & CodeUnit::PUBLIC != 0
    }
    /// Returns whether the FunctionDefinition is native.
    pub fn is_native(&self) -> bool {
        self.flags & CodeUnit::NATIVE != 0
    }
}

// Signature definitions.
// A signature can be for a type (field, local) or for a function - return type: (arguments).
// They both go into the signature table so there is a marker that tags the signature.
// Signature usually don't carry a size and you have to read them to get to the end.

/// A type definition. `SignatureToken` allows the definition of the set of known types and their
/// composition.
#[derive(Arbitrary, Clone, Debug, Eq, Hash, PartialEq)]
#[proptest(no_params)]
pub struct TypeSignature(pub SignatureToken);

/// A `FunctionSignature` describes the arguments and the return types of a function.
#[derive(Arbitrary, Clone, Debug, Eq, Hash, PartialEq)]
#[proptest(params = "usize")]
pub struct FunctionSignature {
    /// The list of return types.
    #[proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")]
    pub return_types: Vec<SignatureToken>,
    /// The list of arguments to the function.
    #[proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")]
    pub arg_types: Vec<SignatureToken>,
}

/// A `LocalsSignature` is the list of locals used by a function.
///
/// Locals include the arguments to the function from position `0` to argument `count - 1`.
/// The remaining elements are the type of each local.
#[derive(Arbitrary, Clone, Debug, Default, Eq, Hash, PartialEq)]
#[proptest(params = "usize")]
pub struct LocalsSignature(
    #[proptest(strategy = "vec(any::<SignatureToken>(), 0..=params)")] pub Vec<SignatureToken>,
);

impl LocalsSignature {
    /// Length of the `LocalsSignature`.
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
    /// Unsigned integers, 64 bits length.
    U64,
    /// Strings, immutable, utf8 representation.
    String,
    /// ByteArray, variable size, immutable byte array.
    ByteArray,
    /// Address, a 32 bytes immutable type.
    Address,
    /// MOVE user type, resource or unrestricted
    Struct(StructHandleIndex),
    /// Reference to a type.
    Reference(Box<SignatureToken>),
    /// Immutable reference to a type.
    MutableReference(Box<SignatureToken>),
}

/// `Arbitrary` for `SignatureToken` cannot be derived automatically as it's a recursive type.
impl Arbitrary for SignatureToken {
    type Strategy = BoxedStrategy<Self>;
    type Parameters = ();

    fn arbitrary_with(_params: Self::Parameters) -> Self::Strategy {
        use SignatureToken::*;

        let leaf = prop_oneof![
            Just(Bool),
            Just(U64),
            Just(String),
            Just(ByteArray),
            Just(Address),
            any::<StructHandleIndex>().prop_map(Struct),
        ];
        leaf.prop_recursive(
            8,  // levels deep
            16, // max size
            1,  // items per collection
            |inner| {
                prop_oneof![
                    inner.clone().prop_map(|token| Reference(Box::new(token))),
                    inner
                        .clone()
                        .prop_map(|token| MutableReference(Box::new(token))),
                ]
            },
        )
        .boxed()
    }
}

impl ::std::fmt::Debug for SignatureToken {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SignatureToken::Bool => write!(f, "Bool"),
            SignatureToken::U64 => write!(f, "U64"),
            SignatureToken::String => write!(f, "String"),
            SignatureToken::ByteArray => write!(f, "ByteArray"),
            SignatureToken::Address => write!(f, "Address"),
            SignatureToken::Struct(idx) => write!(f, "Struct({:?})", idx),
            SignatureToken::Reference(boxed) => write!(f, "Reference({:?})", boxed),
            SignatureToken::MutableReference(boxed) => write!(f, "MutableReference({:?})", boxed),
        }
    }
}

impl SignatureToken {
    /// If a `SignatureToken` is a reference it returns the underlying type of the reference (e.g.
    /// U64 for &U64).
    #[inline]
    pub fn get_struct_handle_from_reference(
        reference_signature: &SignatureToken,
    ) -> Option<StructHandleIndex> {
        match reference_signature {
            SignatureToken::Reference(signature) => match **signature {
                SignatureToken::Struct(idx) => Some(idx),
                _ => None,
            },
            SignatureToken::MutableReference(signature) => match **signature {
                SignatureToken::Struct(idx) => Some(idx),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns the "kind" for the `SignatureToken`
    #[inline]
    pub fn kind(&self) -> SignatureTokenKind {
        // TODO: SignatureTokenKind is out-dated. fix/update/remove SignatureTokenKind and see if
        // this function needs to be cleaned up
        use SignatureToken::*;

        match self {
            Reference(_) => SignatureTokenKind::Reference,
            MutableReference(_) => SignatureTokenKind::MutableReference,
            Bool | U64 | ByteArray | String | Address | Struct(_) => SignatureTokenKind::Value,
        }
    }

    /// Returns the `StructHandleIndex` for a `SignatureToken` that contains a reference to a user
    /// defined type (a resource or unrestricted type).
    #[inline]
    pub fn struct_index(&self) -> Option<StructHandleIndex> {
        use SignatureToken::*;

        match self {
            Struct(sh_idx) => Some(*sh_idx),
            Reference(token) | MutableReference(token) => token.struct_index(),
            Bool | U64 | ByteArray | String | Address => None,
        }
    }

    /// Returns `true` if the `SignatureToken` is a primitive type.
    pub fn is_primitive(&self) -> bool {
        use SignatureToken::*;
        match self {
            Bool | U64 | String | ByteArray | Address => true,
            Struct(_) | Reference(_) | MutableReference(_) => false,
        }
    }

    /// Checks if the signature token is usable for Eq and Neq.
    ///
    /// Currently equality operations are only allowed on:
    /// - Bool
    /// - U64
    /// - String
    /// - ByteArray
    /// - Address
    /// - Reference or Mutable reference to these types
    pub fn allows_equality(&self) -> bool {
        use SignatureToken::*;
        match self {
            Struct(_) => false,
            Reference(token) | MutableReference(token) => token.is_primitive(),
            token => token.is_primitive(),
        }
    }

    /// Returns true if the `SignatureToken` is any kind of reference (mutable and immutable).
    pub fn is_reference(&self) -> bool {
        use SignatureToken::*;

        match self {
            Reference(_) | MutableReference(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `SignatureToken` is a mutable reference.
    pub fn is_mutable_reference(&self) -> bool {
        use SignatureToken::*;

        match self {
            MutableReference(_) => true,
            _ => false,
        }
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
}

/// A `CodeUnit` is the body of a function. It has the function header and the instruction stream.
#[derive(Arbitrary, Clone, Debug, Default, Eq, PartialEq)]
#[proptest(params = "usize")]
pub struct CodeUnit {
    /// Max stack size for the function - currently unused.
    pub max_stack_size: u16,
    /// List of locals type. All locals are typed.
    pub locals: LocalsSignatureIndex,
    /// Code stream, function body.
    #[proptest(strategy = "vec(any::<Bytecode>(), 0..=params)")]
    pub code: Vec<Bytecode>,
}

/// Flags for `FunctionDeclaration`.
impl CodeUnit {
    /// Function can be invoked outside of its declaring module.
    pub const PUBLIC: u8 = 0x1;
    /// A native function implemented in Rust.
    pub const NATIVE: u8 = 0x2;
}

/// `Bytecode` is a VM instruction of variable size. The type of the bytecode (opcode) defines
/// the size of the bytecode.
///
/// Bytecodes operate on a stack machine and each bytecode has side effect on the stack and the
/// instruction stream.
#[derive(Arbitrary, Clone, Hash, Eq, PartialEq)]
#[proptest(no_params)]
pub enum Bytecode {
    /// Pop and discard the value at the top of the stack.
    /// The value on the stack must be an unrestricted type.
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
    /// Push integer constant onto the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    LdConst(u64),
    /// Push a `string` literal onto the stack. The string is loaded from the `StringPool` via
    /// `StringPoolIndex`.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., string_value```
    LdStr(StringPoolIndex),
    /// Push a `ByteArray` literal onto the stack. The `ByteArray` is loaded from the
    /// `ByteArrayPool` via `ByteArrayPoolIndex`.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., bytearray_value```
    LdByteArray(ByteArrayPoolIndex),
    /// Push an 'Address' literal onto the stack. The address is loaded from the
    /// `AddressPool` via `AddressPoolIndex`.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., address_value```
    LdAddr(AddressPoolIndex),
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
    /// The reference must be to an unrestricted type because Resources cannot be overwritten.
    ///
    /// Stack transition:
    ///
    /// ```..., value, reference_value -> ...```
    WriteRef,
    /// Release a reference. The reference will become invalid and cannot be used after.
    ///
    /// All references must be consumed and ReleaseRef is a way to release references not
    /// consumed by other opcodes.
    ///
    /// Stack transition:
    ///
    /// ```..., reference_value -> ...```
    ReleaseRef,
    /// Convert a mutable reference to an immutable reference.
    ///
    /// Stack transition:
    ///
    /// ```..., reference_value -> ..., reference_value```
    FreezeRef,
    /// Load a reference to a local identified by LocalIndex.
    ///
    /// The local must not be a reference.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., reference```
    BorrowLoc(LocalIndex),
    /// Load a reference to a field identified by `FieldDefinitionIndex`.
    /// The top of the stack must be a reference to a type that contains the field definition.
    ///
    /// Stack transition:
    ///
    /// ```..., reference -> ..., field_reference```
    BorrowField(FieldDefinitionIndex),
    /// Return reference to an instance of type `StructDefinitionIndex` published at the address
    /// passed as argument. Abort execution if such an object does not exist or if a reference
    /// has already been handed out.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., reference_value```
    BorrowGlobal(StructDefinitionIndex),
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
    /// asserts that the value at the top of the stack is true. Abort execution with
    /// errorcode otherwise.
    ///
    ///
    /// Stack transition:
    ///
    /// ```..., bool_value, errorcode -> ...```
    Assert,
    /// Get gas unit price from the transaction and pushes it on the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    GetTxnGasUnitPrice,
    /// Get max gas units set in the transaction and pushes it on the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    GetTxnMaxGasUnits,
    /// Get remaining gas for the given transaction at the point of execution of this bytecode.
    /// The result is pushed on the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    GetGasRemaining,
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
    /// Move the instance of type StructDefinitionIndex, at the address at the top of the stack.
    /// Abort execution if such an object does not exist.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ..., value```
    MoveFrom(StructDefinitionIndex),
    /// Move the instance at the top of the stack to the address of the sender.
    /// Abort execution if an object of type StructDefinitionIndex already exists in address.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ...```
    MoveToSender(StructDefinitionIndex),
    /// Create an account at the address specified. Does not return anything.
    ///
    /// Stack transition:
    ///
    /// ```..., address_value -> ...```
    CreateAccount,
    /// Emit a log message.
    /// This bytecode is not fully specified yet.
    ///
    /// Stack transition:
    ///
    /// ```..., reference, key, value -> ...```
    EmitEvent,
    /// Get the sequence number submitted with the transaction and pushes it on the stack.
    ///
    /// Stack transition:
    ///
    /// ```... -> ..., u64_value```
    GetTxnSequenceNumber,
    /// Get the public key of the sender from the transaction and pushes it on the stack.
    ///
    /// Stack transition:
    ///
    /// ```..., -> ..., bytearray_value```
    GetTxnPublicKey,
}

impl ::std::fmt::Debug for Bytecode {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Bytecode::Pop => write!(f, "Pop"),
            Bytecode::Ret => write!(f, "Ret"),
            Bytecode::BrTrue(a) => write!(f, "BrTrue({})", a),
            Bytecode::BrFalse(a) => write!(f, "BrFalse({})", a),
            Bytecode::Branch(a) => write!(f, "Branch({})", a),
            Bytecode::LdConst(a) => write!(f, "LdConst({})", a),
            Bytecode::LdStr(a) => write!(f, "LdStr({})", a),
            Bytecode::LdByteArray(a) => write!(f, "LdByteArray({})", a),
            Bytecode::LdAddr(a) => write!(f, "LdAddr({})", a),
            Bytecode::LdTrue => write!(f, "LdTrue"),
            Bytecode::LdFalse => write!(f, "LdFalse"),
            Bytecode::CopyLoc(a) => write!(f, "CopyLoc({})", a),
            Bytecode::MoveLoc(a) => write!(f, "MoveLoc({})", a),
            Bytecode::StLoc(a) => write!(f, "StLoc({})", a),
            Bytecode::Call(a) => write!(f, "Call({})", a),
            Bytecode::Pack(a) => write!(f, "Pack({})", a),
            Bytecode::Unpack(a) => write!(f, "Unpack({})", a),
            Bytecode::ReadRef => write!(f, "ReadRef"),
            Bytecode::WriteRef => write!(f, "WriteRef"),
            Bytecode::ReleaseRef => write!(f, "ReleaseRef"),
            Bytecode::FreezeRef => write!(f, "FreezeRef"),
            Bytecode::BorrowLoc(a) => write!(f, "BorrowLoc({})", a),
            Bytecode::BorrowField(a) => write!(f, "BorrowField({})", a),
            Bytecode::BorrowGlobal(a) => write!(f, "BorrowGlobal({})", a),
            Bytecode::Add => write!(f, "Add"),
            Bytecode::Sub => write!(f, "Sub"),
            Bytecode::Mul => write!(f, "Mul"),
            Bytecode::Mod => write!(f, "Mod"),
            Bytecode::Div => write!(f, "Div"),
            Bytecode::BitOr => write!(f, "BitOr"),
            Bytecode::BitAnd => write!(f, "BitAnd"),
            Bytecode::Xor => write!(f, "Xor"),
            Bytecode::Or => write!(f, "Or"),
            Bytecode::And => write!(f, "And"),
            Bytecode::Not => write!(f, "Not"),
            Bytecode::Eq => write!(f, "Eq"),
            Bytecode::Neq => write!(f, "Neq"),
            Bytecode::Lt => write!(f, "Lt"),
            Bytecode::Gt => write!(f, "Gt"),
            Bytecode::Le => write!(f, "Le"),
            Bytecode::Ge => write!(f, "Ge"),
            Bytecode::Assert => write!(f, "Assert"),
            Bytecode::GetTxnGasUnitPrice => write!(f, "GetTxnGasUnitPrice"),
            Bytecode::GetTxnMaxGasUnits => write!(f, "GetTxnMaxGasUnits"),
            Bytecode::GetGasRemaining => write!(f, "GetGasRemaining"),
            Bytecode::GetTxnSenderAddress => write!(f, "GetTxnSenderAddress"),
            Bytecode::Exists(a) => write!(f, "Exists({})", a),
            Bytecode::MoveFrom(a) => write!(f, "MoveFrom({})", a),
            Bytecode::MoveToSender(a) => write!(f, "MoveToSender({})", a),
            Bytecode::CreateAccount => write!(f, "CreateAccount"),
            Bytecode::EmitEvent => write!(f, "EmitEvent"),
            Bytecode::GetTxnSequenceNumber => write!(f, "GetTxnSequenceNumber"),
            Bytecode::GetTxnPublicKey => write!(f, "GetTxnPublicKey"),
        }
    }
}

/// A `CompiledProgram` defines the structure of a transaction to execute.
/// It has two parts: modules to be published and a transaction script.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct CompiledProgram {
    /// The modules to be published
    pub modules: Vec<CompiledModule>,
    /// The transaction script to execute
    pub script: CompiledScript,
}

impl CompiledProgram {
    /// Creates a new compiled program from compiled modules and script
    pub fn new(modules: Vec<CompiledModule>, script: CompiledScript) -> Self {
        CompiledProgram { modules, script }
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
#[derive(Arbitrary, Clone, Default, Eq, PartialEq, Debug)]
#[proptest(params = "usize")]
pub struct CompiledScriptMut {
    /// Handles to all modules referenced.
    #[proptest(strategy = "vec(any::<ModuleHandle>(), 0..=params)")]
    pub module_handles: Vec<ModuleHandle>,
    /// Handles to external/imported types.
    #[proptest(strategy = "vec(any::<StructHandle>(), 0..=params)")]
    pub struct_handles: Vec<StructHandle>,
    /// Handles to external/imported functions.
    #[proptest(strategy = "vec(any::<FunctionHandle>(), 0..=params)")]
    pub function_handles: Vec<FunctionHandle>,

    /// Type pool. All external types referenced by the transaction.
    #[proptest(strategy = "vec(any::<TypeSignature>(), 0..=params)")]
    pub type_signatures: TypeSignaturePool,
    /// Function signature pool. The signatures of the function referenced by the transaction.
    #[proptest(strategy = "vec(any_with::<FunctionSignature>(params), 0..=params)")]
    pub function_signatures: FunctionSignaturePool,
    /// Locals signature pool. The signature of the locals in `main`.
    #[proptest(strategy = "vec(any_with::<LocalsSignature>(params), 0..=params)")]
    pub locals_signatures: LocalsSignaturePool,

    /// String pool. All literals and identifiers used in this transaction.
    #[proptest(strategy = "vec(\".*\", 0..=params)")]
    pub string_pool: StringPool,
    /// ByteArray pool. The byte array literals used in the transaction.
    #[proptest(strategy = "vec(any::<ByteArray>(), 0..=params)")]
    pub byte_array_pool: ByteArrayPool,
    /// Address pool. The address literals used in the module. Those include literals for
    /// code references (`ModuleHandle`).
    #[proptest(strategy = "vec(any::<AccountAddress>(), 0..=params)")]
    pub address_pool: AddressPool,

    /// The main (script) to execute.
    #[proptest(strategy = "any_with::<FunctionDefinition>(params)")]
    pub main: FunctionDefinition,
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
    pub fn into_module(self) -> CompiledModule {
        CompiledModule(self.0.into_module())
    }
}

impl CompiledScriptMut {
    /// Converts this instance into `CompiledScript` after verifying it for basic internal
    /// consistency. This includes bounds checks but no others.
    pub fn freeze(self) -> Result<CompiledScript, Vec<VerificationError>> {
        let fake_module = self.into_module();
        Ok(fake_module.freeze()?.into_script())
    }

    /// Converts a `CompiledScriptMut` to a `CompiledModule` for code that wants a uniform view
    /// of both.
    pub fn into_module(self) -> CompiledModuleMut {
        CompiledModuleMut {
            module_handles: self.module_handles,
            struct_handles: self.struct_handles,
            function_handles: self.function_handles,

            type_signatures: self.type_signatures,
            function_signatures: self.function_signatures,
            locals_signatures: self.locals_signatures,

            string_pool: self.string_pool,
            byte_array_pool: self.byte_array_pool,
            address_pool: self.address_pool,

            struct_defs: vec![],
            field_defs: vec![],
            function_defs: vec![self.main],
        }
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
    /// Handles to external modules and self at position 0.
    pub module_handles: Vec<ModuleHandle>,
    /// Handles to external and internal types.
    pub struct_handles: Vec<StructHandle>,
    /// Handles to external and internal functions.
    pub function_handles: Vec<FunctionHandle>,

    /// Type pool. A definition for all types used in the module.
    pub type_signatures: TypeSignaturePool,
    /// Function signature pool. Represents all function signatures defined or used in
    /// the module.
    pub function_signatures: FunctionSignaturePool,
    /// Locals signature pool. The signature for all locals of the functions defined in
    /// the module.
    pub locals_signatures: LocalsSignaturePool,

    /// String pool. All literals and identifiers used in the module.
    pub string_pool: StringPool,
    /// ByteArray pool. The byte array literals used in the module.
    pub byte_array_pool: ByteArrayPool,
    /// Address pool. The address literals used in the module. Those include literals for
    /// code references (`ModuleHandle`).
    pub address_pool: AddressPool,

    /// Types defined in this module.
    pub struct_defs: Vec<StructDefinition>,
    /// Fields defined on types in this module.
    pub field_defs: Vec<FieldDefinition>,
    /// Function defined in this module.
    pub function_defs: Vec<FunctionDefinition>,
}

// Need a custom implementation of Arbitrary because as of proptest-derive 0.1.1, the derivation
// doesn't work for structs with more than 10 fields.
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
            (
                vec(any::<TypeSignature>(), 0..=size),
                vec(any_with::<FunctionSignature>(size), 0..=size),
                vec(any_with::<LocalsSignature>(size), 0..=size),
            ),
            (
                vec(any::<String>(), 0..=size),
                vec(any::<ByteArray>(), 0..=size),
                vec(any::<AccountAddress>(), 0..=size),
            ),
            (
                vec(any::<StructDefinition>(), 0..=size),
                vec(any::<FieldDefinition>(), 0..=size),
                vec(any_with::<FunctionDefinition>(size), 0..=size),
            ),
        )
            .prop_map(
                |(
                    (module_handles, struct_handles, function_handles),
                    (type_signatures, function_signatures, locals_signatures),
                    (string_pool, byte_array_pool, address_pool),
                    (struct_defs, field_defs, function_defs),
                )| {
                    CompiledModuleMut {
                        module_handles,
                        struct_handles,
                        function_handles,
                        type_signatures,
                        function_signatures,
                        locals_signatures,
                        string_pool,
                        byte_array_pool,
                        address_pool,
                        struct_defs,
                        field_defs,
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
            IndexKind::StructDefinition => self.struct_defs.len(),
            IndexKind::FieldDefinition => self.field_defs.len(),
            IndexKind::FunctionDefinition => self.function_defs.len(),
            IndexKind::TypeSignature => self.type_signatures.len(),
            IndexKind::FunctionSignature => self.function_signatures.len(),
            IndexKind::LocalsSignature => self.locals_signatures.len(),
            IndexKind::StringPool => self.string_pool.len(),
            IndexKind::ByteArrayPool => self.byte_array_pool.len(),
            IndexKind::AddressPool => self.address_pool.len(),
            // XXX these two don't seem to belong here
            other @ IndexKind::LocalPool | other @ IndexKind::CodeDefinition => {
                panic!("invalid kind for count: {:?}", other)
            }
        }
    }

    /// Converts this instance into `CompiledModule` after verifying it for basic internal
    /// consistency. This includes bounds checks but no others.
    pub fn freeze(self) -> Result<CompiledModule, Vec<VerificationError>> {
        let errors = BoundsChecker::new(&self).verify();
        if errors.is_empty() {
            Ok(CompiledModule(self))
        } else {
            Err(errors)
        }
    }
}

impl CompiledModule {
    /// By convention, the index of the module being implemented is 0.
    pub const IMPLEMENTED_MODULE_INDEX: u16 = 0;

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
        self.as_inner().kind_count(kind)
    }

    /// Returns the code key of `module_handle`
    pub fn module_id_for_handle(&self, module_handle: &ModuleHandle) -> ModuleId {
        ModuleId::new(
            *self.address_at(module_handle.address),
            self.string_at(module_handle.name).to_string(),
        )
    }

    /// Returns the code key of `self`
    pub fn self_id(&self) -> ModuleId {
        self.module_id_for_handle(self.self_handle())
    }

    /// This function should only be called on an instance of CompiledModule obtained by invoking
    /// into_module on some instance of CompiledScript. This function is the inverse of
    /// into_module, i.e., script.into_module().into_script() == script.
    pub fn into_script(self) -> CompiledScript {
        let mut inner = self.into_inner();
        let main = inner.function_defs.remove(0);
        CompiledScript(CompiledScriptMut {
            module_handles: inner.module_handles,
            struct_handles: inner.struct_handles,
            function_handles: inner.function_handles,

            type_signatures: inner.type_signatures,
            function_signatures: inner.function_signatures,
            locals_signatures: inner.locals_signatures,

            string_pool: inner.string_pool,
            byte_array_pool: inner.byte_array_pool,
            address_pool: inner.address_pool,

            main,
        })
    }
}
