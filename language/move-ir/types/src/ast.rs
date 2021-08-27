// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    location::*,
    spec_language_ast::{Condition, Invariant, SyntheticDefinition},
};
use move_core_types::{
    account_address::AccountAddress, identifier::Identifier, language_storage::ModuleId,
    value::MoveValue,
};
use move_symbol_pool::Symbol;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
/// A set of Move modules and a Move transaction script
pub struct Program {
    /// The modules to publish
    pub modules: Vec<ModuleDefinition>,
    /// The transaction script to execute
    pub script: Script,
}

//**************************************************************************************************
// ScriptOrModule
//**************************************************************************************************

#[derive(Debug, Clone)]
/// A script or a module, used to represent the two types of transactions.
pub enum ScriptOrModule {
    /// The script to execute.
    Script(Script),
    /// The module to publish.
    Module(ModuleDefinition),
}

//**************************************************************************************************
// Script
//**************************************************************************************************

#[derive(Debug, Clone)]
/// The Move transaction script to be executed
pub struct Script {
    /// The dependencies of `main`, i.e. of the transaction script
    pub imports: Vec<ImportDefinition>,
    /// Explicit declaration of dependencies. If not provided, will be inferred based on given
    /// dependencies to the IR compiler
    pub explicit_dependency_declarations: Vec<ModuleDependency>,
    /// the constants that the module defines. Only a utility, the identifiers are not carried into
    /// the Move bytecode
    pub constants: Vec<Constant>,
    /// The transaction script's `main` procedure
    pub main: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

/// Newtype for a name of a module
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ModuleName(pub Symbol);

/// Newtype of the address + the module name
/// `addr.m`
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ModuleIdent {
    /// Name for the module. Will be unique among modules published under the same address
    pub name: ModuleName,
    /// Address that this module is published under
    pub address: AccountAddress,
}

/// A Move module
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDefinition {
    /// name and address of the module
    pub identifier: ModuleIdent,
    /// the module's friends
    pub friends: Vec<ModuleIdent>,
    /// the module's dependencies
    pub imports: Vec<ImportDefinition>,
    /// Explicit declaration of dependencies. If not provided, will be inferred based on given
    /// dependencies to the IR compiler
    pub explicit_dependency_declarations: Vec<ModuleDependency>,
    /// the structs (including resources) that the module defines
    pub structs: Vec<StructDefinition>,
    /// the constants that the script defines. Only a utility, the identifiers are not carried into
    /// the Move bytecode
    pub constants: Vec<Constant>,
    /// the procedure that the module defines
    pub functions: Vec<(FunctionName, Function)>,
    /// the synthetic, specification variables the module defines.
    pub synthetics: Vec<SyntheticDefinition>,
}

/// Explicitly given dependency
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDependency {
    /// Qualified identifer of the dependency
    pub name: ModuleName,
    /// The structs (including resources) that the dependency defines
    pub structs: Vec<StructDependency>,
    /// The signatures of functions that the dependency defines
    pub functions: Vec<FunctionDependency>,
}

//**************************************************************************************************
// Imports
//**************************************************************************************************

/// A dependency/import declaration
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportDefinition {
    /// the dependency
    /// `addr.m`
    pub ident: ModuleIdent,
    /// the alias for that dependency
    /// `m`
    pub alias: ModuleName,
}

//**************************************************************************************************
// Vars
//**************************************************************************************************

/// Newtype for a variable/local
#[derive(Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Var_(pub Symbol);

/// The type of a variable with a location
pub type Var = Spanned<Var_>;

/// New type that represents a type variable. Used to declare type formals & reference them.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeVar_(pub Symbol);

/// The type of a type variable with a location.
pub type TypeVar = Spanned<TypeVar_>;

//**************************************************************************************************
// Abilities
//**************************************************************************************************

/// The abilities of a type. Analogous to `move_binary_format::file_format::Ability`.
#[derive(Debug, Clone, Eq, Copy, Hash, Ord, PartialEq, PartialOrd)]
pub enum Ability {
    /// Allows values of types with this ability to be copied
    Copy,
    /// Allows values of types with this ability to be dropped or if left in a local at return
    Drop,
    /// Allows values of types with this ability to exist inside a struct in global storage
    Store,
    /// Allows the type to serve as a key for global storage operations
    Key,
}
//**************************************************************************************************
// Types
//**************************************************************************************************

/// The type of a single value
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// `address`
    Address,
    /// `signer`
    Signer,
    /// `u8`
    U8,
    /// `u64`
    U64,
    /// `u128`
    U128,
    /// `bool`
    Bool,
    /// `vector`
    Vector(Box<Type>),
    /// A module defined struct
    Struct(QualifiedStructIdent, Vec<Type>),
    /// A reference type, the bool flag indicates whether the reference is mutable
    Reference(bool, Box<Type>),
    /// A type parameter
    TypeParameter(TypeVar_),
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

/// Identifier for a struct definition. Tells us where to look in the storage layer to find the
/// code associated with the interface
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct QualifiedStructIdent {
    /// Module name and address in which the struct is contained
    pub module: ModuleName,
    /// Name for the struct class. Should be unique among structs published under the same
    /// module+address
    pub name: StructName,
}

/// The field newtype
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Field_(pub Symbol);

/// A field coupled with source location information
pub type Field = Spanned<Field_>;

/// A fully-qualified field identifier.
///
/// Rather than simply referring to a field 'f' with a single identifier and
/// relying on type inference to determine the type of the struct being
/// accessed, this type refers to the field 'f' on the explicit struct type
/// 'S<T>' -- that is, 'S<T>::f'.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldIdent_ {
    /// The name of the struct type on which the field is declared.
    pub struct_name: StructName,
    /// For generic struct types, the type parameters used to instantiate the
    /// struct type (this is an empty vector for non-generic struct types).
    pub type_actuals: Vec<Type>,
    /// The name of the field.
    pub field: Field,
}

pub type FieldIdent = Spanned<FieldIdent_>;

/// A field map
pub type Fields<T> = Vec<(Field, T)>;

/// Newtype for the name of a struct
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StructName(pub Symbol);

/// A struct type parameter with its constraints and whether it's declared as phantom.
pub type StructTypeParameter = (bool, TypeVar, BTreeSet<Ability>);

