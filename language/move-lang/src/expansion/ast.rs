// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, ModuleIdent, PragmaProperty,
        ResourceLoc, SpecApplyPattern, SpecBlockTarget, SpecConditionKind, StructName, UnaryOp,
        Value, Var,
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
    pub scripts: BTreeMap<String, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug)]
pub struct Script {
    pub loc: Loc,
    pub function_name: FunctionName,
    pub function: Function,
    pub specs: Vec<SpecBlock>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug)]
pub struct ModuleDefinition {
    pub loc: Loc,
    pub is_source_module: bool,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
    pub specs: Vec<SpecBlock>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub type Fields<T> = UniqueMap<Field, (usize, T)>;

#[derive(Debug, PartialEq)]
pub struct StructDefinition {
    pub loc: Loc,
    pub resource_opt: ResourceLoc,
    pub type_parameters: Vec<(Name, Kind)>,
    pub fields: StructFields,
}

#[derive(Debug, PartialEq)]
pub enum StructFields {
    Defined(Fields<Type>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, Kind)>,
    pub parameters: Vec<(Var, Type)>,
    pub return_type: Type,
}

#[derive(PartialEq, Debug)]

pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SpecId(usize);

#[derive(PartialEq, Debug)]
pub struct Function {
    pub loc: Loc,
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: Vec<ModuleAccess>,
    pub body: FunctionBody,
    pub specs: BTreeMap<SpecId, SpecBlock>,
}

//**************************************************************************************************
// Specification Blocks
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub struct SpecBlock_ {
    pub target: SpecBlockTarget,
    pub members: Vec<SpecBlockMember>,
}

pub type SpecBlock = Spanned<SpecBlock_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SpecBlockMember_ {
    Condition {
        kind: SpecConditionKind,
        exp: Exp,
    },
    Function {
        name: FunctionName,
        signature: FunctionSignature,
        body: FunctionBody,
    },
    Variable {
        is_global: bool,
        name: Name,
        type_parameters: Vec<(Name, Kind)>,
        type_: Type,
    },
    Include {
        exp: Exp,
    },
    Apply {
        exp: Exp,
        patterns: Vec<SpecApplyPattern>,
        exclusion_patterns: Vec<SpecApplyPattern>,
    },
    Pragma {
        properties: Vec<PragmaProperty>,
    },
}

pub type SpecBlockMember = Spanned<SpecBlockMember_>;

//**************************************************************************************************
// Types
//**************************************************************************************************

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleAccess_ {
    Name(Name),
    ModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Type_ {
    Unit,
    Multiple(Vec<Type>),
    Apply(ModuleAccess, Vec<Type>),
    Ref(bool, Box<Type>),
    Fun(Vec<Type>, Box<Type>),
    UnresolvedError,
}
pub type Type = Spanned<Type_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub enum LValue_ {
    Var(ModuleAccess, Option<Vec<Type>>),
    Unpack(ModuleAccess, Option<Vec<Type>>, Fields<LValue>),
}
pub type LValue = Spanned<LValue_>;
pub type LValueList_ = Vec<LValue>;
pub type LValueList = Spanned<LValueList_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExpDotted_ {
    Exp(Exp),
    Dot(Box<ExpDotted>, Name),
}
pub type ExpDotted = Spanned<ExpDotted_>;

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    InferredNum(u128),
    Move(Var),
    Copy(Var),

    Name(ModuleAccess, Option<Vec<Type>>),
    GlobalCall(Name, Option<Vec<Type>>, Spanned<Vec<Exp>>),
    Call(ModuleAccess, Option<Vec<Type>>, Spanned<Vec<Exp>>),
    Pack(ModuleAccess, Option<Vec<Type>>, Fields<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop(Box<Exp>),
    Block(Sequence),
    Lambda(LValueList, Box<Exp>), // spec only

    Assign(LValueList, Box<Exp>),
    FieldMutate(Box<ExpDotted>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),

    Return(Box<Exp>),
    Abort(Box<Exp>),
    Break,
    Continue,

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    ExpList(Vec<Exp>),
    Unit,

    Borrow(bool, Box<Exp>),
    ExpDotted(Box<ExpDotted>),
    Index(Box<Exp>, Box<Exp>), // spec only (no mutation needed right now)

    Cast(Box<Exp>, Type),
    Annotate(Box<Exp>, Type),

    Spec(SpecId, BTreeSet<Name>),

    UnresolvedError,
}
pub type Exp = Spanned<Exp_>;

pub type Sequence = VecDeque<SequenceItem>;
#[derive(Debug, PartialEq)]
pub enum SequenceItem_ {
    Seq(Exp),
    Declare(LValueList, Option<Type>),
    Bind(LValueList, Exp),
}
pub type SequenceItem = Spanned<SequenceItem_>;

//**************************************************************************************************
// impls
//**************************************************************************************************

impl SpecId {
    pub fn new(u: usize) -> Self {
        SpecId(u)
    }

    pub fn inner(self) -> usize {
        self.0
    }
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for ModuleAccess_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use ModuleAccess_::*;
        match self {
            Name(n) => write!(f, "{}", n),
            ModuleAccess(m, n) => write!(f, "{}::{}", m, n),
        }
    }
}

impl fmt::Display for Type_ {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        use Type_::*;
        match self {
            UnresolvedError => write!(f, "_"),
            Apply(n, tys) => {
                write!(f, "{}", n)?;
                if !tys.is_empty() {
                    write!(f, "<")?;
                    write!(f, "{}", format_comma(tys))?;
                    write!(f, ">")?;
                }
                Ok(())
            }
            Ref(mut_, ty) => write!(f, "&{}{}", if *mut_ { "mut " } else { "" }, ty),
            Fun(args, result) => write!(f, "({}):{}", format_comma(args), result),
            Unit => write!(f, "()"),
            Multiple(tys) => {
                write!(f, "(")?;
                write!(f, "{}", format_comma(tys))?;
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for SpecId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

//**************************************************************************************************
// Debug
//**************************************************************************************************

impl AstDebug for Program {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Program { modules, scripts } = self;
        for (m, mdef) in modules {
            w.write(&format!("module {}", m));
            w.block(|w| mdef.ast_debug(w));
            w.new_line();
        }

        for (n, s) in scripts {
            w.write(&format!("script {}", n));
            w.block(|w| s.ast_debug(w));
            w.new_line()
        }
    }
}

impl AstDebug for Script {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Script {
            loc: _loc,
            function_name,
            function,
            specs,
        } = self;
        (function_name.clone(), function).ast_debug(w);
        for spec in specs {
            spec.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for ModuleDefinition {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleDefinition {
            loc: _loc,
            is_source_module,
            structs,
            functions,
            specs,
        } = self;
        w.writeln(if *is_source_module {
            "source module"
        } else {
            "library module"
        });
        for sdef in structs {
            sdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions {
            fdef.ast_debug(w);
            w.new_line();
        }
        for spec in specs {
            spec.ast_debug(w);
            w.new_line();
        }
    }
}

impl AstDebug for (StructName, &StructDefinition) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            StructDefinition {
                loc: _loc,
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
                });
            })
        }
    }
}

