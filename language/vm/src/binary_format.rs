// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//
// Revised binary format changes.
// One of the objective is to create a model for an interpreter to be efficient.
// We want to also maximize type information we have at binding sites (type, function, fields)
// And we want to be compact.
//

// The pool of Identifier
pub type IdentifierPool = Vec<Identifier>;
// the pool of signatures, single pool where every signature is serialized as a single blob.
// Argument signatures and return signatures are split in 2 blobs (see FunctionHandle)
pub type Signature = Vec<Type>;
pub type SignaturePool = Vec<Signature>;

//
// next 2 tables could be replaced by the constant pool (described at the bottom of this file)

// The pool of byte array - Vector<u8> in Move
pub type U8VectorPool = Vec<Vec<u8>>;
// The address pool
pub type AddressPool = Vec<AccountAddress>;

//
// Scripts and Modules
//


// Scripts and Modules have a common "section" expressed in Rust.
// The Rust representation may vary... don't know...
// The split is entirely an abstraction invention because at binary format we only have
// tables. The order of tables can be anything.

// Common section
pub struct CompiledHeader {
    // handles are references to definitions within and cross modules
    pub module_handles: Vec<ModuleHandle>,
    pub struct_handles: Vec<StructHandle>,
    pub function_handles: Vec<FunctionHandle>,
    pub field_handles: Vec<FieldHandle>,
    // instantiations are references to a *generic* handle and and its instantiation, whether
    // partial or complete
    pub struct_instantiations: Vec<StructInstantiation>,
    pub function_instantiations: Vec<FunctionInstantiation>,
    pub field_instantiations: Vec<FieldInstantiation>,
    // common blob tables
    pub signatures: SignaturePool,
    pub identifiers: IdentifierPool,
    // constants
    pub byte_array_pool: U8VectorPool,
    pub address_pool: AddressPool,
}

// In `CompiledScript` we replace the FunctionDefinition for main(..) with broken up data
// to set up the call frame and define/verify main signature and arguments
pub struct CompiledScript {
    pub header: CompiledHeader,

    // Below should be enough information to materialize a
    // main<Cup<T>, W>(arg1, ..., argn) {}
    // with args of any type (including generic types) that the VM (verification
    // and runtime) can use.
    // There is a verification step here that is *purely* runtime.
    // The client is passing the type arguments and you can imagine restrictions
    // given our resource semantic.

    // function arguments signature, main is void on return
    pub arg_types: SignatureIndex,
    // type parameter for the script
    pub type_params: Vec<Kind>,
    // code
    pub main: CodeUnit,
}

// A CompiledModule is what it used to be but you will notice the removal of `FieldDefinition`s.
// Fields are now serialized with the `StructDefinition` which changes verification around fields.
// (see `StructDefinition` for details)
pub struct CompiledModule {
    pub header: CompiledHeader,

    pub struct_defs: Vec<StructDefinition>,
    pub function_defs: Vec<FunctionDefinition>,
}

//
// Basics...
//

pub enum Kind {
    All,
    Resource,
    Unrestricted,
}

// Any expressible type in Move
pub enum Type {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Vector(Box<Type>),
    Struct(StructHandleIndex),
    // a complete or partial instantiation of a generic type
    StructInstantiation(StructInstantiationIndex),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
    // a type argument provided by the definition (via instantiation)
    TypeParameter(TypeParameterIndex),
}

//
// Handles
//


pub struct ModuleHandle {
    pub address: AddressPoolIndex,
    pub name: IdentifierIndex,
}


pub struct StructHandle {
    pub module: ModuleHandleIndex,
    pub name: IdentifierIndex,
    pub is_nominal_resource: bool,
    pub type_params: Vec<Kind>,
}

// FunctionHandles inlines the signature as indexes into the sig table
pub struct FunctionHandle {
    pub module: ModuleHandleIndex,
    pub name: IdentifierIndex,
    pub arg_types: SignatureIndex,
    pub return_types: SignatureIndex,
    pub type_params: Vec<Kind>,
}

// A field access info (owner type and offset)
pub struct FieldHandle {
    owner: StructDefinitionIndex,
    field: FieldOffset,
}

//
// Instantiations
//
// Instantiations point to a generic handle and its instantiation.
// The instantiation can be partial.
// So, for example, `S<T, W>`, `S<u8, bool>`, `S<T, u8>`, `S<X<T>, address>` are all
// `StructInstantiation`s

// A complete or partial instantiation of a generic struct
pub struct StructInstantiation {
    pub handle: StructHandleIndex,
    pub type_actuals: SignatureIndex,
}

// A complete or partial instantiation of a function
pub struct FunctionInstantiation {
    pub handle: FunctionHandleIndex,
    pub type_actuals: SignatureIndex,
}

// A complete or partial instantiation of a field (or the type of it).
// A `FieldInstantiation` points to a generic `FieldHandle` and the instantiation
// of the owner type.
// E.g. for `S<u8, bool>.f` where `f` is a field of any type, `instantiation`
// would be `[u8, boo]`
pub struct FieldInstantiation {
    pub handle: FieldHandleIndex,
    pub type_actuals: SignatureIndex,
}