/// A Move struct
#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition_ {
    /// The declared abilities for the struct
    pub abilities: BTreeSet<Ability>,
    /// Human-readable name for the struct that also serves as a nominal type
    pub name: StructName,
    /// The list of formal type arguments
    pub type_formals: Vec<StructTypeParameter>,
    /// the fields each instance has
    pub fields: StructDefinitionFields,
    /// the invariants for this struct
    pub invariants: Vec<Invariant>,
}
/// The type of a StructDefinition along with its source location information
pub type StructDefinition = Spanned<StructDefinition_>;

/// An explicit struct dependency
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructDependency {
    /// The declared abilities for the struct
    pub abilities: BTreeSet<Ability>,
    /// Human-readable name for the struct that also serves as a nominal type
    pub name: StructName,
    /// The list of formal type arguments
    pub type_formals: Vec<StructTypeParameter>,
}

/// The fields of a Move struct definition
#[derive(Clone, Debug, PartialEq)]
pub enum StructDefinitionFields {
    /// The fields are declared
    Move { fields: Fields<Type> },
    /// The struct is a type provided by the VM
    Native,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

/// Newtype for the name of a constant
#[derive(Debug, Serialize, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct ConstantName(pub Symbol);

/// A constant declaration in a module or script
#[derive(Clone, Debug, PartialEq)]
pub struct Constant {
    /// The constant's name. Not carried through to the Move bytecode
    pub name: ConstantName,
    /// The type of the constant's value
    pub signature: Type,
    /// The constant's value
    pub value: MoveValue,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

/// Newtype for the name of a function
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct FunctionName(pub Symbol);

/// The signature of a function
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    /// Possibly-empty list of (formal name, formal type) pairs. Names are unique.
    pub formals: Vec<(Var, Type)>,
    /// Optional return types
    pub return_type: Vec<Type>,
    /// Possibly-empty list of type parameters and their constraints
    pub type_formals: Vec<(TypeVar, BTreeSet<Ability>)>,
}

/// An explicit function dependency
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionDependency {
    /// Name of the function dependency
    pub name: FunctionName,
    /// Signature of the function dependency
    pub signature: FunctionSignature,
}

/// Public or internal modifier for a procedure
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum FunctionVisibility {
    /// The procedure can be invoked anywhere
    /// `public`
    Public,
    /// The procedure can only be invoked from a script context
    /// `public(script)`
    Script,
    /// The procedure can be invoked internally as well as by modules in the friend list
    /// `public(friend)`
    Friend,
    /// The procedure can be invoked only internally
    /// `<no modifier>`
    Internal,
}

/// The body of a Move function
#[derive(PartialEq, Debug, Clone)]
pub enum FunctionBody {
    /// The body is declared
    /// `locals` are all of the declared locals
    /// `code` is the code that defines the procedure
    Move {
        locals: Vec<(Var, Type)>,
        code: Block_,
    },
    Bytecode {
        locals: Vec<(Var, Type)>,
        code: BytecodeBlocks,
    },
    /// The body is provided by the runtime
    Native,
}

/// A Move function/procedure
#[derive(PartialEq, Debug, Clone)]
pub struct Function_ {
    /// The visibility
    pub visibility: FunctionVisibility,
    /// The type signature
    pub signature: FunctionSignature,
    /// List of nominal resources (declared in this module) that the procedure might access
    /// Either through: BorrowGlobal, MoveFrom, or transitively through another procedure
    /// This list of acquires grants the borrow checker the ability to statically verify the safety
    /// of references into global storage
    pub acquires: Vec<StructName>,
    /// List of specifications for the Move prover (experimental)
    pub specifications: Vec<Condition>,
    /// The code for the procedure
    pub body: FunctionBody,
}

/// The type of a Function coupled with its source location information.
pub type Function = Spanned<Function_>;

//**************************************************************************************************
// Statements
//**************************************************************************************************

/// Builtin "function"-like operators that often have a signature not expressable in the
/// type system and/or have access to some runtime/storage context
#[derive(Debug, PartialEq, Clone)]
pub enum Builtin {
    /// Check if there is a struct object (`StructName` resolved by current module) associated with
    /// the given address
    Exists(StructName, Vec<Type>),
    /// Get a reference to the resource(`StructName` resolved by current module) associated
    /// with the given address
    BorrowGlobal(bool, StructName, Vec<Type>),

    /// Remove a resource of the given type from the account with the given address
    MoveFrom(StructName, Vec<Type>),
    /// Publish an instantiated struct object into signer's (signer is the first arg) account.
    MoveTo(StructName, Vec<Type>),

    /// Pack a vector fix a fixed number of elements. Zero elements means an empty vector.
    VecPack(Vec<Type>, u64),
    /// Get the length of a vector
    VecLen(Vec<Type>),
    /// Acquire an immutable reference to the element at a given index of the vector
    VecImmBorrow(Vec<Type>),
    /// Acquire a mutable reference to the element at a given index of the vector
    VecMutBorrow(Vec<Type>),
    /// Push an element to the end of the vector
    VecPushBack(Vec<Type>),
    /// Pop and return an element from the end of the vector
    VecPopBack(Vec<Type>),
    /// Destroy a vector of a fixed length. Zero length means destroying an empty vector.
    VecUnpack(Vec<Type>, u64),
    /// Swap the elements at twi indices in the vector
    VecSwap(Vec<Type>),

    /// Convert a mutable reference into an immutable one
    Freeze,

