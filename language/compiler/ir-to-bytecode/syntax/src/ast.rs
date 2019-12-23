// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::spec_language_ast::Condition;
use crate::syntax::ParseError;
use codespan::{ByteIndex, Span};
use lazy_static::lazy_static;
use libra_types::{
    account_address::AccountAddress,
    byte_array::ByteArray,
    identifier::{IdentStr, Identifier},
    language_storage::ModuleId,
};
use std::{
    collections::{HashSet, VecDeque},
    fmt,
    ops::Deref,
};

/// Generic wrapper that keeps file locations for any ast-node
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Spanned<T> {
    /// The file location
    pub span: Loc,
    /// The value being wrapped
    pub value: T,
}

/// The file location type
pub type Loc = Span<ByteIndex>;

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
/// A set of move modules and a Move transaction script
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
/// The move transaction script to be executed
pub struct Script {
    /// The dependencies of `main`, i.e. of the transaction script
    pub imports: Vec<ImportDefinition>,
    /// The transaction script's `main` procedure
    pub main: Function_,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

/// Newtype for a name of a module
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ModuleName(Identifier);

/// Newtype of the address + the module name
/// `addr.m`
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct QualifiedModuleIdent {
    /// Name for the module. Will be unique among modules published under the same address
    pub name: ModuleName,
    /// Address that this module is published under
    pub address: AccountAddress,
}

/// A Move module
#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDefinition {
    /// name of the module
    pub name: ModuleName,
    /// the module's dependencies
    pub imports: Vec<ImportDefinition>,
    /// the structs (including resources) that the module defines
    pub structs: Vec<StructDefinition_>,
    /// the procedure that the module defines
    pub functions: Vec<(FunctionName, Function_)>,
}

/// Either a qualified module name like `addr.m` or `Transaction.m`, which refers to a module in
/// the same transaction.
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum ModuleIdent {
    Transaction(ModuleName),
    Qualified(QualifiedModuleIdent),
}

//**************************************************************************************************
// Imports
//**************************************************************************************************

/// A dependency/import declaration
#[derive(Clone, Debug, PartialEq)]
pub struct ImportDefinition {
    /// the dependency
    /// `addr.m` or `Transaction.m`
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
pub struct Var(Identifier);

/// The type of a variable with a location
pub type Var_ = Spanned<Var>;

/// New type that represents a type variable. Used to declare type formals & reference them.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TypeVar(Identifier);

/// The type of a type variable with a location.
pub type TypeVar_ = Spanned<TypeVar>;

//**************************************************************************************************
// Kinds
//**************************************************************************************************

// TODO: This enum is completely equivalent to vm::file_format::Kind.
//       Should we just use vm::file_format::Kind or replace both with a common one?
/// The kind of a type. Analogous to `vm::file_format::Kind`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
    /// Represents the super set of all types.
    All,
    /// `Resource` types must follow move semantics and various resource safety rules.
    Resource,
    /// `Unrestricted` types do not need to follow the `Resource` rules.
    Unrestricted,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

/// The type of a single value
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// `address`
    Address,
    /// `u8`
    U8,
    /// `u64`
    U64,
    /// `u128`
    U128,
    /// `bool`
    Bool,
    /// `bytearray`
    ByteArray,
    /// A module defined struct
    Struct(QualifiedStructIdent, Vec<Type>),
    /// A reference type, the bool flag indicates whether the reference is mutable
    Reference(bool, Box<Type>),
    /// A type parameter
    TypeParameter(TypeVar),
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
pub type Field = libra_types::access_path::Field;

/// A field coupled with source location information
pub type Field_ = Spanned<Field>;

/// A field map
pub type Fields<T> = Vec<(Field_, T)>;

/// Newtype for the name of a struct
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StructName(Identifier);

/// A Move struct
#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    /// The struct will have kind resource if `is_nominal_resource` is true
    /// and will be dependent on it's type arguments otherwise
    pub is_nominal_resource: bool,
    /// Human-readable name for the struct that also serves as a nominal type
    pub name: StructName,
    /// Kind constraints of the type parameters
    pub type_formals: Vec<(TypeVar_, Kind)>,
    /// the fields each instance has
    pub fields: StructDefinitionFields,
}

