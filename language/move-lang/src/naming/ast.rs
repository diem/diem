// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    expansion::ast::Fields,
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent, ResourceLoc,
        StructName, UnaryOp, Value, Value_, Var,
    },
    shared::{ast_debug::*, unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub main: Option<(Address, FunctionName, Function)>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    pub uses: BTreeMap<ModuleIdent, Loc>,
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    /// `dependency_order` is initialized at `0` and set in the uses pass
    pub dependency_order: usize,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
pub struct StructDefinition {
    pub resource_opt: ResourceLoc,
    pub type_parameters: Vec<TParam>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructFields {
    Defined(Fields<BaseType>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionSignature {
    pub type_parameters: Vec<TParam>,
    pub parameters: Vec<(Var, SingleType)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: BTreeSet<StructName>,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone, PartialOrd, Eq, Ord)]
pub enum BuiltinTypeName_ {
    // address
    Address,
    // u8
    U8,
    // u64
    U64,
    // u128
    U128,
    // Vector
    Vector,
    // bool
    Bool,
    // bytearray
    Bytearray,
}
pub type BuiltinTypeName = Spanned<BuiltinTypeName_>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TypeName_ {
    Builtin(BuiltinTypeName),
    ModuleType(ModuleIdent, StructName),
}
pub type TypeName = Spanned<TypeName_>;

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TParamID(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TParam {
    pub id: TParamID,
    pub debug: Name,
    pub kind: Kind,
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct TVar(u64);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BaseType_ {
    Param(TParam),
    Apply(Option<Kind>, TypeName, Vec<BaseType>),
    Var(TVar),
    Anything,
}
pub type BaseType = Spanned<BaseType_>;

#[derive(Debug, PartialEq, Clone)]
pub enum SingleType_ {
    Base(BaseType),
    Ref(bool, BaseType),
}
pub type SingleType = Spanned<SingleType_>;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    Unit,
    Single(SingleType),
    Multiple(Vec<SingleType>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub enum Assign_ {
    Ignore,
    Var(Var),
    Unpack(
        ModuleIdent,
        StructName,
        Option<Vec<BaseType>>,
        Fields<Assign>,
    ),
}
pub type Assign = Spanned<Assign_>;
pub type AssignList = Spanned<Vec<Assign>>;

#[derive(Debug, PartialEq)]
pub enum Bind_ {
    Ignore,
    Var(Var),
    Unpack(ModuleIdent, StructName, Option<Vec<BaseType>>, Fields<Bind>),
}
pub type Bind = Spanned<Bind_>;
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
pub enum ExpDotted_ {
    Exp(Box<Exp>),
    Dot(Box<ExpDotted>, Field),
}
pub type ExpDotted = Spanned<ExpDotted_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum BuiltinFunction_ {
    MoveToSender(Option<BaseType>),
    MoveFrom(Option<BaseType>),
    BorrowGlobal(bool, Option<BaseType>),
    Exists(Option<BaseType>),
    Freeze(Option<BaseType>),
    /* TODO move these to a native module
     * GetHeight,
     * GetMaxGasPrice,
     * GetMaxGasUnits,
     * GetPublicKey,
     * GetSender,
     * GetSequenceNumber,
     * EmitEvent, */
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    InferredNum(u128),
    Move(Var),
    Copy(Var),
    Use(Var),

    ModuleCall(ModuleIdent, FunctionName, Option<Vec<BaseType>>, Box<Exp>),
    Builtin(BuiltinFunction, Box<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop(Box<Exp>),
    Block(Sequence),

    Assign(AssignList, Box<Exp>),
    FieldMutate(ExpDotted, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),

    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    Pack(ModuleIdent, StructName, Option<Vec<BaseType>>, Fields<Exp>),
    ExpList(Vec<Exp>),
    Unit,

    DerefBorrow(ExpDotted),
    Borrow(bool, ExpDotted),

    Cast(Box<Exp>, Type),
    Annotate(Box<Exp>, Type),

    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

pub type Sequence = VecDeque<SequenceItem>;
#[derive(Debug, PartialEq)]
pub enum SequenceItem_ {
    Seq(Exp),
    Declare(BindList, Option<Type>),
    Bind(BindList, Exp),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// impls
//**************************************************************************************************

impl BuiltinTypeName_ {
    pub const ADDRESS: &'static str = "address";
    pub const U_8: &'static str = "u8";
    pub const U_64: &'static str = "u64";
    pub const U_128: &'static str = "u128";
    pub const BOOL: &'static str = "bool";
    pub const BYTE_ARRAY: &'static str = "bytearray";
    pub const VECTOR: &'static str = "vector";

    pub fn all_names() -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        s.insert(Self::ADDRESS);
        s.insert(Self::U_8);
        s.insert(Self::U_64);
        s.insert(Self::U_128);
        s.insert(Self::BOOL);
        s.insert(Self::BYTE_ARRAY);
        s.insert(Self::VECTOR);
        s
    }

    pub fn numeric() -> BTreeSet<BuiltinTypeName_> {
        use BuiltinTypeName_ as BT;
        let mut s = BTreeSet::new();
        s.insert(BT::U8);
        s.insert(BT::U64);
        s.insert(BT::U128);
        s
    }

    pub fn bits() -> BTreeSet<BuiltinTypeName_> {
        use BuiltinTypeName_ as BT;
        BT::numeric()
    }

    pub fn ordered() -> BTreeSet<BuiltinTypeName_> {
        Self::bits()
    }

    pub fn is_numeric(&self) -> bool {
        Self::numeric().contains(self)
    }

    pub fn resolve(name_str: &str) -> Option<Self> {
        use BuiltinTypeName_ as BT;
        match name_str {
            BT::ADDRESS => Some(BT::Address),
            BT::U_8 => Some(BT::U8),
            BT::U_64 => Some(BT::U64),
            BT::U_128 => Some(BT::U128),
            BT::BOOL => Some(BT::Bool),
            BT::BYTE_ARRAY => Some(BT::Bytearray),
            BT::VECTOR => Some(BT::Vector),
            _ => None,
        }
    }

    pub fn tparam_constraints(&self, loc: Loc) -> Vec<Kind> {
        use BuiltinTypeName_::*;
        // Match here to make sure this function is fixed when collections are added
        match self {
            Address | U8 | U64 | U128 | Bool | Bytearray => vec![],
            Vector => vec![Spanned::new(loc, Kind_::Unknown)],
        }
    }
}

impl TParamID {
    pub fn next() -> TParamID {
        TParamID(Counter::next())
    }
}

impl TVar {
    pub fn next() -> TVar {
        TVar(Counter::next())
    }
}

impl BuiltinFunction_ {
    pub const MOVE_TO_SENDER: &'static str = "move_to_sender";
    pub const MOVE_FROM: &'static str = "move_from";
    pub const BORROW_GLOBAL: &'static str = "borrow_global";
    pub const BORROW_GLOBAL_MUT: &'static str = "borrow_global_mut";
    pub const EXISTS: &'static str = "exists";
    pub const FREEZE: &'static str = "freeze";

    pub fn all_names() -> BTreeSet<&'static str> {
        let mut s = BTreeSet::new();
        s.insert(Self::MOVE_TO_SENDER);
        s.insert(Self::MOVE_FROM);
        s.insert(Self::BORROW_GLOBAL);
        s.insert(Self::BORROW_GLOBAL_MUT);
        s.insert(Self::EXISTS);
        s.insert(Self::FREEZE);
        s
    }

    pub fn resolve(name_str: &str, arg: Option<BaseType>) -> Option<Self> {
        use BuiltinFunction_ as BF;
        match name_str {
            BF::MOVE_TO_SENDER => Some(BF::MoveToSender(arg)),
            BF::MOVE_FROM => Some(BF::MoveFrom(arg)),
            BF::BORROW_GLOBAL => Some(BF::BorrowGlobal(false, arg)),
            BF::BORROW_GLOBAL_MUT => Some(BF::BorrowGlobal(true, arg)),
            BF::EXISTS => Some(BF::Exists(arg)),
            BF::FREEZE => Some(BF::Freeze(arg)),
            _ => None,
        }
    }
}

impl BaseType_ {
    pub fn anything(loc: Loc) -> BaseType {
        sp(loc, BaseType_::Anything)
    }

    pub fn builtin_(loc: Loc, b_: BuiltinTypeName_, ty_args: Vec<BaseType>) -> BaseType_ {
        use BuiltinTypeName_::*;

        let kind = match b_ {
            U8 | U64 | U128 | Address | Bool | Bytearray => Some(sp(loc, Kind_::Unrestricted)),
            Vector => None,
        };
        let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
        BaseType_::Apply(kind, n, ty_args)
    }

    pub fn builtin(loc: Loc, b_: BuiltinTypeName_, ty_args: Vec<BaseType>) -> BaseType {
        sp(loc, Self::builtin_(loc, b_, ty_args))
    }

    pub fn bool(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Bool, vec![])
    }

    pub fn address(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Address, vec![])
    }

    pub fn u8(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::U8, vec![])
    }

    pub fn u64(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::U64, vec![])
    }

    pub fn u128(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::U128, vec![])
    }

    pub fn bytearray(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Bytearray, vec![])
    }

    pub fn builtin_name(&self) -> Option<&BuiltinTypeName> {
        match self {
            BaseType_::Apply(_, sp!(_, TypeName_::Builtin(b)), _) => Some(b),
            _ => None,
        }
    }
}

impl SingleType_ {
    pub fn base(sp!(loc, b_): BaseType) -> SingleType {
        sp(loc, SingleType_::Base(sp(loc, b_)))
    }

    pub fn anything(loc: Loc) -> SingleType {
        Self::base(BaseType_::anything(loc))
    }

    pub fn bool(loc: Loc) -> SingleType {
        Self::base(BaseType_::bool(loc))
    }

    pub fn address(loc: Loc) -> SingleType {
        Self::base(BaseType_::address(loc))
    }

    pub fn u8(loc: Loc) -> SingleType {
        Self::base(BaseType_::u8(loc))
    }

    pub fn u64(loc: Loc) -> SingleType {
        Self::base(BaseType_::u64(loc))
    }

    pub fn u128(loc: Loc) -> SingleType {
        Self::base(BaseType_::u128(loc))
    }

    pub fn builtin_name(&self) -> Option<&BuiltinTypeName> {
        match self {
            SingleType_::Ref(_, _) => None,
            SingleType_::Base(b) => b.value.builtin_name(),
        }
    }
}

impl Type_ {
    pub fn base(b: BaseType) -> Type {
        Self::single(SingleType_::base(b))
    }

    pub fn single(sp!(loc, s_): SingleType) -> Type {
        sp(loc, Type_::Single(sp(loc, s_)))
    }

    pub fn anything(loc: Loc) -> Type {
        Self::single(SingleType_::anything(loc))
    }

    pub fn bool(loc: Loc) -> Type {
        Self::single(SingleType_::bool(loc))
    }

    pub fn address(loc: Loc) -> Type {
        Self::single(SingleType_::address(loc))
    }

    pub fn u8(loc: Loc) -> Type {
        Self::single(SingleType_::u8(loc))
    }

    pub fn u64(loc: Loc) -> Type {
        Self::single(SingleType_::u64(loc))
    }

    pub fn u128(loc: Loc) -> Type {
        Self::single(SingleType_::u128(loc))
    }

    pub fn builtin_name(&self) -> Option<&BuiltinTypeName> {
        match self {
            Type_::Unit | Type_::Multiple(_) => None,
            Type_::Single(s) => s.value.builtin_name(),
        }
    }
}

impl Value_ {
    pub fn type_(&self, loc: Loc) -> BaseType {
        use Value_::*;
        match self {
            Address(_) => BaseType_::address(loc),
            U8(_) => BaseType_::u8(loc),
            U64(_) => BaseType_::u64(loc),
            U128(_) => BaseType_::u128(loc),
            Bool(_) => BaseType_::bool(loc),
            Bytearray(_) => BaseType_::bytearray(loc),
        }
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for BuiltinTypeName_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use BuiltinTypeName_ as BT;
        write!(
            f,
            "{}",
            match self {
                BT::Address => BT::ADDRESS,
                BT::U8 => BT::U_8,
                BT::U64 => BT::U_64,
                BT::U128 => BT::U_128,
                BT::Bool => BT::BOOL,
                BT::Bytearray => BT::BYTE_ARRAY,
                BT::Vector => BT::VECTOR,
            }
        )
    }
}

impl fmt::Display for TypeName_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use TypeName_::*;
        match self {
            Builtin(b) => write!(f, "{}", b),
            ModuleType(m, n) => write!(f, "{}::{}", m, n),
        }
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Program { modules, main } = self;
        for (m, mdef) in modules {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        if let Some((addr, n, fdef)) = main {
            w.writeln(&format!("address {}:", addr));
            (n.clone(), fdef).ast_debug(w);
        }
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            is_source_module,
            dependency_order,
            uses,
            structs,
            functions,
        } = self;
        if *is_source_module {
            w.writeln("library module")
        } else {
            w.writeln("source module")
        }
        w.writeln(&format!("dependency order #{}", dependency_order));
        if !uses.is_empty() {
            w.writeln("uses: ");
            w.indent(2, |w| w.comma(uses, |w, (m, _)| w.write(&format!("{}", m))))
        }
        for sdef in structs {
            sdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions {
            fdef.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (StructName, &StructDefinition) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            StructDefinition {
                resource_opt,
                type_parameters,
                fields,
            },
        ) = self;
        if let StructFields::Native(_) = fields {
            w.write("native ");
        }
        if resource_opt.is_some() {
            w.write("resource ");
        }
        w.write(&format!("struct {}", name));
        type_parameters.ast_debug(w);
        if let StructFields::Defined(fields) = fields {
            w.block(|w| {
                w.list(fields, ",", |w, (f, idx_st)| {
                    let (idx, st) = idx_st;
                    w.write(&format!("{}#{}: ", idx, f));
                    st.ast_debug(w);
                    true
                })
            })
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                visibility,
                signature,
                acquires,
                body,
            },
        ) = self;
        visibility.ast_debug(w);
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("{}", name));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires, |w, s| w.write(&format!("{}", s)));
            w.write(" ")
        }
        match &body.value {
            FunctionBody_::Defined(body) => w.block(|w| body.ast_debug(w)),
            FunctionBody_::Native => w.writeln(";"),
        }
    }
}

