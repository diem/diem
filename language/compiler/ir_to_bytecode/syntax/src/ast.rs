// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use codespan::{ByteIndex, Span};
use std::{
    collections::{BTreeMap, HashSet, VecDeque},
    fmt,
    ops::Deref,
};
use types::{account_address::AccountAddress, byte_array::ByteArray, language_storage::ModuleId};

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
// Script
//**************************************************************************************************

#[derive(Debug, Clone)]
/// The move transaction script to be executed
pub struct Script {
    /// The dependencies of `main`, i.e. of the transaction script
    pub imports: Vec<ImportDefinition>,
    /// The transaction script's `main` procedure
    pub main: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

/// Newtype for a name of a module
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ModuleName(String);

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
    pub structs: Vec<StructDefinition>,
    /// the procedure that the module defines
    pub functions: Vec<(FunctionName, Function)>,
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
// Structs
//**************************************************************************************************

/// The file newtype
pub type Field = types::access_path::Field;
/// A field map
pub type Fields<T> = BTreeMap<Field, T>;

/// Newtype for the name of a struct
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StructName(String);

/// A Move struct
#[derive(Clone, Debug, PartialEq)]
pub struct StructDefinition {
    /// The struct will have kind resource if `resource_kind` is true
    /// and a value otherwise
    pub resource_kind: bool,
    /// Human-readable name for the struct that also serves as a nominal type
    pub name: StructName,
    /// the fields each instance has
    pub fields: Fields<Type>,
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

/// Newtype for the name of a function
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct FunctionName(String);

/// The signature of a function
#[derive(PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    /// Possibly-empty list of (formal name, formal type) pairs. Names are unique.
    pub formals: Vec<(Var, Type)>,
    /// Optional return types
    pub return_type: Vec<Type>,
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

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionAnnotation {
    Requires(String),
    Ensures(String),
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
    /// Annotations on the function
    pub annotations: Vec<FunctionAnnotation>,
    /// The code for the procedure
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

/// Used to annotate struct types as a resource or value
#[derive(Debug, PartialEq, Clone)]
pub enum Kind {
    /// `R`
    Resource,
    /// `V`
    Value,
}

/// Identifier for a struct definition. Tells us where to look in the storage layer to find the
/// code associated with the interface
#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StructType {
    /// Module name and address in which the struct is contained
    pub module: ModuleName,
    /// Name for the struct class. Should be unique among structs published under the same
    /// module+address
    pub name: StructName,
}

/// Type "name" of the type
#[derive(Debug, PartialEq, Clone)]
pub enum Tag {
    /// `address`
    Address,
    /// `u64`
    U64,
    /// `bool`
    Bool,
    /// `bytearray`
    ByteArray,
    /// `string`
    String,
    /// A module defined struct
    /// `n`
    Struct(StructType),
}

/// The type of a single value
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    /// A non reference type
    /// `g` or `k#d.n`
    Normal(Kind, Tag),
    /// A reference type
    /// `&t` or `&mut t`
    Reference {
        /// true if `&mut` and false if `&`
        is_mutable: bool,
        /// the kind, value or resource
        kind: Kind,
        /// the "name" of the type
        tag: Tag,
    },
}

//**************************************************************************************************
// Statements
//**************************************************************************************************

/// Newtype for a variable/local
#[derive(Debug, PartialEq, Hash, Eq, Clone, Ord, PartialOrd)]
pub struct Var(String);
/// The type of a variable with a location
pub type Var_ = Spanned<Var>;

/// Builtin "function"-like operators that often have a signature not expressable in the
/// type system and/or have access to some runtime/storage context
#[derive(Debug, PartialEq, Clone)]
pub enum Builtin {
    /// Intentionally destroy a resource (i.e., the inverse of `new`).
    Release,
    /// Check if there is a struct object (`StructName` resolved by current module) associated with
    /// the given address
    Exists(StructName),
    /// Get the struct object (`StructName` resolved by current module) associated with the given
    /// address
    BorrowGlobal(StructName),
    /// Returns the height of the current transaction.
    GetHeight,
    /// Returns the price per gas unit the current transaction is willing to pay
    GetTxnGasUnitPrice,
    /// Returns the maximum units of gas the current transaction is willing to use
    GetTxnMaxGasUnits,
    /// Returns the public key of the current transaction's sender
    GetTxnPublicKey,
    /// Returns the address of the current transaction's sender
    GetTxnSender,
    /// Returns the sequence number of the current transaction.
    GetTxnSequenceNumber,
    /// Returns the unit of gas remain to be used for now.
    GetGasRemaining,
    /// Emit an event
    EmitEvent,

    /// Publishing,
    /// Initialize a previously empty address by publishing a resource of type Account
    CreateAccount,
    /// Remove a resource of the given type from the account with the given address
    MoveFrom(StructName),
    /// Publish an instantiated struct object into sender's account.
    MoveToSender(StructName),