    /// Cast an integer into u8.
    ToU8,
    /// Cast an integer into u64.
    ToU64,
    /// Cast an integer into u128.
    ToU128,
}

/// Enum for different function calls
#[derive(Debug, PartialEq, Clone)]
pub enum FunctionCall_ {
    /// functions defined in the host environment
    Builtin(Builtin),
    /// The call of a module defined procedure
    ModuleFunctionCall {
        module: ModuleName,
        name: FunctionName,
        type_actuals: Vec<Type>,
    },
}
/// The type for a function call and its location
pub type FunctionCall = Spanned<FunctionCall_>;

/// Enum for Move lvalues
#[derive(Debug, Clone, PartialEq)]
pub enum LValue_ {
    /// `x`
    Var(Var),
    /// `*e`
    Mutate(Exp),
    /// `_`
    Pop,
}
pub type LValue = Spanned<LValue_>;

/// Enum for Move commands
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd_ {
    /// `l_1, ..., l_n = e`
    Assign(Vec<LValue>, Exp),
    /// `n { f_1: x_1, ... , f_j: x_j  } = e`
    Unpack(StructName, Vec<Type>, Fields<Var>, Box<Exp>),
    /// `abort e`
    Abort(Option<Box<Exp>>),
    /// `return e_1, ... , e_j`
    Return(Box<Exp>),
    /// `break`
    Break,
    /// `continue`
    Continue,
    Exp(Box<Exp>),
}
/// The type of a command with its location
pub type Cmd = Spanned<Cmd_>;

/// Struct defining an if statement
#[derive(Debug, PartialEq, Clone)]
pub struct IfElse {
    /// the if's condition
    pub cond: Exp,
    /// the block taken if the condition is `true`
    pub if_block: Block,
    /// the block taken if the condition is `false`
    pub else_block: Option<Block>,
}

/// Struct defining a while statement
#[derive(Debug, PartialEq, Clone)]
pub struct While {
    /// The condition for a while statement
    pub cond: Exp,
    /// The block taken if the condition is `true`
    pub block: Block,
}

/// Struct defining a loop statement
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    /// The body of the loop
    pub block: Block,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Statement {
    /// `c;`
    CommandStatement(Cmd),
    /// `if (e) { s_1 } else { s_2 }`
    IfElseStatement(IfElse),
    /// `while (e) { s }`
    WhileStatement(While),
    /// `loop { s }`
    LoopStatement(Loop),
    /// no-op that eases parsing in some places
    EmptyStatement,
}

#[derive(Debug, PartialEq, Clone)]
/// `{ s }`
pub struct Block_ {
    /// The statements that make up the block
    pub stmts: VecDeque<Statement>,
}

/// The type of a Block coupled with source location information.
pub type Block = Spanned<Block_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

/// Bottom of the value hierarchy. These values can be trivially copyable and stored in statedb as a
/// single entry.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CopyableVal_ {
    /// An address in the global storage
    Address(AccountAddress),
    /// An unsigned 8-bit integer
    U8(u8),
    /// An unsigned 64-bit integer
    U64(u64),
    /// An unsigned 128-bit integer
    U128(u128),
    /// true or false
    Bool(bool),
    /// `b"<bytes>"`
    ByteArray(Vec<u8>),
}

/// The type of a value and its location
pub type CopyableVal = Spanned<CopyableVal_>;

/// The type for fields and their bound expressions
pub type ExpFields = Fields<Exp>;

/// Enum for unary operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    /// Boolean negation
    Not,
}

/// Enum for binary operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    // u64 ops
    /// `+`
    Add,
    /// `-`
    Sub,
    /// `*`
    Mul,
    /// `%`
    Mod,
    /// `/`
    Div,
    /// `|`
    BitOr,
    /// `&`
    BitAnd,
    /// `^`
    Xor,
    /// `<<`
    Shl,
    /// `>>`
    Shr,

    // Bool ops
    /// `&&`
    And,
    /// `||`
    Or,

    // Compare Ops
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    Le,
    /// `>=`
    Ge,
    /// '..'  only used in specs
    Subrange,
}

/// Enum for all expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Exp_ {
    /// `*e`
    Dereference(Box<Exp>),
    /// `op e`
    UnaryExp(UnaryOp, Box<Exp>),
    /// `e_1 op e_2`
    BinopExp(Box<Exp>, BinOp, Box<Exp>),
    /// Wrapper to lift `CopyableVal` into `Exp`
    /// `v`
    Value(CopyableVal),
    /// Takes the given field values and instantiates the struct
    /// Returns a fresh `StructInstance` whose type and kind (resource or otherwise)
    /// as the current struct class (i.e., the class of the method we're currently executing).
    /// `n { f_1: e_1, ... , f_j: e_j }`
    Pack(StructName, Vec<Type>, ExpFields),
    /// `&e.f`, `&mut e.f`
    Borrow {
        /// mutable or not
        is_mutable: bool,
        /// the expression containing the reference
        exp: Box<Exp>,
        /// the field being borrowed
        field: FieldIdent,
    },
    /// `move(x)`
    Move(Var),
    /// `copy(x)`
    Copy(Var),
    /// `&x` or `&mut x`
    BorrowLocal(bool, Var),
    /// `f(e)` or `f(e_1, e_2, ..., e_j)`
    FunctionCall(FunctionCall, Box<Exp>),
    /// (e_1, e_2, e_3, ..., e_j)
    ExprList(Vec<Exp>),
}

/// The type for a `Exp_` and its location
pub type Exp = Spanned<Exp_>;

//**************************************************************************************************
// Bytecode
//**************************************************************************************************

pub type BytecodeBlocks = Vec<(BlockLabel, BytecodeBlock)>;
pub type BytecodeBlock = Vec<Bytecode>;

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockLabel(pub Symbol);

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NopLabel(pub Symbol);

#[derive(Debug, Clone, PartialEq)]
pub enum Bytecode_ {
    Pop,
    Ret,
    Nop(Option<NopLabel>),
    BrTrue(BlockLabel),
    BrFalse(BlockLabel),
    Branch(BlockLabel),
    LdU8(u8),
    LdU64(u64),
    LdU128(u128),
    CastU8,
    CastU64,
    CastU128,
    LdByteArray(Vec<u8>),
    LdAddr(AccountAddress),
    LdTrue,
    LdFalse,
    LdConst(ConstantName),
    CopyLoc(Var),
    MoveLoc(Var),
    StLoc(Var),
    Call(ModuleName, FunctionName, Vec<Type>),
    Pack(StructName, Vec<Type>),
    Unpack(StructName, Vec<Type>),
    ReadRef,
    WriteRef,
    FreezeRef,
    MutBorrowLoc(Var),
    ImmBorrowLoc(Var),
    MutBorrowField(StructName, Vec<Type>, Field),
    ImmBorrowField(StructName, Vec<Type>, Field),
    MutBorrowGlobal(StructName, Vec<Type>),
    ImmBorrowGlobal(StructName, Vec<Type>),
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    BitOr,
    BitAnd,
    Xor,
    Or,
    And,
    Not,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Abort,
    Exists(StructName, Vec<Type>),
    MoveFrom(StructName, Vec<Type>),
    MoveTo(StructName, Vec<Type>),
    Shl,
    Shr,
}
pub type Bytecode = Spanned<Bytecode_>;

//**************************************************************************************************
// impls
//**************************************************************************************************