/// The type of a StructDefinition along with its source location information
pub type StructDefinition_ = Spanned<StructDefinition>;

/// The fields of a Move struct definition
#[derive(Clone, Debug, PartialEq)]
pub enum StructDefinitionFields {
    /// The fields are declared
    Move { fields: Fields<Type> },
    /// The struct is a type provided by the VM
    Native,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

/// Newtype for the name of a function
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct FunctionName(Identifier);

/// The signature of a function
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    /// Possibly-empty list of (formal name, formal type) pairs. Names are unique.
    pub formals: Vec<(Var_, Type)>,
    /// Optional return types
    pub return_type: Vec<Type>,
    /// Possibly-empty list of (TypeVar, Kind) pairs.s.
    pub type_formals: Vec<(TypeVar_, Kind)>,
}

/// Public or internal modifier for a procedure
#[derive(PartialEq, Debug, Clone)]
pub enum FunctionVisibility {
    /// The procedure can be invoked anywhere
    /// `public`
    Public,
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
        locals: Vec<(Var_, Type)>,
        code: Block,
    },
    /// The body is provided by the runtime
    Native,
}

/// A Move function/procedure
#[derive(PartialEq, Debug, Clone)]
pub struct Function {
    /// The visibility (public or internal)
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
pub type Function_ = Spanned<Function>;

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
    /// Returns the address of the current transaction's sender
    GetTxnSender,

    /// Remove a resource of the given type from the account with the given address
    MoveFrom(StructName, Vec<Type>),
    /// Publish an instantiated struct object into sender's account.
    MoveToSender(StructName, Vec<Type>),

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
pub enum FunctionCall {
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
pub type FunctionCall_ = Spanned<FunctionCall>;

/// Enum for Move lvalues
#[derive(Debug, Clone, PartialEq)]
pub enum LValue {
    /// `x`
    Var(Var_),
    /// `*e`
    Mutate(Exp_),
    /// `_`
    Pop,
}
pub type LValue_ = Spanned<LValue>;

/// Enum for Move commands
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// `l_1, ..., l_n = e`
    Assign(Vec<LValue_>, Exp_),
    /// `n { f_1: x_1, ... , f_j: x_j  } = e`
    Unpack(StructName, Vec<Type>, Fields<Var_>, Box<Exp_>),
    /// `abort e`
    Abort(Option<Box<Exp_>>),
    /// `return e_1, ... , e_j`
    Return(Box<Exp_>),
    /// `break`
    Break,
    /// `continue`
    Continue,
    Exp(Box<Exp_>),
}
/// The type of a command with its location
pub type Cmd_ = Spanned<Cmd>;

/// Struct defining an if statement
#[derive(Debug, PartialEq, Clone)]
pub struct IfElse {
    /// the if's condition
    pub cond: Exp_,
    /// the block taken if the condition is `true`
    pub if_block: Block_,
    /// the block taken if the condition is `false`
    pub else_block: Option<Block_>,
}

/// Struct defining a while statement
#[derive(Debug, PartialEq, Clone)]
pub struct While {
    /// The condition for a while statement
    pub cond: Exp_,
    /// The block taken if the condition is `true`
    pub block: Block_,
}

/// Struct defining a loop statement
#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    /// The body of the loop
    pub block: Block_,
}

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Statement {
    /// `c;`
    CommandStatement(Cmd_),
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
pub struct Block {
    /// The statements that make up the block
    pub stmts: VecDeque<Statement>,
}

/// The type of a Block coupled with source location information.
pub type Block_ = Spanned<Block>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

/// Bottom of the value hierarchy. These values can be trivially copyable and stored in statedb as a
/// single entry.
#[derive(Debug, PartialEq, Clone)]
pub enum CopyableVal {
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
    ByteArray(ByteArray),
}

/// The type of a value and its location
pub type CopyableVal_ = Spanned<CopyableVal>;

/// The type for fields and their bound expressions
pub type ExpFields = Fields<Exp_>;

/// Enum for unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    /// Boolean negation
    Not,
}

/// Enum for binary operators
#[derive(Debug, Clone, PartialEq)]
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
}