impl AstDebug for FunctionSignature {
    fn ast_debug(&self, w: &mut AstWriter) {
        let FunctionSignature {
            type_parameters,
            parameters,
            return_type,
        } = self;
        type_parameters.ast_debug(w);
        w.write("(");
        w.comma(parameters, |w, (v, st)| {
            w.write(&format!("{}: ", v));
            st.ast_debug(w);
        });
        w.write("): ");
        return_type.ast_debug(w)
    }
}

impl AstDebug for Vec<TParam> {
    fn ast_debug(&self, w: &mut AstWriter) {
        if !self.is_empty() {
            w.write("<");
            w.comma(self, |w, tp| tp.ast_debug(w));
            w.write(">")
        }
    }
}

impl AstDebug for BuiltinTypeName_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&format!("{}", self));
    }
}

impl AstDebug for TypeName_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            TypeName_::Builtin(bt) => bt.ast_debug(w),
            TypeName_::ModuleType(m, s) => w.write(&format!("{}::{}", m, s)),
        }
    }
}

impl AstDebug for TParam {
    fn ast_debug(&self, w: &mut AstWriter) {
        let TParam { id, debug, kind } = self;
        w.write(&format!("{}#{}", debug, id.0));
        match &kind.value {
            Kind_::Unknown => (),
            Kind_::Resource => w.write(": resource"),
            Kind_::Affine => w.write(": copyable"),
            Kind_::Unrestricted => panic!("ICE 'unrestricted' kind constraint"),
        }
    }
}