fn get_external_deps(imports: &[ImportDefinition]) -> Vec<ModuleId> {
    let mut deps = HashSet::new();
    for dep in imports.iter() {
        let identifier = Identifier::new(dep.ident.name.0.as_str().to_owned()).unwrap();
        deps.insert(ModuleId::new(dep.ident.address, identifier));
    }
    deps.into_iter().collect()
}

impl Program {
    /// Create a new `Program` from modules and transaction script
    pub fn new(modules: Vec<ModuleDefinition>, script: Script) -> Self {
        Program { modules, script }
    }
}

impl Script {
    /// Create a new `Script` from the imports and the main function
    pub fn new(
        imports: Vec<ImportDefinition>,
        explicit_dependency_declarations: Vec<ModuleDependency>,
        constants: Vec<Constant>,
        main: Function,
    ) -> Self {
        Script {
            imports,
            explicit_dependency_declarations,
            constants,
            main,
        }
    }

    /// Accessor for the body of the 'main' procedure
    pub fn body(&self) -> &Block_ {
        match self.main.value.body {
            FunctionBody::Move { ref code, .. } => code,
            FunctionBody::Bytecode { .. } => panic!("Invalid body access on bytecode main()"),
            FunctionBody::Native => panic!("main() cannot be native"),
        }
    }

    /// Return a vector of `ModuleId` for the external dependencies.
    pub fn get_external_deps(&self) -> Vec<ModuleId> {
        get_external_deps(self.imports.as_slice())
    }
}

static SELF_MODULE_NAME: Lazy<Symbol> = Lazy::new(|| Symbol::from("Self"));

impl ModuleName {
    /// Name for the current module handle
    pub fn self_name() -> &'static str {
        SELF_MODULE_NAME.as_str()
    }

    /// Create a new `ModuleName` from `self_name`.
    pub fn module_self() -> Self {
        ModuleName(*SELF_MODULE_NAME)
    }
}

impl ModuleIdent {
    /// Creates a new fully qualified module identifier from the module name and the address at
    /// which it is published
    pub fn new(name: ModuleName, address: AccountAddress) -> Self {
        ModuleIdent { name, address }
    }

    /// Accessor for the name of the fully qualified module identifier
    pub fn name(&self) -> &ModuleName {
        &self.name
    }

    /// Accessor for the address at which the module is published
    pub fn address(&self) -> &AccountAddress {
        &self.address
    }
}

impl ModuleDefinition {
    /// Creates a new `ModuleDefinition` from its string name, dependencies, structs+resources,
    /// and procedures
    /// Does not verify the correctness of any internal properties of its elements
    pub fn new(
        identifier: ModuleIdent,
        friends: Vec<ModuleIdent>,
        imports: Vec<ImportDefinition>,
        explicit_dependency_declarations: Vec<ModuleDependency>,
        structs: Vec<StructDefinition>,
        constants: Vec<Constant>,
        functions: Vec<(FunctionName, Function)>,
        synthetics: Vec<SyntheticDefinition>,
    ) -> Self {
        ModuleDefinition {
            identifier,
            friends,
            imports,
            explicit_dependency_declarations,
            structs,
            constants,
            functions,
            synthetics,
        }
    }

    /// Return a vector of `ModuleId` for the external dependencies.
    pub fn get_external_deps(&self) -> Vec<ModuleId> {
        get_external_deps(self.imports.as_slice())
    }
}

impl Ability {
    pub const COPY: &'static str = "copy";
    pub const DROP: &'static str = "drop";
    pub const STORE: &'static str = "store";
    pub const KEY: &'static str = "key";
}

impl Type {
    /// Creates a new struct type
    pub fn r#struct(ident: QualifiedStructIdent, type_actuals: Vec<Type>) -> Type {
        Type::Struct(ident, type_actuals)
    }

    /// Creates a new reference type from its mutability and underlying type
    pub fn reference(is_mutable: bool, t: Type) -> Type {
        Type::Reference(is_mutable, Box::new(t))
    }

    /// Creates a new address type
    pub fn address() -> Type {
        Type::Address
    }

    /// Creates a new u64 type
    pub fn u64() -> Type {
        Type::U64
    }

    /// Creates a new bool type
    pub fn bool() -> Type {
        Type::Bool
    }
}

impl QualifiedStructIdent {
    /// Creates a new StructType handle from the name of the module alias and the name of the struct
    pub fn new(module: ModuleName, name: StructName) -> Self {
        QualifiedStructIdent { module, name }
    }

    /// Accessor for the module alias
    pub fn module(&self) -> &ModuleName {
        &self.module
    }

    /// Accessor for the struct name
    pub fn name(&self) -> &StructName {
        &self.name
    }
}

impl ImportDefinition {
    /// Creates a new import definition from a module identifier and an optional alias
    /// If the alias is `None`, the alias will be a cloned copy of the identifiers module name
    pub fn new(ident: ModuleIdent, alias_opt: Option<ModuleName>) -> Self {
        let alias = match alias_opt {
            Some(alias) => alias,
            None => *ident.name(),
        };
        ImportDefinition { ident, alias }
    }
}

impl StructDefinition_ {
    /// Creates a new StructDefinition from the abilities, the string representation of the name,
    /// and the user specified fields, a map from their names to their types
    /// Does not verify the correctness of any internal properties, e.g. doesn't check that the
    /// fields do not have reference types
    pub fn move_declared(
        abilities: BTreeSet<Ability>,
        name: Symbol,
        type_formals: Vec<StructTypeParameter>,
        fields: Fields<Type>,
        invariants: Vec<Invariant>,
    ) -> Self {
        StructDefinition_ {
            abilities,
            name: StructName(name),
            type_formals,
            fields: StructDefinitionFields::Move { fields },
            invariants,
        }
    }

    /// Creates a new StructDefinition from the abilities, the string representation of the name,
    /// and the user specified fields, a map from their names to their types
    pub fn native(
        abilities: BTreeSet<Ability>,
        name: Symbol,
        type_formals: Vec<StructTypeParameter>,
    ) -> Self {
        StructDefinition_ {
            abilities,
            name: StructName(name),
            type_formals,
            fields: StructDefinitionFields::Native,
            invariants: vec![],
        }
    }
}

impl FunctionSignature {
    /// Creates a new function signature from the parameters and the return types
    pub fn new(
        formals: Vec<(Var, Type)>,
        return_type: Vec<Type>,
        type_parameters: Vec<(TypeVar, BTreeSet<Ability>)>,
    ) -> Self {
        FunctionSignature {
            formals,
            return_type,
            type_formals: type_parameters,
        }
    }
}

