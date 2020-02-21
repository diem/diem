// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::shared::{ast_debug::*, sp, Address, Identifier, Loc, Name, Spanned, TName};
use std::fmt;

macro_rules! new_name {
    ($n:ident) => {
        #[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
        pub struct $n(pub Name);

        impl TName for $n {
            type Key = String;
            type Loc = Loc;

            fn drop_loc(self) -> (Loc, String) {
                (self.0.loc, self.0.value)
            }

            fn clone_drop_loc(&self) -> (Loc, String) {
                (self.0.loc, self.0.value.clone())
            }

            fn add_loc(loc: Loc, key: String) -> Self {
                $n(sp(loc, key))
            }
        }

        impl Identifier for $n {
            fn value(&self) -> &str {
                &self.0.value
            }
            fn loc(&self) -> Loc {
                self.0.loc
            }
        }

        impl fmt::Display for $n {
            fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", &self.0)
            }
        }
    };
}

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug)]
pub struct Program {
    pub source_definitions: Vec<FileDefinition>,
    pub lib_definitions: Vec<FileDefinition>,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum FileDefinition {
    Modules(Vec<ModuleOrAddress>),
    Main(Main),
}

#[derive(Debug)]
pub enum ModuleOrAddress {
    Module(ModuleDefinition),
    Address(Loc, Address),
}

#[derive(Debug)]
pub struct Main {
    pub uses: Vec<(ModuleIdent, Option<ModuleName>)>,
    pub function: Function,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

new_name!(ModuleName);

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct ModuleIdent_ {
    pub name: ModuleName,
    pub address: Address,
}
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct ModuleIdent(pub Spanned<ModuleIdent_>);

#[derive(Debug)]
pub struct ModuleDefinition {
    pub uses: Vec<(ModuleIdent, Option<ModuleName>)>,
    pub name: ModuleName,
    pub structs: Vec<StructDefinition>,
    pub functions: Vec<Function>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

new_name!(Field);
new_name!(StructName);

pub type ResourceLoc = Option<Loc>;

#[derive(Debug, PartialEq)]
pub struct StructDefinition {
    pub resource_opt: ResourceLoc,
    pub name: StructName,
    pub type_parameters: Vec<(Name, Kind)>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq)]
pub enum StructFields {
    Defined(Vec<(Field, SingleType)>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

new_name!(FunctionName);

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, Kind)>,
    pub parameters: Vec<(Var, SingleType)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionVisibility {
    Public(Loc),
    Internal,
}

#[derive(PartialEq, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(PartialEq, Debug)]
// (public?) foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn {
//    body
//  }
// (public?) native foo<T1(: copyable?), ..., TN(: copyable?)>(x1: t1, ..., xn: tn): t1 * ... * tn;
pub struct Function {
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: Vec<ModuleAccess>,
    pub name: FunctionName,
    pub body: FunctionBody,
}

//**************************************************************************************************
// Types
//**************************************************************************************************

// A ModuleAccess references something from a module, either a struct or a function.
#[derive(Debug, PartialEq)]
pub enum ModuleAccess_ {
    // N
    Name(Name),
    // M.S
    ModuleAccess(ModuleName, Name),
    // OxADDR.M.S
    QualifiedModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Kind_ {
    // Kind representing all types
    Unknown,
    // Linear resource types
    Resource,
    // Explicitly copyable types
    Affine,
    // Implicitly copyable types
    Unrestricted,
}
pub type Kind = Spanned<Kind_>;

#[derive(Debug, PartialEq)]
pub enum SingleType_ {
    // N
    // N<t1, ... , tn>
    Apply(ModuleAccess, Vec<SingleType>),
    // &t
    // &mut t
    Ref(bool, Box<SingleType>),
}
pub type SingleType = Spanned<SingleType_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    // ()
    Unit,
    // t
    Single(SingleType),
    // (t1, t2, ... , tn)
    // Used for return values and expression blocks
    Multiple(Vec<SingleType>),
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

new_name!(Var);

#[derive(Debug, PartialEq)]
pub enum Bind_ {
    // x
    Var(Var),
    // T { f1: b1, ... fn: bn }
    // T<t1, ... , tn> { f1: b1, ... fn: bn }
    Unpack(ModuleAccess, Option<Vec<SingleType>>, Vec<(Field, Bind)>),
}
pub type Bind = Spanned<Bind_>;
// b1, ..., bn
pub type BindList = Spanned<Vec<Bind>>;

#[derive(Debug, PartialEq)]
pub enum Value_ {
    // 0x<hex representation up to 64 digits with padding 0s>
    Address(Address),
    // <num>u8
    U8(u8),
    // <num>u64
    U64(u64),
    // <num>u128
    U128(u128),
    // true
    // false
    Bool(bool),
    Bytearray(Vec<u8>),
}
pub type Value = Spanned<Value_>;

#[derive(Debug, PartialEq)]
pub enum UnaryOp_ {
    // !
    Not,
}
pub type UnaryOp = Spanned<UnaryOp_>;

#[derive(Debug, PartialEq)]
pub enum BinOp_ {
    // Int ops
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // %
    Mod,
    // /
    Div,
    // |
    BitOr,
    // &
    BitAnd,
    // ^
    Xor,
    // <<
    Shl,
    // >>
    Shr,