/// Enum for all expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Exp {
    /// `*e`
    Dereference(Box<Exp_>),
    /// `op e`
    UnaryExp(UnaryOp, Box<Exp_>),
    /// `e_1 op e_2`
    BinopExp(Box<Exp_>, BinOp, Box<Exp_>),
    /// Wrapper to lift `CopyableVal` into `Exp`
    /// `v`
    Value(CopyableVal_),
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
        exp: Box<Exp_>,
        /// the field being borrowed
        field: Field,
    },
    /// `move(x)`
    Move(Var_),
    /// `copy(x)`
    Copy(Var_),
    /// `&x` or `&mut x`
    BorrowLocal(bool, Var_),
    /// `f(e)` or `f(e_1, e_2, ..., e_j)`
    FunctionCall(FunctionCall_, Box<Exp_>),
    /// (e_1, e_2, e_3, ..., e_j)
    ExprList(Vec<Exp_>),
}

/// The type for a `Exp` and its location
pub type Exp_ = Spanned<Exp>;

//**************************************************************************************************
// impls
//**************************************************************************************************

fn get_external_deps(imports: &[ImportDefinition]) -> Vec<ModuleId> {
    let mut deps = HashSet::new();
    for dep in imports.iter() {
        if let ModuleIdent::Qualified(id) = &dep.ident {
            deps.insert(ModuleId::new(id.address, id.name.clone().into_inner()));
        }
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
    pub fn new(imports: Vec<ImportDefinition>, main: Function_) -> Self {
        Script { imports, main }
    }

    /// Accessor for the body of the 'main' procedure
    pub fn body(&self) -> &Block {
        match self.main.body {
            FunctionBody::Move { ref code, .. } => &code,
            FunctionBody::Native => panic!("main() can't be native"),
        }
    }

    /// Return a vector of `ModuleId` for the external dependencies.
    pub fn get_external_deps(&self) -> Vec<ModuleId> {
        get_external_deps(self.imports.as_slice())
    }
}

lazy_static! {
    static ref SELF_MODULE_NAME: Identifier = Identifier::new("Self").unwrap();
}

impl ModuleName {
    /// Create a new `ModuleName` from an identifier
    pub fn new(name: Identifier) -> Self {
        assert!(!name.is_empty());
        ModuleName(name)
    }

    /// Creates a new `ModuleName` from a raw string. Intended for use by the parser.
    pub fn parse<L>(s: impl Into<Box<str>>) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(ModuleName::new(parse_identifier(s.into())?))
    }

    /// Name for the current module handle
    pub fn self_name() -> &'static IdentStr {
        &*SELF_MODULE_NAME
    }

    /// Create a new `ModuleName` from `self_name`.
    pub fn module_self() -> Self {
        ModuleName::new(ModuleName::self_name().into())
    }

    /// Converts self into an identifier.
    pub fn into_inner(self) -> Identifier {
        self.0
    }

    /// Accessor for the name of the module
    pub fn as_inner(&self) -> &IdentStr {
        &self.0
    }
}

impl QualifiedModuleIdent {
    /// Creates a new fully qualified module identifier from the module name and the address at
    /// which it is published
    pub fn new(name: ModuleName, address: AccountAddress) -> Self {
        QualifiedModuleIdent { address, name }
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

impl ModuleIdent {
    pub fn name(&self) -> &ModuleName {
        match self {
            ModuleIdent::Transaction(name) => &name,
            ModuleIdent::Qualified(id) => &id.name,
        }
    }
}

impl ModuleDefinition {
    /// Creates a new `ModuleDefinition` from its string name, dependencies, structs+resources,
    /// and procedures
    /// Does not verify the correctness of any internal properties of its elements
    pub fn new<L>(
        name: impl Into<Box<str>>,
        imports: Vec<ImportDefinition>,
        structs: Vec<StructDefinition_>,
        functions: Vec<(FunctionName, Function_)>,
    ) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(ModuleDefinition {
            name: ModuleName::parse(name.into())?,
            imports,
            structs,
            functions,
        })
    }

    /// Return a vector of `ModuleId` for the external dependencies.
    pub fn get_external_deps(&self) -> Vec<ModuleId> {
        get_external_deps(self.imports.as_slice())
    }
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

    /// Creates a new bytearray type
    pub fn bytearray() -> Type {
        Type::ByteArray
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
            None => ident.name().clone(),
        };
        ImportDefinition { ident, alias }
    }
}