impl Function_ {
    /// Creates a new function declaration from the components of the function
    /// See the declaration of the struct `Function` for more details
    pub fn new(
        visibility: FunctionVisibility,
        formals: Vec<(Var, Type)>,
        return_type: Vec<Type>,
        type_parameters: Vec<(TypeVar, BTreeSet<Ability>)>,
        acquires: Vec<StructName>,
        specifications: Vec<Condition>,
        body: FunctionBody,
    ) -> Self {
        let signature = FunctionSignature::new(formals, return_type, type_parameters);
        Function_ {
            visibility,
            signature,
            acquires,
            specifications,
            body,
        }
    }
}

impl FunctionCall_ {
    /// Creates a `FunctionCall::ModuleFunctionCall` variant
    pub fn module_call(module: ModuleName, name: FunctionName, type_actuals: Vec<Type>) -> Self {
        FunctionCall_::ModuleFunctionCall {
            module,
            name,
            type_actuals,
        }
    }

    /// Creates a `FunctionCall::Builtin` variant with no location information
    pub fn builtin(bif: Builtin) -> FunctionCall {
        Spanned::unsafe_no_loc(FunctionCall_::Builtin(bif))
    }
}

impl Cmd_ {
    /// Creates a command that returns no values
    pub fn return_empty() -> Self {
        Cmd_::Return(Box::new(Spanned::unsafe_no_loc(Exp_::ExprList(vec![]))))
    }

    /// Creates a command that returns a single value
    pub fn return_(op: Exp) -> Self {
        Cmd_::Return(Box::new(op))
    }
}

impl IfElse {
    /// Creates an if-statement with no else branch
    pub fn if_block(cond: Exp, if_block: Block) -> Self {
        IfElse {
            cond,
            if_block,
            else_block: None,
        }
    }
    #[allow(clippy::self_named_constructors)]
    /// Creates an if-statement with an else branch
    pub fn if_else(cond: Exp, if_block: Block, else_block: Block) -> Self {
        IfElse {
            cond,
            if_block,
            else_block: Some(else_block),
        }
    }
}

impl Statement {
    /// Lifts a command into a statement
    pub fn cmd(c: Cmd) -> Self {
        Statement::CommandStatement(c)
    }

    /// Creates an `Statement::IfElseStatement` variant with no else branch
    pub fn if_block(cond: Exp, if_block: Block) -> Self {
        Statement::IfElseStatement(IfElse::if_block(cond, if_block))
    }

    /// Creates an `Statement::IfElseStatement` variant with an else branch
    pub fn if_else(cond: Exp, if_block: Block, else_block: Block) -> Self {
        Statement::IfElseStatement(IfElse::if_else(cond, if_block, else_block))
    }
}

impl Block_ {
    /// Creates a new block from the vector of statements
    pub fn new(stmts: Vec<Statement>) -> Self {
        Block_ {
            stmts: VecDeque::from(stmts),
        }
    }

    /// Creates an empty block
    pub fn empty() -> Self {
        Block_ {
            stmts: VecDeque::new(),
        }
    }
}

impl Exp_ {
    /// Creates a new address `Exp` with no location information
    pub fn address(addr: AccountAddress) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Value(Spanned::unsafe_no_loc(CopyableVal_::Address(
            addr,
        ))))
    }

    /// Creates a new value `Exp` with no location information
    pub fn value(b: CopyableVal_) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Value(Spanned::unsafe_no_loc(b)))
    }

    /// Creates a new u64 `Exp` with no location information
    pub fn u64(i: u64) -> Exp {
        Exp_::value(CopyableVal_::U64(i))
    }

    /// Creates a new bool `Exp` with no location information
    pub fn bool(b: bool) -> Exp {
        Exp_::value(CopyableVal_::Bool(b))
    }

    /// Creates a new bytearray `Exp` with no location information
    pub fn byte_array(buf: Vec<u8>) -> Exp {
        Exp_::value(CopyableVal_::ByteArray(buf))
    }

    /// Creates a new pack/struct-instantiation `Exp` with no location information
    pub fn instantiate(n: StructName, tys: Vec<Type>, s: ExpFields) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Pack(n, tys, s))
    }

    /// Creates a new binary operator `Exp` with no location information
    pub fn binop(lhs: Exp, op: BinOp, rhs: Exp) -> Exp {
        Spanned::unsafe_no_loc(Exp_::BinopExp(Box::new(lhs), op, Box::new(rhs)))
    }

    /// Creates a new `e+e` `Exp` with no location information
    pub fn add(lhs: Exp, rhs: Exp) -> Exp {
        Exp_::binop(lhs, BinOp::Add, rhs)
    }

    /// Creates a new `e-e` `Exp` with no location information
    pub fn sub(lhs: Exp, rhs: Exp) -> Exp {
        Exp_::binop(lhs, BinOp::Sub, rhs)
    }

    /// Creates a new `*e` `Exp` with no location information
    pub fn dereference(e: Exp) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Dereference(Box::new(e)))
    }

    /// Creates a new borrow field `Exp` with no location information
    pub fn borrow(is_mutable: bool, exp: Box<Exp>, field: FieldIdent) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Borrow {
            is_mutable,
            exp,
            field,
        })
    }

    /// Creates a new copy-local `Exp` with no location information
    pub fn copy(v: Var) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Copy(v))
    }

    /// Creates a new move-local `Exp` with no location information
    pub fn move_(v: Var) -> Exp {
        Spanned::unsafe_no_loc(Exp_::Move(v))
    }

    /// Creates a new function call `Exp` with no location information
    pub fn function_call(f: FunctionCall, e: Exp) -> Exp {
        Spanned::unsafe_no_loc(Exp_::FunctionCall(f, Box::new(e)))
    }

    pub fn expr_list(exps: Vec<Exp>) -> Exp {
        Spanned::unsafe_no_loc(Exp_::ExprList(exps))
    }
}

//**************************************************************************************************
// Trait impls
//**************************************************************************************************

impl Iterator for Script {
    type Item = Statement;

    fn next(&mut self) -> Option<Statement> {
        match self.main.value.body {
            FunctionBody::Move { ref mut code, .. } => code.stmts.pop_front(),
            FunctionBody::Bytecode { .. } => panic!("main() cannot currently be bytecode"),
            FunctionBody::Native => panic!("main() cannot be native code"),
        }
    }
}