//
// Definitions
//

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructDefinition {
    pub struct_handle: StructHandleIndex,
    pub layout: StructLayout,
}

// Fields are inlined and each struct carries the fields it owns.
// The instructions that use fields ([imm]borrow_field) now have to specify
// the type that owns the field and the field index.
// For example, if you are referring to a field of `Cup<T>` instantiated
// let c: cup<int> = ...;
// let tea_type = &c.f // f typed T
// here we want to put in the instruction borrow_field (Cup<int>, 1) // assuming f is field 1
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StructLayout {
    Native,
    Declared(Vec<FieldDefinition>),
}

// no reason to carry the owner in the FieldDefinition now, as it is implicit
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDefinition {
    pub name: IdentifierIndex,
    pub signature: Type,
}

// we want a better way to define native functions. That encoding is in `Code` which can
// be native or move bytecoces
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FunctionDefinition {
    pub function: FunctionHandleIndex,
    pub flags: u8, // this is just the `public` flag now
    pub acquires_global_resources: Vec<StructDefinitionIndex>,
    pub code: Code,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Code {
    Native,
    Declared(CodeUnit),
}

pub struct CodeUnit {
    // ... max_stack_size is gone...

    // locals carry only the locals signature without the arguments.
    // However there is a single instruction that access locals uniformly,
    // that is, as if there was a single vector of arguments + locals.
    // The compiler (and generally clients) are supposed to do the math properly
    pub locals: SignatureIndex,
    pub code: Vec<Bytecode>,
}

//
// Bytecodes
//

// The main issue to digest here is that every bytecode that deals with generic is now
// split in 2 versions. The non generic and the generic. That has some advantages
// we have talked about and we may touch on more
pub enum Bytecode {
    Pop,
    Ret,
    Nop, // should we have one?

    // the branch instructions pending a structured control flow model we may want to provide
    BrTrue(CodeOffset),
    BrFalse(CodeOffset),
    Branch(CodeOffset),

    // load value on stack
    LdU8(u8),
    LdU64(u64),
    LdU128(u128),
    LdU8Vector(U8VectorPoolIndex),
    LdAddr(AddressPoolIndex),
    LdTrue,
    LdFalse,

    // random operators
    CastU8,
    CastU64,
    CastU128,
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    Shl,
    Shr,
    BitOr,
    BitAnd,
    Xor,
    Or,
    And,
    Not,
    Lt,
    Gt,
    Le,
    Ge,

    // system ops
    Abort,
    GetTxnSenderAddress,

    // locals ops
    CopyLoc(LocalIndex),
    MoveLoc(LocalIndex),
    StLoc(LocalIndex),
    MutBorrowLoc(LocalIndex),
    ImmBorrowLoc(LocalIndex),

    // reference ops
    ReadRef,
    WriteRef,
    FreezeRef,

    // equality ops, reference and not
    Eq,
    Neq,

    // calls
    Call(FunctionHandleIndex),
    CallGeneric(FunctionInstantiationIndex),

    // fields
    MutBorrowField(FieldHandleIndex),
    MutBorrowFieldGeneric(FieldInstantiationIndex),
    ImmBorrowField(FieldHandleIndex),
    ImmBorrowFieldGeneric(FieldInstantiationIndex),

    // type ops
    Pack(StructDefinitionIndex),
    PackGeneric(StructInstantiationIndex),
    Unpack(StructDefinitionIndex),
    UnpackGeneric(StructInstantiationIndex),
    MutBorrowGlobal(StructDefinitionIndex),
    MutBorrowGlobalGeneric(StructInstantiationIndex),
    ImmBorrowGlobal(StructDefinitionIndex),
    ImmBorrowGlobalGeneric(StructInstantiationIndex),
    Exists(StructDefinitionIndex),
    ExistsGeneric(StructInstantiationIndex),
    MoveFrom(StructDefinitionIndex),
    MoveFromGeneric(StructInstantiationIndex),
    MoveToSender(StructDefinitionIndex),
    MoveToSenderGeneric(StructInstantiationIndex),
}

//
// Constants (possible story)
//

// A ConstantPool will replace all data pools (address, byte_array)
// with a more general constant pool that would allow a mix of primitive types and
// Vectors of them. Types can and should be restricted to allow for only a subset
// of types.
pub struct ConstantPool {
    type_: Type,
    data: Vec<u8>,
}

// A declared named constant, e.g. `const ONE = 1;`
// Those are the constants exposed by this module.
pub struct Constant {
    name: IdentifierIndex,
    data: ConstantPoolIndex,
}

// A reference to a named constant used by the LdNamedConst bytecode.
// The constant may be within this module or outside of it.
pub struct ConstantHandle {
    module: ModuleHandleIndex,
    name: IdentifierIndex,
    data: ConstantPoolIndex,
}

// Constant bytecodes
// pub enum Bytecode {
//   ...
//   // load constant data without a name
//   LdConst(ConstantPoolIndex),
//   // load a named constant
//   LdNamedConst(ConstantHandleIndex),
//   ...
// }