impl AstDebug for BaseType_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            BaseType_::Param(tp) => tp.ast_debug(w),
            BaseType_::Apply(k_opt, m, ss) => {
                let w_ty = move |w: &mut AstWriter| {
                    m.ast_debug(w);
                    if !ss.is_empty() {
                        w.write("<");
                        ss.ast_debug(w);
                        w.write(">");
                    }
                };
                match k_opt {
                    None => w_ty(w),
                    Some(k) => w.annotate(w_ty, k),
                }
            }
            BaseType_::Var(tv) => w.write(&format!("#{}", tv.0)),
            BaseType_::Anything => w.write("_"),
        }
    }
}

impl AstDebug for SingleType_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            SingleType_::Base(b) => b.ast_debug(w),
            SingleType_::Ref(mut_, s) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                s.ast_debug(w)
            }
        }
    }
}

impl AstDebug for Type_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            Type_::Unit => w.write("()"),
            Type_::Single(s) => s.ast_debug(w),
            Type_::Multiple(ss) => {
                w.write("(");
                ss.ast_debug(w);
                w.write(")")
            }
        }
    }
}

impl AstDebug for Vec<SingleType> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, s| s.ast_debug(w))
    }
}

impl AstDebug for Vec<BaseType> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, s| s.ast_debug(w))
    }
}