impl PartialEq for Script {
    fn eq(&self, other: &Script) -> bool {
        self.imports == other.imports && self.main.value.body == other.main.value.body
    }
}

impl Iterator for Block_ {
    type Item = Statement;

    fn next(&mut self) -> Option<Statement> {
        self.stmts.pop_front()
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for TypeVar_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Ability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Ability::Copy => Ability::COPY,
                Ability::Drop => Ability::DROP,
                Ability::Store => Ability::STORE,
                Ability::Key => Ability::KEY,
            }
        )
    }
}

fn format_constraints(set: &BTreeSet<Ability>) -> String {
    set.iter()
        .map(|a| format!("{}", a))
        .collect::<Vec<_>>()
        .join(" + ")
}

impl fmt::Display for ScriptOrModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScriptOrModule::*;
        match self {
            Module(module_def) => write!(f, "{}", module_def),
            Script(script) => write!(f, "{}", script),
        }
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Script(")?;

        write!(f, "Imports(")?;
        write!(f, "{}", intersperse(&self.imports, ", "))?;
        writeln!(f, ")")?;

        writeln!(f, "Dependency(")?;
        for dependency in &self.explicit_dependency_declarations {
            writeln!(f, "{},", dependency)?;
        }
        writeln!(f, ")")?;

        writeln!(f, "Constants(")?;
        for constant in &self.constants {
            writeln!(f, "{};", constant)?;
        }
        writeln!(f, ")")?;

        write!(f, "Main(")?;
        write!(f, "{}", self.main)?;
        write!(f, ")")?;
        write!(f, ")")
    }
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{}", self.address, self.name)
    }
}

impl fmt::Display for ModuleDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module({}, ", self.identifier)?;

        writeln!(f, "Imports(")?;
        for import in &self.imports {
            writeln!(f, "{};", import)?;
        }
        writeln!(f, ")")?;

        writeln!(f, "Dependency(")?;
        for dependency in &self.explicit_dependency_declarations {
            writeln!(f, "{},", dependency)?;
        }
        writeln!(f, ")")?;

        writeln!(f, "Structs(")?;
        for struct_def in &self.structs {
            writeln!(f, "{}, ", struct_def)?;
        }
        writeln!(f, ")")?;

        writeln!(f, "Constants(")?;
        for constant in &self.constants {
            writeln!(f, "{};", constant)?;
        }
        writeln!(f, ")")?;

        writeln!(f, "Functions(")?;
        for (fun_name, fun) in &self.functions {
            writeln!(f, "({}, {}), ", fun_name, fun)?;
        }
        writeln!(f, ")")?;

        writeln!(f, ")")
    }
}

impl fmt::Display for ImportDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "import {} as {}", &self.ident, &self.alias)
    }
}

impl fmt::Display for ModuleDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dependency({}, ", &self.name)?;
        for sdep in &self.structs {
            writeln!(f, "{}, ", sdep)?
        }
        for fdep in &self.functions {
            writeln!(f, "{}, ", fdep)?
        }
        writeln!(f, ")")
    }
}

impl fmt::Display for StructDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StructDep({} {}{}",
            self.abilities
                .iter()
                .map(|a| format!("{}", a))
                .collect::<Vec<_>>()
                .join(" "),
            &self.name,
            format_struct_type_formals(&self.type_formals)
        )
    }
}

impl fmt::Display for FunctionDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FunctionDep({}{}", &self.name, &self.signature)
    }
}

impl fmt::Display for StructDefinition_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Struct({}{}, ",
            self.name,
            format_struct_type_formals(&self.type_formals)
        )?;
        match &self.fields {
            StructDefinitionFields::Move { fields } => writeln!(f, "{}", format_fields(fields))?,
            StructDefinitionFields::Native => writeln!(f, "{{native}}")?,
        }
        write!(f, ")")
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "const {}: {} = {:?}",
            &self.name.0, self.signature, self.value
        )
    }
}

impl fmt::Display for Function_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.signature, self.body)
    }
}

impl fmt::Display for Field_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FieldIdent_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.struct_name, self.field)
    }
}

impl fmt::Display for StructName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for ConstantName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionBody::Move {
                ref locals,
                ref code,
            } => {
                for (local, ty) in locals {
                    write!(f, "let {}: {};", local, ty)?;
                }
                writeln!(f, "{}", code)
            }
            FunctionBody::Bytecode { locals, code } => {
                write!(f, "locals: [")?;
                for (local, ty) in locals {
                    write!(f, "{}: {},", local, ty)?;
                }
                writeln!(f, "]")?;
                for (label, block) in code {
                    writeln!(f, "label {}:", &label.0)?;
                    for instr in block {
                        writeln!(f, "  {}", instr)?;
                    }
                }
                Ok(())
            }
            FunctionBody::Native => write!(f, "native"),
        }
    }
}

// TODO: This function should take an iterator instead.
fn intersperse<T: fmt::Display>(items: &[T], join: &str) -> String {
    // TODO: Any performance issues here? Could be O(n^2) if not optimized.
    items.iter().fold(String::new(), |acc, v| {
        format!("{acc}{join}{v}", acc = acc, join = join, v = v)
    })
}

fn format_fields<T: fmt::Display>(fields: &[(Field, T)]) -> String {
    fields.iter().fold(String::new(), |acc, (field, val)| {
        format!("{} {}: {},", acc, field.value, val)
    })
}

impl fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_fun_type_formals(&self.type_formals))?;
        write!(f, "(")?;
        for (v, ty) in self.formals.iter() {
            write!(f, "{}: {}, ", v, ty)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl fmt::Display for QualifiedStructIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.name)
    }
}

fn format_type_actuals(tys: &[Type]) -> String {
    if tys.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", intersperse(tys, ", "))
    }
}

fn format_fun_type_formals(formals: &[(TypeVar, BTreeSet<Ability>)]) -> String {
    if formals.is_empty() {
        "".to_string()
    } else {
        let formatted = formals
            .iter()
            .map(|(tv, abilities)| format!("{}: {}", tv.value, format_constraints(abilities)))
            .collect::<Vec<_>>();
        format!("<{}>", intersperse(&formatted, ", "))
    }
}