impl StructName {
    /// Create a new `StructName` from an identifier
    pub fn new(name: Identifier) -> Self {
        StructName(name)
    }

    /// Creates a new `StructName` from a raw string. Intended for use by the parser.
    pub fn parse<L>(s: impl Into<Box<str>>) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(StructName::new(parse_identifier(s.into())?))
    }

    /// Converts self into an identifier.
    pub fn into_inner(self) -> Identifier {
        self.0
    }

    /// Accessor for the name of the struct
    pub fn as_inner(&self) -> &IdentStr {
        &self.0
    }
}

impl StructDefinition {
    /// Creates a new StructDefinition from the resource kind (true if resource), the string
    /// representation of the name, and the user specified fields, a map from their names to their
    /// types
    /// Does not verify the correctness of any internal properties, e.g. doesn't check that the
    /// fields do not have reference types
    pub fn move_declared<L>(
        is_nominal_resource: bool,
        name: impl Into<Box<str>>,
        type_formals: Vec<(TypeVar_, Kind)>,
        fields: Fields<Type>,
    ) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(StructDefinition {
            is_nominal_resource,
            name: StructName::parse(name)?,
            type_formals,
            fields: StructDefinitionFields::Move { fields },
        })
    }

    /// Creates a new StructDefinition from the resource kind (true if resource), the string
    /// representation of the name, and the user specified fields, a map from their names to their
    /// types
    pub fn native<L>(
        is_nominal_resource: bool,
        name: impl Into<Box<str>>,
        type_formals: Vec<(TypeVar_, Kind)>,
    ) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(StructDefinition {
            is_nominal_resource,
            name: StructName::parse(name)?,
            type_formals,
            fields: StructDefinitionFields::Native,
        })
    }
}

impl FunctionName {
    /// Create a new `FunctionName` from an identifier
    pub fn new(name: Identifier) -> Self {
        FunctionName(name)
    }

    /// Creates a new `FunctionName` from a raw string. Intended for use by the parser.
    pub fn parse<L>(s: impl Into<Box<str>>) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(FunctionName::new(parse_identifier(s.into())?))
    }

    /// Converts self into an identifier.
    pub fn into_inner(self) -> Identifier {
        self.0
    }

    /// Accessor for the name of the function
    pub fn as_inner(&self) -> &IdentStr {
        &self.0
    }
}

impl FunctionSignature {
    /// Creates a new function signature from the parameters and the return types
    pub fn new(
        formals: Vec<(Var_, Type)>,
        return_type: Vec<Type>,
        type_formals: Vec<(TypeVar_, Kind)>,
    ) -> Self {
        FunctionSignature {
            formals,
            return_type,
            type_formals,
        }
    }
}

impl Function {
    /// Creates a new function declaration from the components of the function
    /// See the declaration of the struct `Function` for more details
    pub fn new(
        visibility: FunctionVisibility,
        formals: Vec<(Var_, Type)>,
        return_type: Vec<Type>,
        type_formals: Vec<(TypeVar_, Kind)>,
        acquires: Vec<StructName>,
        specifications: Vec<Condition>,
        body: FunctionBody,
    ) -> Self {
        let signature = FunctionSignature::new(formals, return_type, type_formals);
        Function {
            visibility,
            signature,
            acquires,
            specifications,
            body,
        }
    }
}

impl Var {
    /// Creates a new `Var` from an identifier.
    pub fn new(s: Identifier) -> Self {
        Var(s)
    }

    /// Creates a new `Var_` identifier from an identifier with an empty location.
    pub fn new_(s: Identifier) -> Var_ {
        Spanned::no_loc(Var::new(s))
    }

    /// Creates a new `Var` from a raw string. Intended for use by the parser.
    pub fn parse<L>(s: impl Into<Box<str>>) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(Var::new(parse_identifier(s.into())?))
    }

    /// Accessor for the name of the var
    pub fn name(&self) -> &IdentStr {
        &self.0
    }
}

impl TypeVar {
    /// Creates a new `TypeVar` from an identifier.
    pub fn new(s: Identifier) -> Self {
        TypeVar(s)
    }

    /// Creates a new `TypeVar` from a raw string. Intended for use by the parser.
    pub fn parse<L>(s: impl Into<Box<str>>) -> Result<Self, ParseError<L, anyhow::Error>> {
        Ok(TypeVar::new(parse_identifier(s.into())?))
    }