impl AstDebug for SpecBlock_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(" spec ");
        self.target.ast_debug(w);
        w.write("{");
        w.semicolon(&self.members, |w, m| m.ast_debug(w));
        w.write("}");
    }
}

impl AstDebug for SpecBlockMember_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            SpecBlockMember_::Condition { kind, exp } => {
                kind.ast_debug(w);
                exp.ast_debug(w);
            }
            SpecBlockMember_::Function {
                signature,
                name,
                body,
            } => {
                if let FunctionBody_::Native = &body.value {
                    w.write("native ");
                }
                w.write("define ");
                w.write(&format!("{}", name));
                signature.ast_debug(w);
                match &body.value {
                    FunctionBody_::Defined(body) => w.block(|w| body.ast_debug(w)),
                    FunctionBody_::Native => w.writeln(";"),
                }
            }
            SpecBlockMember_::Variable {
                is_global,
                name,
                type_parameters,
                type_,
            } => {
                if *is_global {
                    w.write("global ");
                } else {
                    w.write("local");
                }
                w.write(&format!("{}", name));
                type_parameters.ast_debug(w);
                w.write(": ");
                type_.ast_debug(w);
            }
            SpecBlockMember_::Include { exp } => {
                w.write("include ");
                exp.ast_debug(w);
            }
            SpecBlockMember_::Apply {
                exp,
                patterns,
                exclusion_patterns,
            } => {
                w.write("apply ");
                exp.ast_debug(w);
                w.write(" to ");
                w.list(patterns, ", ", |w, p| {
                    p.ast_debug(w);
                    true
                });
                if !exclusion_patterns.is_empty() {
                    w.write(" exclude ");
                    w.list(exclusion_patterns, ", ", |w, p| {
                        p.ast_debug(w);
                        true
                    });
                }
            }
            SpecBlockMember_::Pragma { properties } => {
                w.write("pragma ");
                w.list(properties, ", ", |w, p| {
                    p.ast_debug(w);
                    true
                });
            }
        }
    }
}