    // Bool ops
    // &&
    And,
    // ||
    Or,

    // Compare Ops
    // ==
    Eq,
    // !=
    Neq,
    // <
    Lt,
    // >
    Gt,
    // <=
    Le,
    // >=
    Ge,
}
pub type BinOp = Spanned<BinOp_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    // <num>
    InferredNum(u128),
    // move(x)
    Move(Var),
    // copy(x)
    Copy(Var),
    // n
    Name(Name),
    // ::n(e)
    GlobalCall(Name, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),

    // f(earg,*)
    Call(ModuleAccess, Option<Vec<SingleType>>, Spanned<Vec<Exp>>),

    // tn {f1: e1, ... , f_n: e_n }
    Pack(ModuleAccess, Option<Vec<SingleType>>, Vec<(Field, Exp)>),

    // if (eb) et else ef
    IfElse(Box<Exp>, Box<Exp>, Option<Box<Exp>>),
    // while (eb) eloop
    While(Box<Exp>, Box<Exp>),
    // loop eloop
    Loop(Box<Exp>),

    // { seq }
    Block(Sequence),
    // (e1, ..., en)
    ExpList(Vec<Exp>),
    // ()
    Unit,

    // a = e
    Assign(Box<Exp>, Box<Exp>),

    // return e
    Return(Option<Box<Exp>>),
    // abort e
    Abort(Box<Exp>),
    // break
    Break,
    // continue
    Continue,

    // *e
    Dereference(Box<Exp>),
    // op e
    UnaryExp(UnaryOp, Box<Exp>),
    // e1 op e2
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    // &e
    // &mut e
    Borrow(bool, Box<Exp>),
    // e.f
    Dot(Box<Exp>, Name),

    // (e as t)
    Cast(Box<Exp>, Type),
    // (e: t)
    Annotate(Box<Exp>, Type),

    // Internal node marking an error was added to the error list
    // This is here so the pass can continue even when an error is hit
    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

// { e1; ... ; en }
// { e1; ... ; en; }
// The Loc field holds the source location of the final semicolon, if there is one.
pub type Sequence = (Vec<SequenceItem>, Option<Loc>, Box<Option<Exp>>);
#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SequenceItem_ {
    // e;
    Seq(Box<Exp>),
    // let b : t = e;
    // let b = e;
    Declare(BindList, Option<Type>),
    // let b : t = e;
    // let b = e;
    Bind(BindList, Option<Type>, Box<Exp>),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// Loc
//**************************************************************************************************

impl TName for ModuleIdent {
    type Key = (Address, String);
    type Loc = (Loc, Loc);
    fn drop_loc(self) -> ((Loc, Loc), (Address, String)) {
        let inner = self.0.value;
        let (nloc, name_) = inner.name.drop_loc();
        ((self.0.loc, nloc), (inner.address, name_))
    }
    fn clone_drop_loc(&self) -> ((Loc, Loc), (Address, String)) {
        let (nloc, name_) = self.0.value.name.clone_drop_loc();
        ((self.0.loc, nloc), (self.0.value.address, name_))
    }
    fn add_loc(locs: (Loc, Loc), key: (Address, String)) -> ModuleIdent {
        let (iloc, nloc) = locs;
        let (address, name_str) = key;
        let name = ModuleName::add_loc(nloc, name_str);
        let ident_ = ModuleIdent_ { address, name };
        ModuleIdent(sp(iloc, ident_))
    }
}

//**************************************************************************************************
// Impl
//**************************************************************************************************

impl ModuleIdent {
    pub fn loc(&self) -> Loc {
        self.0.loc
    }
}

impl ModuleName {
    pub const SELF_NAME: &'static str = "Self";
}

impl FunctionName {
    pub const MAIN_NAME: &'static str = "main";
}

impl Var {
    pub fn starts_with_underscore(&self) -> bool {
        self.0.value.starts_with('_')
    }
}

impl Kind_ {
    pub const VALUE_CONSTRAINT: &'static str = "copyable";
    pub const RESOURCE_CONSTRAINT: &'static str = "resource";

    pub fn is_resourceful(&self) -> bool {
        match self {
            Kind_::Affine | Kind_::Unrestricted => false,
            Kind_::Resource | Kind_::Unknown => true,
        }
    }
}

impl Type_ {
    pub fn unit(loc: Loc) -> Type {
        sp(loc, Type_::Unit)
    }
}

impl UnaryOp_ {
    pub const NOT: &'static str = "!";