fn format_struct_type_formals(formals: &[StructTypeParameter]) -> String {
    if formals.is_empty() {
        "".to_string()
    } else {
        let formatted = formals
            .iter()
            .map(|(is_phantom, tv, abilities)| {
                format!(
                    "{}{}: {}",
                    if *is_phantom { "phantom " } else { "" },
                    tv.value,
                    format_constraints(abilities)
                )
            })
            .collect::<Vec<_>>();
        format!("<{}>", intersperse(&formatted, ", "))
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::U8 => write!(f, "u8"),
            Type::U64 => write!(f, "u64"),
            Type::U128 => write!(f, "u128"),
            Type::Bool => write!(f, "bool"),
            Type::Address => write!(f, "address"),
            Type::Signer => write!(f, "signer"),
            Type::Vector(ty) => write!(f, "vector<{}>", ty),
            Type::Struct(ident, tys) => write!(f, "{}{}", ident, format_type_actuals(tys)),
            Type::Reference(is_mutable, t) => {
                write!(f, "&{}{}", if *is_mutable { "mut " } else { "" }, t)
            }
            Type::TypeParameter(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Var_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Builtin::Exists(t, tys) => write!(f, "exists<{}{}>", t, format_type_actuals(tys)),
            Builtin::BorrowGlobal(mut_, t, tys) => {
                let mut_flag = if *mut_ { "_mut" } else { "" };
                write!(
                    f,
                    "borrow_global{}<{}{}>",
                    mut_flag,
                    t,
                    format_type_actuals(tys)
                )
            }
            Builtin::MoveFrom(t, tys) => write!(f, "move_from<{}{}>", t, format_type_actuals(tys)),
            Builtin::MoveTo(t, tys) => write!(f, "move_to<{}{}>", t, format_type_actuals(tys)),
            Builtin::VecPack(tys, num) => write!(f, "vec_pack_{}{}", num, format_type_actuals(tys)),
            Builtin::VecLen(tys) => write!(f, "vec_len{}", format_type_actuals(tys)),
            Builtin::VecImmBorrow(tys) => write!(f, "vec_imm_borrow{}", format_type_actuals(tys)),
            Builtin::VecMutBorrow(tys) => write!(f, "vec_mut_borrow{}", format_type_actuals(tys)),
            Builtin::VecPushBack(tys) => write!(f, "vec_push_back{}", format_type_actuals(tys)),
            Builtin::VecPopBack(tys) => write!(f, "vec_pop_back{}", format_type_actuals(tys)),
            Builtin::VecUnpack(tys, num) => {
                write!(f, "vec_unpack_{}{}", num, format_type_actuals(tys))
            }
            Builtin::VecSwap(tys) => write!(f, "vec_swap{}", format_type_actuals(tys)),
            Builtin::Freeze => write!(f, "freeze"),
            Builtin::ToU8 => write!(f, "to_u8"),
            Builtin::ToU64 => write!(f, "to_u64"),
            Builtin::ToU128 => write!(f, "to_u128"),
        }
    }
}

impl fmt::Display for FunctionCall_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionCall_::Builtin(fun) => write!(f, "{}", fun),
            FunctionCall_::ModuleFunctionCall {
                module,
                name,
                type_actuals,
            } => write!(
                f,
                "{}.{}{}",
                module,
                name,
                format_type_actuals(type_actuals)
            ),
        }
    }
}

impl fmt::Display for LValue_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LValue_::Var(x) => write!(f, "{}", x),
            LValue_::Mutate(e) => write!(f, "*{}", e),
            LValue_::Pop => write!(f, "_"),
        }
    }
}

impl fmt::Display for Cmd_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cmd_::Assign(var_list, e) => {
                if var_list.is_empty() {
                    write!(f, "{};", e)
                } else {
                    write!(f, "{} = ({});", intersperse(var_list, ", "), e)
                }
            }
            Cmd_::Unpack(n, tys, bindings, e) => write!(
                f,
                "{}{} {{ {} }} = {}",
                n,
                format_type_actuals(tys),
                bindings
                    .iter()
                    .fold(String::new(), |acc, (field, var)| format!(
                        "{} {} : {},",
                        acc, field, var
                    )),
                e
            ),
            Cmd_::Abort(None) => write!(f, "abort;"),
            Cmd_::Abort(Some(err)) => write!(f, "abort {};", err),
            Cmd_::Return(exps) => write!(f, "return {};", exps),
            Cmd_::Break => write!(f, "break;"),
            Cmd_::Continue => write!(f, "continue;"),
            Cmd_::Exp(e) => write!(f, "({});", e),
        }
    }
}

impl fmt::Display for IfElse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "if ({}) {{\n{:indent$}\n}}",
            self.cond,
            self.if_block,
            indent = 4
        )?;
        match self.else_block {
            None => Ok(()),
            Some(ref block) => write!(f, " else {{\n{:indent$}\n}}", block, indent = 4),
        }
    }
}

impl fmt::Display for While {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "while ({}) {{\n{:indent$}\n}}",
            self.cond,
            self.block,
            indent = 4
        )?;
        Ok(())
    }
}

impl fmt::Display for Loop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "loop {{\n{:indent$}\n}}", self.block, indent = 4)?;
        Ok(())
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::CommandStatement(cmd) => write!(f, "{}", cmd),
            Statement::IfElseStatement(if_else) => write!(f, "{}", if_else),
            Statement::WhileStatement(while_) => write!(f, "{}", while_),
            Statement::LoopStatement(loop_) => write!(f, "{}", loop_),
            Statement::EmptyStatement => write!(f, "<empty statement>"),
        }
    }
}

impl fmt::Display for Block_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in self.stmts.iter() {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for CopyableVal_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopyableVal_::U8(v) => write!(f, "{}u8", v),
            CopyableVal_::U64(v) => write!(f, "{}", v),
            CopyableVal_::U128(v) => write!(f, "{}u128", v),
            CopyableVal_::Bool(v) => write!(f, "{}", v),
            CopyableVal_::ByteArray(v) => write!(f, "0b{}", hex::encode(v)),
            CopyableVal_::Address(v) => write!(f, "0x{}", hex::encode(v)),
        }
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                UnaryOp::Not => "!",
            }
        )
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Mod => "%",
                BinOp::Div => "/",
                BinOp::BitOr => "|",
                BinOp::BitAnd => "&",
                BinOp::Xor => "^",
                BinOp::Shl => "<<",
                BinOp::Shr => ">>",

                // Bool ops
                BinOp::Or => "||",
                BinOp::And => "&&",

                // Compare Ops
                BinOp::Eq => "==",
                BinOp::Neq => "!=",
                BinOp::Lt => "<",
                BinOp::Gt => ">",
                BinOp::Le => "<=",
                BinOp::Ge => ">=",
                BinOp::Subrange => "..",
            }
        )
    }
}