    /// Accessor for the name of the var.
    pub fn name(&self) -> &IdentStr {
        &self.0
    }
}

impl FunctionCall {
    /// Creates a `FunctionCall::ModuleFunctionCall` variant
    pub fn module_call(module: ModuleName, name: FunctionName, type_actuals: Vec<Type>) -> Self {
        FunctionCall::ModuleFunctionCall {
            module,
            name,
            type_actuals,
        }
    }

    /// Creates a `FunctionCall::Builtin` variant with no location information
    pub fn builtin(bif: Builtin) -> FunctionCall_ {
        Spanned::no_loc(FunctionCall::Builtin(bif))
    }
}

impl Cmd {
    /// Creates a command that returns no values
    pub fn return_empty() -> Self {
        Cmd::Return(Box::new(Spanned::no_loc(Exp::ExprList(vec![]))))
    }

    /// Creates a command that returns a single value
    pub fn return_(op: Exp_) -> Self {
        Cmd::Return(Box::new(op))
    }
}

impl IfElse {
    /// Creates an if-statement with no else branch
    pub fn if_block(cond: Exp_, if_block: Block_) -> Self {
        IfElse {
            cond,
            if_block,
            else_block: None,
        }
    }

    /// Creates an if-statement with an else branch
    pub fn if_else(cond: Exp_, if_block: Block_, else_block: Block_) -> Self {
        IfElse {
            cond,
            if_block,
            else_block: Some(else_block),
        }
    }
}

impl Statement {
    /// Lifts a command into a statement
    pub fn cmd(c: Cmd_) -> Self {
        Statement::CommandStatement(c)
    }

    /// Creates an `Statement::IfElseStatement` variant with no else branch
    pub fn if_block(cond: Exp_, if_block: Block_) -> Self {
        Statement::IfElseStatement(IfElse::if_block(cond, if_block))
    }

    /// Creates an `Statement::IfElseStatement` variant with an else branch
    pub fn if_else(cond: Exp_, if_block: Block_, else_block: Block_) -> Self {
        Statement::IfElseStatement(IfElse::if_else(cond, if_block, else_block))
    }
}

impl Block {
    /// Creates a new block from the vector of statements
    pub fn new(stmts: Vec<Statement>) -> Self {
        Block {
            stmts: VecDeque::from(stmts),
        }
    }

    /// Creates an empty block
    pub fn empty() -> Self {
        Block {
            stmts: VecDeque::new(),
        }
    }
}

impl Exp {
    /// Creates a new address `Exp` with no location information
    pub fn address(addr: AccountAddress) -> Exp_ {
        Spanned::no_loc(Exp::Value(Spanned::no_loc(CopyableVal::Address(addr))))
    }

    /// Creates a new value `Exp` with no location information
    pub fn value(b: CopyableVal) -> Exp_ {
        Spanned::no_loc(Exp::Value(Spanned::no_loc(b)))
    }

    /// Creates a new u64 `Exp` with no location information
    pub fn u64(i: u64) -> Exp_ {
        Exp::value(CopyableVal::U64(i))
    }

    /// Creates a new bool `Exp` with no location information
    pub fn bool(b: bool) -> Exp_ {
        Exp::value(CopyableVal::Bool(b))
    }

    /// Creates a new bytearray `Exp` with no location information
    pub fn byte_array(buf: ByteArray) -> Exp_ {
        Exp::value(CopyableVal::ByteArray(buf))
    }

    /// Creates a new pack/struct-instantiation `Exp` with no location information
    pub fn instantiate(n: StructName, tys: Vec<Type>, s: ExpFields) -> Exp_ {
        Spanned::no_loc(Exp::Pack(n, tys, s))
    }

    /// Creates a new binary operator `Exp` with no location information
    pub fn binop(lhs: Exp_, op: BinOp, rhs: Exp_) -> Exp_ {
        Spanned::no_loc(Exp::BinopExp(Box::new(lhs), op, Box::new(rhs)))
    }

    /// Creates a new `e+e` `Exp` with no location information
    pub fn add(lhs: Exp_, rhs: Exp_) -> Exp_ {
        Exp::binop(lhs, BinOp::Add, rhs)
    }