    pub fn symbol(&self) -> &'static str {
        use UnaryOp_ as U;
        match self {
            U::Not => U::NOT,
        }
    }

    pub fn is_pure(&self) -> bool {
        use UnaryOp_ as U;
        match self {
            U::Not => true,
        }
    }
}

impl BinOp_ {
    pub const ADD: &'static str = "+";
    pub const SUB: &'static str = "-";
    pub const MUL: &'static str = "*";
    pub const MOD: &'static str = "%";
    pub const DIV: &'static str = "/";
    pub const BIT_OR: &'static str = "|";
    pub const BIT_AND: &'static str = "&";
    pub const XOR: &'static str = "^";
    pub const SHL: &'static str = "<<";
    pub const SHR: &'static str = ">>";
    pub const AND: &'static str = "&&";
    pub const OR: &'static str = "||";
    pub const EQ: &'static str = "==";
    pub const NEQ: &'static str = "!=";
    pub const LT: &'static str = "<";
    pub const GT: &'static str = ">";
    pub const LE: &'static str = "<=";
    pub const GE: &'static str = ">=";

    pub fn symbol(&self) -> &'static str {
        use BinOp_ as B;
        match self {
            B::Add => B::ADD,
            B::Sub => B::SUB,
            B::Mul => B::MUL,
            B::Mod => B::MOD,
            B::Div => B::DIV,
            B::BitOr => B::BIT_OR,
            B::BitAnd => B::BIT_AND,
            B::Xor => B::XOR,
            B::Shl => B::SHL,
            B::Shr => B::SHR,
            B::And => B::AND,
            B::Or => B::OR,
            B::Eq => B::EQ,
            B::Neq => B::NEQ,
            B::Lt => B::LT,
            B::Gt => B::GT,
            B::Le => B::LE,
            B::Ge => B::GE,
        }
    }

    pub fn is_pure(&self) -> bool {
        use BinOp_ as B;
        match self {
            B::Add | B::Sub | B::Mul | B::Mod | B::Div | B::Shl | B::Shr => false,
            B::BitOr
            | B::BitAnd
            | B::Xor
            | B::And
            | B::Or
            | B::Eq
            | B::Neq
            | B::Lt
            | B::Gt
            | B::Le
            | B::Ge => true,
        }
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleIdent {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}::{}", self.0.value.address, &self.0.value.name)
    }
}

impl fmt::Display for UnaryOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for BinOp_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write("------ Lib Defs: ------");
        for src in &self.source_definitions {
            src.ast_debug(w);
        }
        w.new_line();
        w.write("------ Source Defs: ------");
        for src in &self.source_definitions {
            src.ast_debug(w);
        }
    }
}

impl AstDebug for FileDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            FileDefinition::Main(m) => m.ast_debug(w),
            FileDefinition::Modules(moras) => {
                for mora in moras {
                    mora.ast_debug(w);
                    w.new_line();
                    w.new_line();
                }
            }
        }
    }
}

impl AstDebug for Main {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Main { uses, function } = self;
        uses.ast_debug(w);
        function.ast_debug(w);
    }
}

impl AstDebug for ModuleOrAddress {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            ModuleOrAddress::Address(_, addr) => {
                w.writeln(&format!("address {}:", addr));
            }
            ModuleOrAddress::Module(m) => m.ast_debug(w),
        }
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            uses,
            name,
            structs,
            functions,
        } = self;
        w.write(&format!("module {}", name));
        w.block(|w| {
            uses.ast_debug(w);
            for sdef in structs {
                sdef.ast_debug(w);
                w.new_line();
            }
            for fdef in functions {
                fdef.ast_debug(w);
                w.new_line();
            }
        });
    }
}

impl AstDebug for Vec<(ModuleIdent, Option<ModuleName>)> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.semicolon(self, |w, item| item.ast_debug(w));
        w.writeln(";");
    }
}

impl AstDebug for (ModuleIdent, Option<ModuleName>) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (m, alias_opt) = self;
        w.write(&format!("use {}", m));
        if let Some(alias) = alias_opt {
            w.write(&format!(" as {}", alias))
        }
        w.writeln(";");
    }
}

impl AstDebug for StructDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let StructDefinition {
            resource_opt,
            name,
            type_parameters,
            fields,
        } = self;
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
                w.semicolon(fields, |w, (f, st)| {
                    w.write(&format!("{}: ", f));
                    st.ast_debug(w);
                })
            })
        }
    }
}