impl AstDebug for VecDeque<SequenceItem> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.semicolon(self, |w, item| item.ast_debug(w))
    }
}

impl AstDebug for SequenceItem_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use SequenceItem_ as I;
        match self {
            I::Seq(e) => e.ast_debug(w),
            I::Declare(sp!(_, bs), ty_opt) => {
                w.write("let ");
                bs.ast_debug(w);
                if let Some(ty) = ty_opt {
                    ty.ast_debug(w)
                }
            }
            I::Bind(sp!(_, bs), e) => {
                w.write("let ");
                bs.ast_debug(w);
                w.write(" = ");
                e.ast_debug(w);
            }
        }
    }
}

impl AstDebug for Exp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Exp_ as E;
        match self {
            E::Unit => w.write("()"),
            E::Value(v) => v.ast_debug(w),
            E::InferredNum(u) => w.write(&format!("{}", u)),
            E::Move(v) => w.write(&format!("move {}", v)),
            E::Copy(v) => w.write(&format!("copy {}", v)),
            E::Use(v) => w.write(&format!("{}", v)),
            E::ModuleCall(m, f, tys_opt, rhs) => {
                w.write(&format!("{}::{}", m, f));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("(");
                rhs.ast_debug(w);
                w.write(")");
            }
            E::Builtin(bf, rhs) => {
                bf.ast_debug(w);
                w.write("(");
                rhs.ast_debug(w);
                w.write(")");
            }
            E::Pack(m, s, tys_opt, fields) => {
                w.write(&format!("{}::{}", m, s));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("{");
                w.comma(fields, |w, (f, idx_e)| {
                    let (idx, e) = idx_e;
                    w.write(&format!("{}#{}: ", idx, f));
                    e.ast_debug(w);
                });
                w.write("}");
            }
            E::IfElse(b, t, f) => {
                w.write("if (");
                b.ast_debug(w);
                w.write(") ");
                t.ast_debug(w);
                w.write(" else ");
                f.ast_debug(w);
            }
            E::While(b, e) => {
                w.write("while (");
                b.ast_debug(w);
                w.write(")");
                e.ast_debug(w);
            }
            E::Loop(e) => {
                w.write("loop ");
                e.ast_debug(w);
            }
            E::Block(seq) => w.block(|w| seq.ast_debug(w)),
            E::ExpList(es) => {
                w.write("(");
                w.comma(es, |w, e| e.ast_debug(w));
                w.write(")");
            }

            E::Assign(sp!(_, lvalues), rhs) => {
                lvalues.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }
            E::FieldMutate(ed, rhs) => {
                ed.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }
            E::Mutate(lhs, rhs) => {
                w.write("*");
                lhs.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }

            E::Return(e) => {
                w.write("return ");
                e.ast_debug(w);
            }
            E::Abort(e) => {
                w.write("abort ");
                e.ast_debug(w);
            }
            E::Break => w.write("break"),
            E::Continue => w.write("continue"),
            E::Dereference(e) => {
                w.write("*");
                e.ast_debug(w)
            }
            E::UnaryExp(op, e) => {
                op.ast_debug(w);
                w.write(" ");
                e.ast_debug(w);
            }
            E::BinopExp(l, op, r) => {
                l.ast_debug(w);
                w.write(" ");
                op.ast_debug(w);
                w.write(" ");
                r.ast_debug(w)
            }
            E::Borrow(mut_, e) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
            }
            E::DerefBorrow(ed) => {
                w.write("(&*)");
                ed.ast_debug(w)
            }
            E::Cast(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(" as ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Annotate(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(": ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::UnresolvedError => w.write("_|_"),
        }
    }
}