    /// Creates a new `e-e` `Exp` with no location information
    pub fn sub(lhs: Exp_, rhs: Exp_) -> Exp_ {
        Exp::binop(lhs, BinOp::Sub, rhs)
    }

    /// Creates a new `*e` `Exp` with no location information
    pub fn dereference(e: Exp_) -> Exp_ {
        Spanned::no_loc(Exp::Dereference(Box::new(e)))
    }

    /// Creates a new borrow field `Exp` with no location information
    pub fn borrow(is_mutable: bool, exp: Box<Exp_>, field: Field) -> Exp_ {
        Spanned::no_loc(Exp::Borrow {
            is_mutable,
            exp,
            field,
        })
    }

    /// Creates a new copy-local `Exp` with no location information
    pub fn copy(v: Var_) -> Exp_ {
        Spanned::no_loc(Exp::Copy(v))
    }

    /// Creates a new move-local `Exp` with no location information
    pub fn move_(v: Var_) -> Exp_ {
        Spanned::no_loc(Exp::Move(v))
    }

    /// Creates a new function call `Exp` with no location information
    pub fn function_call(f: FunctionCall_, e: Exp_) -> Exp_ {
        Spanned::no_loc(Exp::FunctionCall(f, Box::new(e)))
    }

    pub fn expr_list(exps: Vec<Exp_>) -> Exp_ {
        Spanned::no_loc(Exp::ExprList(exps))
    }
}

/// Parses a field.
pub fn parse_field<L>(s: impl Into<Box<str>>) -> Result<Field, ParseError<L, anyhow::Error>> {
    Ok(Field::new(parse_identifier(s.into())?))
}

fn parse_identifier<L>(s: Box<str>) -> Result<Identifier, ParseError<L, anyhow::Error>> {
    Identifier::new(s).map_err(|error| ParseError::User { error })
}

//**************************************************************************************************
// Trait impls
//**************************************************************************************************

impl Iterator for Script {
    type Item = Statement;

    fn next(&mut self) -> Option<Statement> {
        match self.main.value.body {
            FunctionBody::Move { ref mut code, .. } => code.stmts.pop_front(),
            FunctionBody::Native => panic!("main() cannot be native code"),
        }
    }
}

impl PartialEq for Script {
    fn eq(&self, other: &Script) -> bool {
        self.imports == other.imports && self.main.body == other.main.body
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> Spanned<T> {
    pub fn no_loc(value: T) -> Spanned<T> {
        Spanned {
            value,
            span: Span::default(),
        }
    }
}

impl Iterator for Block {
    type Item = Statement;

    fn next(&mut self) -> Option<Statement> {
        self.stmts.pop_front()
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl<T> fmt::Display for Spanned<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Kind::All => "all",
                Kind::Resource => "resource",
                Kind::Unrestricted => "unrestricted",
            }
        )
    }
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
        write!(f, "Main(")?;
        write!(f, "{}", self.main)?;
        write!(f, ")")?;
        write!(f, ")")
    }
}

impl fmt::Display for ImportDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ModuleIdent::*;
        write!(f, "ImportDefinition(")?;
        match &self.ident {
            Transaction(module_name) => write!(f, "{}", module_name)?,
            Qualified(qual_module_ident) => write!(f, "{}", qual_module_ident)?,
        };
        write!(f, " => {})", self.alias)
    }
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for QualifiedModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}.{}", self.address, self.name)
    }
}

impl fmt::Display for ModuleDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Module({}, ", self.name)?;
        write!(f, "Structs(")?;
        for struct_def in &self.structs {
            write!(f, "{}, ", struct_def)?;
        }
        write!(f, "Functions(")?;
        for (fun_name, fun) in &self.functions {
            write!(f, "({}, {}), ", fun_name, fun)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for StructDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Struct({}{}, ",
            self.name,
            format_type_formals(&self.type_formals)
        )?;
        match &self.fields {
            StructDefinitionFields::Move { fields } => writeln!(f, "{}", format_fields(fields))?,
            StructDefinitionFields::Native => writeln!(f, "{{native}}")?,
        }
        write!(f, ")")
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.signature, self.body)
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

fn format_fields<T: fmt::Display>(fields: &[(Field_, T)]) -> String {
    fields.iter().fold(String::new(), |acc, (field, val)| {
        format!("{} {}: {},", acc, field.value, val)
    })
}

impl fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_type_formals(&self.type_formals))?;
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