    /// Convert a mutable reference into an immutable one
    Freeze,
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
    },
}
/// The type for a function call and its location
pub type FunctionCall_ = Spanned<FunctionCall>;

/// Enum for Move commands
#[derive(Debug, Clone, PartialEq)]
pub enum Cmd {
    /// `x = e`
    Assign(Vec<Var_>, Exp_),
    /// `n { f_1: x_1, ... , f_j: x_j  } = e`
    Unpack(StructName, Fields<Var_>, Exp_),
    /// `*e_1 = e_2`
    Mutate(Exp_, Exp_),
    /// `abort e`
    Abort(Option<Exp_>),
    /// `return e_1, ... , e_j`
    Return(Exp_),
    /// `break`
    Break,
    /// `continue`
    Continue,
    Exp(Exp_),
}
/// The type of a command with its location
pub type Cmd_ = Spanned<Cmd>;

/// Struct defining an if statement
#[derive(Debug, PartialEq, Clone)]
pub struct IfElse {
    /// the if's condition
    pub cond: Exp_,
    /// the block taken if the condition is `true`
    pub if_block: Block,
    /// the block taken if the condition is `false`
    pub else_block: Option<Block>,
}

/// Struct defining a while statement
#[derive(Debug, PartialEq, Clone)]
pub struct While {
    /// The condition for a while statement
    pub cond: Exp_,
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
    CommandStatement(Cmd_),
    /// `if (e) { s_1 } else { s_2 }`
    IfElseStatement(IfElse),
    /// `while (e) { s }`
    WhileStatement(While),
    /// `loop { s }`
    LoopStatement(Loop),
    VerifyStatement(String),
    AssumeStatement(String),
    /// no-op that eases parsing in some places
    EmptyStatement,
}

#[derive(Debug, PartialEq, Clone)]
/// `{ s }`
pub struct Block {
    /// The statements that make up the block
    pub stmts: VecDeque<Statement>,
}

//**************************************************************************************************
// Expressions
//**************************************************************************************************