impl AstDebug for BuiltinFunction_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use BuiltinFunction_ as F;
        let (n, bt) = match self {
            F::MoveToSender(bt) => (F::MOVE_TO_SENDER, bt),
            F::MoveFrom(bt) => (F::MOVE_FROM, bt),
            F::BorrowGlobal(true, bt) => (F::BORROW_GLOBAL_MUT, bt),
            F::BorrowGlobal(false, bt) => (F::BORROW_GLOBAL, bt),
            F::Exists(bt) => (F::EXISTS, bt),
            F::Freeze(bt) => (F::FREEZE, bt),
        };
        w.write(n);
        if let Some(bt) = bt {
            w.write("<");
            bt.ast_debug(w);
            w.write(">");
        }
    }
}

impl AstDebug for ExpDotted_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use ExpDotted_ as D;
        match self {
            D::Exp(e) => e.ast_debug(w),
            D::Dot(e, n) => {
                e.ast_debug(w);
                w.write(&format!(".{}", n))
            }
        }
    }
}

impl AstDebug for Vec<Bind> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let parens = self.len() != 1;
        if parens {
            w.write("(");
        }
        w.comma(self, |w, b| b.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for Bind_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Bind_ as B;
        match self {
            B::Ignore => w.write("_"),
            B::Var(v) => w.write(&format!("{}", v)),
            B::Unpack(m, s, tys_opt, fields) => {
                w.write(&format!("{}::{}", m, s));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("{");
                w.comma(fields, |w, (f, idx_b)| {
                    let (idx, b) = idx_b;
                    w.write(&format!("{}#{}: ", idx, f));
                    b.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}

impl AstDebug for Vec<Assign> {
    fn ast_debug(&self, w: &mut AstWriter) {
        let parens = self.len() != 1;
        if parens {
            w.write("(");
        }
        w.comma(self, |w, a| a.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for Assign_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Assign_ as A;
        match self {
            A::Ignore => w.write("_"),
            A::Var(v) => w.write(&format!("{}", v)),
            A::Unpack(m, s, tys_opt, fields) => {
                w.write(&format!("{}::{}", m, s));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("{");
                w.comma(fields, |w, (f, idx_a)| {
                    let (idx, a) = idx_a;
                    w.write(&format!("{}#{}: ", idx, f));
                    a.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}