fn format_type_formals(formals: &[(TypeVar_, Kind)]) -> String {
    if formals.is_empty() {
        "".to_string()
    } else {
        let formatted = formals
            .iter()
            .map(|(tv, k)| format!("{}: {}", tv.value, k))
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
            Type::ByteArray => write!(f, "bytearray"),
            Type::Struct(ident, tys) => write!(f, "{}{}", ident, format_type_actuals(tys)),
            Type::Reference(is_mutable, t) => {
                write!(f, "&{}{}", if *is_mutable { "mut " } else { "" }, t)
            }
            Type::TypeParameter(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for Var {
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
            Builtin::GetTxnSender => write!(f, "get_txn_sender"),
            Builtin::MoveFrom(t, tys) => write!(f, "move_from<{}{}>", t, format_type_actuals(tys)),
            Builtin::MoveToSender(t, tys) => {
                write!(f, "move_to_sender<{}{}>", t, format_type_actuals(tys))
            }
            Builtin::Freeze => write!(f, "freeze"),
            Builtin::ToU8 => write!(f, "to_u8"),
            Builtin::ToU64 => write!(f, "to_u64"),
            Builtin::ToU128 => write!(f, "to_u128"),
        }
    }
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionCall::Builtin(fun) => write!(f, "{}", fun),
            FunctionCall::ModuleFunctionCall {
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

impl fmt::Display for LValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LValue::Var(x) => write!(f, "{}", x),
            LValue::Mutate(e) => write!(f, "*{}", e),
            LValue::Pop => write!(f, "_"),
        }
    }
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cmd::Assign(var_list, e) => {
                if var_list.is_empty() {
                    write!(f, "{};", e)
                } else {
                    write!(f, "{} = ({});", intersperse(var_list, ", "), e)
                }
            }
            Cmd::Unpack(n, tys, bindings, e) => write!(
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
            Cmd::Abort(None) => write!(f, "abort;"),
            Cmd::Abort(Some(err)) => write!(f, "abort {};", err),
            Cmd::Return(exps) => write!(f, "return {};", exps),
            Cmd::Break => write!(f, "break;"),
            Cmd::Continue => write!(f, "continue;"),
            Cmd::Exp(e) => write!(f, "({});", e),
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

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for stmt in self.stmts.iter() {
            writeln!(f, "{}", stmt)?;
        }
        Ok(())
    }
}

impl fmt::Display for CopyableVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CopyableVal::U8(v) => write!(f, "{}u8", v),
            CopyableVal::U64(v) => write!(f, "{}", v),
            CopyableVal::U128(v) => write!(f, "{}u128", v),
            CopyableVal::Bool(v) => write!(f, "{}", v),
            CopyableVal::ByteArray(v) => write!(f, "{}", v),
            CopyableVal::Address(v) => write!(f, "0x{}", hex::encode(&v)),
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
            }
        )
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Exp::Dereference(e) => write!(f, "*({})", e),
            Exp::UnaryExp(o, e) => write!(f, "({}{})", o, e),
            Exp::BinopExp(e1, o, e2) => write!(f, "({} {} {})", o, e1, e2),
            Exp::Value(v) => write!(f, "{}", v),
            Exp::Pack(n, tys, s) => write!(
                f,
                "{}{}{{{}}}",
                n,
                format_type_actuals(tys),
                s.iter().fold(String::new(), |acc, (field, op)| format!(
                    "{} {} : {},",
                    acc, field, op,
                ))
            ),
            Exp::Borrow {
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
            Exp::Move(v) => write!(f, "move({})", v),
            Exp::Copy(v) => write!(f, "copy({})", v),
            Exp::BorrowLocal(is_mutable, v) => {
                write!(f, "&{}{}", if *is_mutable { "mut " } else { "" }, v)
            }
            Exp::FunctionCall(func, e) => write!(f, "{}({})", func, e),
            Exp::ExprList(exps) => {
                if exps.is_empty() {
                    write!(f, "()")
                } else {
                    write!(f, "({})", intersperse(exps, ", "))
                }
            }
        }
    }
}