impl AstDebug for Function {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Function {
            visibility,
            signature,
            acquires,
            name,
            body,
        } = self;
        visibility.ast_debug(w);
        if let FunctionBody_::Native = &body.value {
            w.write("native ");
        }
        w.write(&format!("{}", name));
        signature.ast_debug(w);
        if !acquires.is_empty() {
            w.write(" acquires ");
            w.comma(acquires, |w, m| m.ast_debug(w));
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined(body) => w.block(|w| body.ast_debug(w)),
            FunctionBody_::Native => w.writeln(";"),
        }
    }
}

impl AstDebug for FunctionVisibility {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            FunctionVisibility::Internal => (),
            FunctionVisibility::Public(_) => w.write("public "),
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
        w.write(")");
        w.write(": ");
        return_type.ast_debug(w)
    }
}

impl AstDebug for Vec<(Name, Kind)> {
    fn ast_debug(&self, w: &mut AstWriter) {
        if !self.is_empty() {
            w.write("<");
            w.comma(self, |w, tp| tp.ast_debug(w));
            w.write(">")
        }
    }
}

impl AstDebug for (Name, Kind) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (n, k) = self;
        w.write(&n.value);
        match &k.value {
            Kind_::Unknown => (),
            Kind_::Resource | Kind_::Affine => {
                w.write(": ");
                k.ast_debug(w)
            }
            Kind_::Unrestricted => panic!("ICE 'unrestricted' kind constraint"),
        }
    }
}

impl AstDebug for Kind_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(match self {
            Kind_::Unknown => "unknown",
            Kind_::Resource => "resource",
            Kind_::Affine => "copyable",
            Kind_::Unrestricted => "unrestricted",
        })
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

impl AstDebug for SingleType_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            SingleType_::Apply(m, ss) => {
                m.ast_debug(w);
                if !ss.is_empty() {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
            }
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

impl AstDebug for Vec<SingleType> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, s| s.ast_debug(w))
    }
}

impl AstDebug for ModuleAccess_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&match self {
            ModuleAccess_::Name(n) => format!("{}", n),
            ModuleAccess_::ModuleAccess(m, n) => format!("{}::{}", m, n),
            ModuleAccess_::QualifiedModuleAccess(m, n) => format!("{}::{}", m, n),
        })
    }
}

impl AstDebug for (Vec<SequenceItem>, Option<Loc>, Box<Option<Exp>>) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (seq, _, last_e) = self;
        w.semicolon(seq, |w, item| item.ast_debug(w));
        if !seq.is_empty() {
            w.writeln(";")
        }
        if let Some(e) = &**last_e {
            e.ast_debug(w)
        }
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
            I::Bind(sp!(_, bs), ty_opt, e) => {
                w.write("let ");
                bs.ast_debug(w);
                if let Some(ty) = ty_opt {
                    ty.ast_debug(w)
                }
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
            E::Name(n) => w.write(&format!("{}", n)),
            E::GlobalCall(n, tys_opt, sp!(_, rhs)) => {
                w.write(&format!("::{}", n));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("(");
                w.comma(rhs, |w, e| e.ast_debug(w));
                w.write(")");
            }
            E::Call(ma, tys_opt, sp!(_, rhs)) => {
                ma.ast_debug(w);
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("(");
                w.comma(rhs, |w, e| e.ast_debug(w));
                w.write(")");
            }
            E::Pack(ma, tys_opt, fields) => {
                ma.ast_debug(w);
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("{");
                w.comma(fields, |w, (f, e)| {
                    w.write(&format!("{}: ", f));
                    e.ast_debug(w);
                });
                w.write("}");
            }
            E::IfElse(b, t, f_opt) => {
                w.write("if (");
                b.ast_debug(w);
                w.write(") ");
                t.ast_debug(w);
                if let Some(f) = f_opt {
                    w.write(" else ");
                    f.ast_debug(w);
                }
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
            E::Assign(lvalue, rhs) => {
                lvalue.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }
            E::Return(e) => {
                w.write("return");
                if let Some(v) = e {
                    w.write(" ");
                    v.ast_debug(w);
                }
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
            E::Dot(e, n) => {
                e.ast_debug(w);
                w.write(&format!(".{}", n));
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

impl AstDebug for BinOp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&format!("{}", self));
    }
}

impl AstDebug for UnaryOp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&format!("{}", self));
    }
}

impl AstDebug for Value_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Value_ as V;
        w.write(&match self {
            V::Address(addr) => format!("{}", addr),
            V::U8(u) => format!("{}u8", u),
            V::U64(u) => format!("{}u64", u),
            V::U128(u) => format!("{}u128", u),
            V::Bool(b) => format!("{}", b),
            V::Bytearray(v) => format!("{:?}", v),
        })
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
            B::Var(v) => w.write(&format!("{}", v)),
            B::Unpack(ma, tys_opt, fields) => {
                ma.ast_debug(w);
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
                w.write("{");
                w.comma(fields, |w, (f, b)| {
                    w.write(&format!("{}: ", f));
                    b.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}