/// Bottom of the value hierarchy. These values can be trivially copyable and stored in statedb as a
/// single entry.
#[derive(Debug, PartialEq, Clone)]
pub enum CopyableVal {
    /// An address in the global storage
    Address(AccountAddress),
    /// An unsigned 64-bit integer
    U64(u64),
    /// true or false
    Bool(bool),
    /// `b"<bytes>"`
    ByteArray(ByteArray),
    /// Not yet supported in the parser
    String(String),
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
    Pack(StructName, ExpFields),
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
    FunctionCall(FunctionCall, Box<Exp_>),
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
            deps.insert(ModuleId::new(id.address, id.name.name()));
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
    pub fn new(imports: Vec<ImportDefinition>, main: Function) -> Self {
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

impl ModuleName {
    /// Create a new `ModuleName` identifier from a string
    pub fn new(name: String) -> Self {
        assert!(name != "");
        ModuleName(name)
    }

    /// String value for the current module handle
    pub const SELF: &'static str = "Self";

    /// Create a new `ModuleName` for the `SELF` constant
    pub fn module_self() -> Self {
        ModuleName::new(ModuleName::SELF.to_string())
    }

    /// Returns the raw bytes of the module name's string value
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Returns a cloned copy of the module name's string value
    pub fn name(&self) -> String {
        self.0.clone()
    }

    /// Accessor for the module name's string value
    pub fn name_ref(&self) -> &String {
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
    pub fn get_name(&self) -> &ModuleName {
        &self.name
    }

    /// Accessor for the address at which the module is published
    pub fn get_address(&self) -> &AccountAddress {
        &self.address
    }
}

impl ModuleIdent {
    pub fn get_name(&self) -> &ModuleName {
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
    pub fn new(
        name: String,
        imports: Vec<ImportDefinition>,
        structs: Vec<StructDefinition>,
        functions: Vec<(FunctionName, Function)>,
    ) -> Self {
        ModuleDefinition {
            name: ModuleName::new(name),
            imports,
            structs,
            functions,
        }
    }

    /// Return a vector of `ModuleId` for the external dependencies.
    pub fn get_external_deps(&self) -> Vec<ModuleId> {
        get_external_deps(self.imports.as_slice())
    }
}

impl Type {
    /// Creates a new non-reference type from the type's kind and tag
    pub fn nonreference(kind: Kind, tag: Tag) -> Type {
        Type::Normal(kind, tag)
    }

    /// Creates a new reference type from its mutability and underlying type
    pub fn reference(is_mutable: bool, annot: Type) -> Type {
        match annot {
            Type::Normal(kind, tag) => Type::Reference {
                is_mutable,
                kind,
                tag,
            },
            _ => panic!("ICE expected Normal annotation"),
        }
    }

    /// Creates a new address type
    pub fn address() -> Type {
        Type::Normal(Kind::Value, Tag::Address)
    }

    /// Creates a new u64 type
    pub fn u64() -> Type {
        Type::Normal(Kind::Value, Tag::U64)
    }

    /// Creates a new bool type
    pub fn bool() -> Type {
        Type::Normal(Kind::Value, Tag::Bool)
    }

    /// Creates a new bytearray type
    pub fn bytearray() -> Type {
        Type::Normal(Kind::Value, Tag::ByteArray)
    }
}

impl StructType {
    /// Creates a new StructType handle from the name of the module alias and the name of the struct
    pub fn new(module: ModuleName, name: StructName) -> Self {
        StructType { module, name }
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
            None => ident.get_name().clone(),
        };
        ImportDefinition { ident, alias }
    }
}

impl StructName {
    /// Create a new `StructName` identifier from a string
    pub fn new(name: String) -> Self {
        StructName(name)
    }

    /// Returns the raw bytes of the struct name's string value
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }

    /// Returns a cloned copy of the struct name's string value
    pub fn name(&self) -> String {
        self.0.clone()
    }

    /// Accessor for the name of the struct
    pub fn name_ref(&self) -> &String {
        &self.0
    }
}

impl StructDefinition {
    /// Creates a new StructDefinition from the resource kind (true if resource), the string
    /// representation of the name, and the field names with their types
    /// Does not verify the correctness of any internal properties, e.g. doesn't check that the
    /// fields do not have reference types
    pub fn new(resource_kind: bool, name: String, fields: Fields<Type>) -> Self {
        StructDefinition {
            resource_kind,
            name: StructName::new(name),
            fields,
        }
    }
}

impl FunctionName {
    /// Create a new `FunctionName` identifier from a string
    pub fn new(name: String) -> Self {
        FunctionName(name)
    }

    /// Returns a cloned copy of the function name's string value
    pub fn name(&self) -> String {
        self.0.clone()
    }

    /// Accessor for the name of the function
    pub fn name_ref(&self) -> &String {
        &self.0
    }
}

impl FunctionSignature {
    /// Creates a new function signature from the parameters and the return types
    pub fn new(formals: Vec<(Var, Type)>, return_type: Vec<Type>) -> Self {
        FunctionSignature {
            formals,
            return_type,
        }
    }
}

impl Function {
    /// Creates a new function declaration from the components of the function
    /// See the declaration of the struct `Function` for more details
    pub fn new(
        visibility: FunctionVisibility,
        formals: Vec<(Var, Type)>,
        return_type: Vec<Type>,
        annotations: Vec<FunctionAnnotation>,
        body: FunctionBody,
    ) -> Self {
        let signature = FunctionSignature::new(formals, return_type);
        Function {
            visibility,
            signature,
            annotations,
            body,
        }
    }
}

impl Var {
    /// Create a new `Var` identifier from a string
    pub fn new(s: &str) -> Self {
        Var(s.to_string())
    }

    /// Create a new `Var_` identifier from a string with an empty location
    pub fn new_(s: &str) -> Var_ {
        Spanned::no_loc(Var::new(s))
    }

    /// Accessor for the name of the var
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl FunctionCall {
    /// Creates a `FunctionCall::ModuleFunctionCall` variant
    pub fn module_call(module: ModuleName, name: FunctionName) -> Self {
        FunctionCall::ModuleFunctionCall { module, name }
    }

    /// Creates a `FunctionCall::Builtin` variant with no location information
    pub fn builtin(bif: Builtin) -> FunctionCall_ {
        Spanned::no_loc(FunctionCall::Builtin(bif))
    }
}

impl Cmd {
    /// Creates a command that returns no values
    pub fn return_empty() -> Self {
        Cmd::Return(Spanned::no_loc(Exp::ExprList(vec![])))
    }

    /// Creates a command that returns a single value
    pub fn return_(op: Exp_) -> Self {
        Cmd::Return(op)
    }
}

impl IfElse {
    /// Creates an if-statement with no else branch
    pub fn if_block(cond: Exp_, if_block: Block) -> Self {
        IfElse {
            cond,
            if_block,
            else_block: None,
        }
    }

    /// Creates an if-statement with an else branch
    pub fn if_else(cond: Exp_, if_block: Block, else_block: Block) -> Self {
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
    pub fn if_block(cond: Exp_, if_block: Block) -> Self {
        Statement::IfElseStatement(IfElse::if_block(cond, if_block))
    }

    /// Creates an `Statement::IfElseStatement` variant with an else branch
    pub fn if_else(cond: Exp_, if_block: Block, else_block: Block) -> Self {
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
    pub fn instantiate(n: StructName, s: ExpFields) -> Exp_ {
        Spanned::no_loc(Exp::Pack(n, s))
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
    pub fn function_call(f: FunctionCall, e: Exp_) -> Exp_ {
        Spanned::no_loc(Exp::FunctionCall(f, Box::new(e)))
    }

    pub fn expr_list(exps: Vec<Exp_>) -> Exp_ {
        Spanned::no_loc(Exp::ExprList(exps))
    }
}

//**************************************************************************************************
// Trait impls
//**************************************************************************************************

impl Iterator for Script {
    type Item = Statement;

    fn next(&mut self) -> Option<Statement> {
        match self.main.body {
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

impl Into<Field> for CopyableVal {
    fn into(self) -> Field {
        Field::new(self.to_string().as_ref())
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
        writeln!(f, "Module({}, ", self.name.name())?;
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
        writeln!(f, "Struct({}, ", self.name)?;
        writeln!(f, "{}", format_fields(&self.fields))?;
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

fn intersperse<T: fmt::Display>(items: &[T], join: &str) -> String {
    items.iter().fold(String::new(), |acc, v| {
        format!("{acc}{join}{v}", acc = acc, join = join, v = v)
    })
}

fn format_fields<T: fmt::Display>(fields: &Fields<T>) -> String {
    fields.iter().fold(String::new(), |acc, (field, val)| {
        format!("{} {}: {},", acc, field, val)
    })
}

impl fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (v, ty) in self.formals.iter() {
            write!(f, "{}: {}, ", v, ty)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Resource => write!(f, "R"),
            Kind::Value => write!(f, "V"),
        }
    }
}

impl fmt::Display for StructType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.module, self.name.name())
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::U64 => write!(f, "u64"),
            Tag::Bool => write!(f, "bool"),
            Tag::Address => write!(f, "address"),
            Tag::ByteArray => write!(f, "bytearray"),
            Tag::String => write!(f, "string"),
            Tag::Struct(ty) => write!(f, "{}", ty),
        }
    }
}

fn write_kind_tag(f: &mut fmt::Formatter<'_>, k: &Kind, t: &Tag) -> fmt::Result {
    match t {
        Tag::Struct(_) => write!(f, "{}#{}", k, t),
        _ => write!(f, "{}", t),
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Normal(k, t) => write_kind_tag(f, k, t),
            Type::Reference {
                kind,
                tag,
                is_mutable,
            } => {
                write!(f, "&{}", if *is_mutable { "mut " } else { "" })?;
                write_kind_tag(f, kind, tag)
            }
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
            Builtin::CreateAccount => write!(f, "create_account"),
            Builtin::Release => write!(f, "release"),
            Builtin::EmitEvent => write!(f, "log"),
            Builtin::Exists(t) => write!(f, "exists<{}>", t),
            Builtin::BorrowGlobal(t) => write!(f, "borrow_global<{}>", t),
            Builtin::GetHeight => write!(f, "get_height"),
            Builtin::GetTxnMaxGasUnits => write!(f, "get_txn_max_gas_units"),
            Builtin::GetTxnGasUnitPrice => write!(f, "get_txn_gas_unit_price"),
            Builtin::GetTxnPublicKey => write!(f, "get_txn_public_key"),
            Builtin::GetTxnSender => write!(f, "get_txn_sender"),
            Builtin::GetTxnSequenceNumber => write!(f, "get_txn_sequence_number"),
            Builtin::GetGasRemaining => write!(f, "get_gas_remaining"),
            Builtin::MoveFrom(t) => write!(f, "move_from<{}>", t),
            Builtin::MoveToSender(t) => write!(f, "move_to_sender<{}>", t),
            Builtin::Freeze => write!(f, "freeze"),
        }
    }
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FunctionCall::Builtin(fun) => write!(f, "{}", fun),
            FunctionCall::ModuleFunctionCall { module, name } => write!(f, "{}.{}", module, name),
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
            Cmd::Unpack(n, bindings, e) => write!(
                f,
                "{} {{ {} }} = {}",
                n,
                bindings
                    .iter()
                    .fold(String::new(), |acc, (field, var)| format!(
                        "{} {} : {},",
                        acc, field, var
                    )),
                e
            ),
            Cmd::Mutate(e, o) => write!(f, "*({}) = {};", e, o),
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
            Statement::VerifyStatement(cond) => write!(f, "verify<{}>)", cond),
            Statement::AssumeStatement(cond) => write!(f, "assume<{}>", cond),
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
            CopyableVal::U64(v) => write!(f, "{}", v),
            CopyableVal::Bool(v) => write!(f, "{}", v),
            CopyableVal::ByteArray(v) => write!(f, "{}", v),
            CopyableVal::Address(v) => write!(f, "0x{}", hex::encode(&v)),
            CopyableVal::String(v) => write!(f, "{}", v),
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
            Exp::Pack(n, s) => write!(
                f,
                "{}{{{}}}",
                n,
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