impl fmt::Display for Exp_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exp_::Dereference(e) => write!(f, "*({})", e),
            Exp_::UnaryExp(o, e) => write!(f, "({}{})", o, e),
            Exp_::BinopExp(e1, o, e2) => write!(f, "({} {} {})", o, e1, e2),
            Exp_::Value(v) => write!(f, "{}", v),
            Exp_::Pack(n, tys, s) => write!(
                f,
                "{}{}{{{}}}",
                n,
                format_type_actuals(tys),
                s.iter().fold(String::new(), |acc, (field, op)| format!(
                    "{} {} : {},",
                    acc, field, op,
                ))
            ),
            Exp_::Borrow {
                is_mutable,
                exp,
                field,
            } => write!(
                f,
                "&{}{}.{}",
                if *is_mutable { "mut " } else { "" },
                exp,
                field
            ),
            Exp_::Move(v) => write!(f, "move({})", v),
            Exp_::Copy(v) => write!(f, "copy({})", v),
            Exp_::BorrowLocal(is_mutable, v) => {
                write!(f, "&{}{}", if *is_mutable { "mut " } else { "" }, v)
            }
            Exp_::FunctionCall(func, e) => write!(f, "{}({})", func, e),
            Exp_::ExprList(exps) => {
                if exps.is_empty() {
                    write!(f, "()")
                } else {
                    write!(f, "({})", intersperse(exps, ", "))
                }
            }
        }
    }
}

impl fmt::Display for Bytecode_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bytecode_::Pop => write!(f, "Pop"),
            Bytecode_::Ret => write!(f, "Ret"),
            Bytecode_::Nop(None) => write!(f, "Nop"),
            Bytecode_::Nop(Some(s)) => write!(f, "Nop {}", &s.0),
            Bytecode_::BrTrue(lbl) => write!(f, "BrTrue {}", &lbl.0),
            Bytecode_::BrFalse(lbl) => write!(f, "BrFalse {}", &lbl.0),
            Bytecode_::Branch(lbl) => write!(f, "Branch {}", &lbl.0),
            Bytecode_::LdU8(u) => write!(f, "LdU8 {}", u),
            Bytecode_::LdU64(u) => write!(f, "LdU64 {}", u),
            Bytecode_::LdU128(u) => write!(f, "LdU128 {}", u),
            Bytecode_::CastU8 => write!(f, "CastU8"),
            Bytecode_::CastU64 => write!(f, "CastU64"),
            Bytecode_::CastU128 => write!(f, "CastU128"),
            Bytecode_::LdByteArray(b) => write!(f, "LdByteArray 0b{}", hex::encode(b)),
            Bytecode_::LdAddr(a) => write!(f, "LdAddr {}", a),
            Bytecode_::LdTrue => write!(f, "LdTrue"),
            Bytecode_::LdFalse => write!(f, "LdFalse"),
            Bytecode_::LdConst(n) => write!(f, "LdConst({})", &n.0),
            Bytecode_::CopyLoc(v) => write!(f, "CopyLoc {}", v),
            Bytecode_::MoveLoc(v) => write!(f, "MoveLoc {}", v),
            Bytecode_::StLoc(v) => write!(f, "StLoc {}", v),
            Bytecode_::Call(m, n, tys) => write!(f, "Call {}.{}{}", m, n, format_type_actuals(tys)),
            Bytecode_::Pack(n, tys) => write!(f, "Pack {}{}", n, format_type_actuals(tys)),
            Bytecode_::Unpack(n, tys) => write!(f, "Unpack {}{}", n, format_type_actuals(tys)),
            Bytecode_::ReadRef => write!(f, "ReadRef"),
            Bytecode_::WriteRef => write!(f, "WriteRef"),
            Bytecode_::FreezeRef => write!(f, "FreezeRef"),
            Bytecode_::MutBorrowLoc(v) => write!(f, "MutBorrowLoc {}", v),
            Bytecode_::ImmBorrowLoc(v) => write!(f, "ImmBorrowLoc {}", v),
            Bytecode_::MutBorrowField(n, tys, field) => write!(
                f,
                "MutBorrowField {}{}.{}",
                n,
                format_type_actuals(tys),
                field
            ),
            Bytecode_::ImmBorrowField(n, tys, field) => write!(
                f,
                "ImmBorrowField {}{}.{}",
                n,
                format_type_actuals(tys),
                field
            ),
            Bytecode_::MutBorrowGlobal(n, tys) => {
                write!(f, "MutBorrowGlobal {}{}", n, format_type_actuals(tys))
            }
            Bytecode_::ImmBorrowGlobal(n, tys) => {
                write!(f, "ImmBorrowGlobal {}{}", n, format_type_actuals(tys))
            }
            Bytecode_::Add => write!(f, "Add"),
            Bytecode_::Sub => write!(f, "Sub"),
            Bytecode_::Mul => write!(f, "Mul"),
            Bytecode_::Mod => write!(f, "Mod"),
            Bytecode_::Div => write!(f, "Div"),
            Bytecode_::BitOr => write!(f, "BitOr"),
            Bytecode_::BitAnd => write!(f, "BitAnd"),
            Bytecode_::Xor => write!(f, "Xor"),
            Bytecode_::Or => write!(f, "Or"),
            Bytecode_::And => write!(f, "And"),
            Bytecode_::Not => write!(f, "Not"),
            Bytecode_::Eq => write!(f, "Eq"),
            Bytecode_::Neq => write!(f, "Neq"),
            Bytecode_::Lt => write!(f, "Lt"),
            Bytecode_::Gt => write!(f, "Gt"),
            Bytecode_::Le => write!(f, "Le"),
            Bytecode_::Ge => write!(f, "Ge"),
            Bytecode_::Abort => write!(f, "Abort"),
            Bytecode_::Exists(n, tys) => write!(f, "Exists {}{}", n, format_type_actuals(tys)),
            Bytecode_::MoveFrom(n, tys) => write!(f, "MoveFrom {}{}", n, format_type_actuals(tys)),
            Bytecode_::MoveTo(n, tys) => write!(f, "MoveTo {}{}", n, format_type_actuals(tys)),
            Bytecode_::Shl => write!(f, "Shl"),
            Bytecode_::Shr => write!(f, "Shr"),
        }
    }
}