impl AstDebug for (FunctionName, &Function) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Function {
                loc: _loc,
                visibility,
                signature,
                acquires,
                body,
                ..
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
            w.comma(acquires, |w, m| m.ast_debug(w));
            w.write(" ");
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

impl AstDebug for Type_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            Type_::Unit => w.write("()"),
            Type_::Multiple(ss) => {
                w.write("(");
                ss.ast_debug(w);
                w.write(")")
            }
            Type_::Apply(m, ss) => {
                m.ast_debug(w);
                if !ss.is_empty() {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
            }
            Type_::Ref(mut_, s) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                s.ast_debug(w)
            }
            Type_::Fun(args, result) => {
                w.write("(");
                w.comma(args, |w, ty| ty.ast_debug(w));
                w.write("):");
                result.ast_debug(w);
            }
            Type_::UnresolvedError => w.write("_|_"),
        }
    }
}

impl AstDebug for Vec<Type> {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.comma(self, |w, s| s.ast_debug(w))
    }
}

impl AstDebug for ModuleAccess_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&match self {
            ModuleAccess_::Name(n) => format!("{}", n),
            ModuleAccess_::ModuleAccess(m, n) => format!("{}::{}", m, n),
        })
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
            E::InferredNum(u) => w.write(&format!("{}", u)),
            E::Value(v) => v.ast_debug(w),
            E::Move(v) => w.write(&format!("move {}", v)),
            E::Copy(v) => w.write(&format!("copy {}", v)),
            E::Name(ma, tys_opt) => {
                ma.ast_debug(w);
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
            }
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
            E::Lambda(sp!(_, bs), e) => {
                w.write("fun ");
                bs.ast_debug(w);
                w.write(" ");
                e.ast_debug(w);
            }
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
            E::ExpDotted(ed) => ed.ast_debug(w),
            E::Cast(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(" as ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Index(oper, index) => {
                oper.ast_debug(w);
                w.write("[");
                index.ast_debug(w);
                w.write("]");
            }
            E::Annotate(e, ty) => {
                w.write("(");
                e.ast_debug(w);
                w.write(": ");
                ty.ast_debug(w);
                w.write(")");
            }
            E::Spec(u, unbound_names) => {
                w.write(&format!("spec #{}", u));
                if !unbound_names.is_empty() {
                    w.write("uses [");
                    w.comma(unbound_names, |w, n| w.write(&format!("{}", n)));
                    w.write("]");
                }
            }
            E::UnresolvedError => w.write("_|_"),
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

impl AstDebug for Vec<LValue> {
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

impl AstDebug for LValue_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use LValue_ as L;
        match self {
            L::Var(v, tys_opt) => {
                w.write(&format!("{}", v));
                if let Some(ss) = tys_opt {
                    w.write("<");
                    ss.ast_debug(w);
                    w.write(">");
                }
            }
            L::Unpack(ma, tys_opt, fields) => {
                ma.ast_debug(w);
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
