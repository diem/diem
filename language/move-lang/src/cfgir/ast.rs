// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    naming::ast::{BuiltinTypeName, BuiltinTypeName_, TParam, TypeName, TypeName_},
    parser::ast::{
        BinOp, Field, FunctionName, FunctionVisibility, Kind, Kind_, ModuleIdent, ResourceLoc,
        StructName, UnaryOp, Value, Var,
    },
    shared::{ast_debug::*, unique_map::UniqueMap, *},
};
use move_ir_types::location::*;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// HLIR + Unstructured Control Flow + CFG

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
    /// `None` if it is a library dependency
    /// `Some(order)` if it is a source file. Where `order` is the topological order/rank in the
    /// depedency graph
    pub is_source_module: Option<usize>,
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
    Defined(Vec<(Field, BaseType)>),
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
    Native,
    Defined {
        locals: UniqueMap<Var, SingleType>,
        start: Label,
        blocks: Blocks,
    },
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum BaseType_ {
    // TODO this can be removed after the type system rework
    UnresolvedError,
    Param(TParam),
    Apply(Kind, TypeName, Vec<BaseType>),
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
// Blocks
//**************************************************************************************************

pub type Blocks = BTreeMap<Label, BasicBlock>;

pub type BasicBlock = VecDeque<Command>;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub struct Label(pub usize);

//**************************************************************************************************
// Commands
//**************************************************************************************************

#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Command_ {
    Assign(Vec<LValue>, Box<Exp>),
    Mutate(Box<Exp>, Box<Exp>),
    Abort(Exp),
    Return(Exp),
    IgnoreAndPop {
        pop_num: usize,
        exp: Exp,
    },
    Jump(Label),
    JumpIf {
        cond: Exp,
        if_true: Label,
        if_false: Label,
    },
}
pub type Command = Spanned<Command_>;

#[derive(Debug, PartialEq)]
pub enum LValue_ {
    Ignore,
    Var(Var, Box<SingleType>),
    Unpack(StructName, Vec<BaseType>, Vec<(Field, LValue)>),
}
pub type LValue = Spanned<LValue_>;

//**************************************************************************************************
// Expressions
//**************************************************************************************************

#[derive(Debug, PartialEq)]
pub struct ModuleCall {
    pub module: ModuleIdent,
    pub name: FunctionName,
    pub type_arguments: Vec<BaseType>,
    pub arguments: Box<Exp>,
    pub acquires: BTreeSet<StructName>,
}

#[derive(Debug, PartialEq)]
pub enum BuiltinFunction_ {
    MoveToSender(BaseType),
    MoveFrom(BaseType),
    BorrowGlobal(bool, BaseType),
    Exists(BaseType),
}
pub type BuiltinFunction = Spanned<BuiltinFunction_>;

#[derive(Debug, PartialEq)]
pub enum UnannotatedExp_ {
    Unit,
    Value(Value),
    Move { from_user: bool, var: Var },
    Copy { from_user: bool, var: Var },

    ModuleCall(Box<ModuleCall>),

    Builtin(Box<BuiltinFunction>, Box<Exp>),
    Freeze(Box<Exp>),

    Dereference(Box<Exp>),
    UnaryExp(UnaryOp, Box<Exp>),
    BinopExp(Box<Exp>, BinOp, Box<Exp>),

    Pack(StructName, Vec<BaseType>, Vec<(Field, BaseType, Exp)>),
    ExpList(Vec<ExpListItem>),

    Borrow(bool, Box<Exp>, Field),
    BorrowLocal(bool, Var),

    Cast(Box<Exp>, BuiltinTypeName),

    Unreachable,

    UnresolvedError,
}
pub type UnannotatedExp = Spanned<UnannotatedExp_>;
#[derive(Debug, PartialEq)]
pub struct Exp {
    pub ty: Type,
    pub exp: UnannotatedExp,
}
pub fn exp(ty: Type, exp: UnannotatedExp) -> Exp {
    Exp { ty, exp }
}

#[derive(Debug, PartialEq)]
pub enum ExpListItem {
    Single(Exp, Box<SingleType>),
    Splat(Loc, Exp, Vec<SingleType>),
}

//**************************************************************************************************
// impls
//**************************************************************************************************

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Command_ {
    pub fn is_terminal(&self) -> bool {
        use Command_::*;
        match self {
            Assign(_, _) | Mutate(_, _) | IgnoreAndPop { .. } => false,
            Abort(_) | Return(_) | Jump(_) | JumpIf { .. } => true,
        }
    }

    pub fn is_exit(&self) -> bool {
        use Command_::*;
        match self {
            Assign(_, _) | Mutate(_, _) | IgnoreAndPop { .. } | Jump(_) | JumpIf { .. } => false,
            Abort(_) | Return(_) => true,
        }
    }

    pub fn is_unit(&self) -> bool {
        use Command_ as C;
        match self {
            C::Assign(ls, e) => ls.is_empty() && e.is_unit(),
            C::IgnoreAndPop { exp: e, .. } => e.is_unit(),

            C::Mutate(_, _) | C::Return(_) | C::Abort(_) | C::JumpIf { .. } | C::Jump(_) => false,
        }
    }

    pub fn successors(&self) -> BTreeSet<Label> {
        use Command_::*;

        let mut successors = BTreeSet::new();
        match self {
            Mutate(_, _) | Assign(_, _) | IgnoreAndPop { .. } => {
                panic!("ICE Should not be last command in block")
            }
            Abort(_) | Return(_) => (),
            Jump(lbl) => {
                successors.insert(lbl.clone());
            }
            JumpIf {
                if_true, if_false, ..
            } => {
                successors.insert(if_true.clone());
                successors.insert(if_false.clone());
            }
        }
        successors
    }
}

impl Exp {
    pub fn is_unit(&self) -> bool {
        self.exp.value.is_unit()
    }
}

impl UnannotatedExp_ {
    pub fn is_unit(&self) -> bool {
        match self {
            UnannotatedExp_::Unit => true,
            _ => false,
        }
    }
}

impl BaseType_ {
    pub fn builtin(loc: Loc, b_: BuiltinTypeName_, ty_args: Vec<BaseType>) -> BaseType {
        use BuiltinTypeName_::*;

        let kind = match b_ {
            U8 | U64 | U128 | Bool | Address | Bytearray => sp(loc, Kind_::Unrestricted),
            Vector => {
                assert!(
                    ty_args.len() == 1,
                    "ICE vector should have exactly 1 type argument."
                );
                ty_args[0].value.kind()
            }
        };
        let n = sp(loc, TypeName_::Builtin(sp(loc, b_)));
        sp(loc, BaseType_::Apply(kind, n, ty_args))
    }

    pub fn bool(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Bool, vec![])
    }

    pub fn address(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::Address, vec![])
    }

    pub fn u64(loc: Loc) -> BaseType {
        Self::builtin(loc, BuiltinTypeName_::U64, vec![])
    }

    pub fn kind(&self) -> Kind {
        match self {
            BaseType_::Apply(k, _, _) => k.clone(),
            BaseType_::Param(TParam { kind, .. }) => kind.clone(),
            BaseType_::UnresolvedError => panic!(
                "ICE unresolved error has no kind. \
                 Should only exist in dead code that should not be analyzed"
            ),
        }
    }
}

impl SingleType_ {
    pub fn base(sp!(loc, b_): BaseType) -> SingleType {
        sp(loc, SingleType_::Base(sp(loc, b_)))
    }

    pub fn bool(loc: Loc) -> SingleType {
        Self::base(BaseType_::bool(loc))
    }

    pub fn address(loc: Loc) -> SingleType {
        Self::base(BaseType_::address(loc))
    }

    pub fn u64(loc: Loc) -> SingleType {
        Self::base(BaseType_::u64(loc))
    }

    pub fn kind(&self, loc: Loc) -> Kind {
        match self {
            SingleType_::Ref(_, _) => sp(loc, Kind_::Unrestricted),
            SingleType_::Base(b) => b.value.kind(),
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

    pub fn bool(loc: Loc) -> Type {
        Self::single(SingleType_::bool(loc))
    }

    pub fn address(loc: Loc) -> Type {
        Self::single(SingleType_::address(loc))
    }

    pub fn u64(loc: Loc) -> Type {
        Self::single(SingleType_::u64(loc))
    }

    pub fn type_at_index(&self, idx: usize) -> &SingleType {
        match self {
            Type_::Unit => panic!("ICE type mismatch on index lookup"),
            Type_::Single(s) => {
                assert!(idx == 0);
                s
            }
            Type_::Multiple(ss) => {
                assert!(idx < ss.len());
                ss.get(idx).unwrap()
            }
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
            structs,
            functions,
        } = self;
        match is_source_module {
            None => w.writeln("library module"),
            Some(ord) => w.writeln(&format!("source moduel #{}", ord)),
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
                w.list(fields, ",", |w, (f, bt)| {
                    w.write(&format!("{}: ", f));
                    bt.ast_debug(w);
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
            w.write(" ");
        }
        match &body.value {
            FunctionBody_::Defined {
                locals,
                start,
                blocks,
            } => w.block(|w| {
                w.write("locals:");
                w.indent(4, |w| {
                    w.list(locals, ",", |w, (v, st)| {
                        w.write(&format!("{}: ", v));
                        st.ast_debug(w);
                        true
                    })
                });
                w.new_line();
                w.writeln(&format!("start={}", start.0));
                w.new_line();
                blocks.ast_debug(w);
            }),
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
            st.ast_debug(w)
        });
        w.write("): ");
        return_type.ast_debug(w)
    }
}

impl AstDebug for BaseType_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            BaseType_::UnresolvedError => w.write("_|_"),
            BaseType_::Param(tp) => tp.ast_debug(w),
            BaseType_::Apply(k, m, ss) => {
                w.annotate(
                    |w| {
                        m.ast_debug(w);
                        if !ss.is_empty() {
                            w.write("<");
                            ss.ast_debug(w);
                            w.write(">");
                        }
                    },
                    k,
                );
            }
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

impl AstDebug for Blocks {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.list(self, "", |w, lbl_block| {
            lbl_block.ast_debug(w);
            w.new_line();
            true
        })
    }
}

impl AstDebug for (&Label, &BasicBlock) {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&format!("label {}:", (self.0).0));
        w.indent(4, |w| w.semicolon(self.1, |w, cmd| cmd.ast_debug(w)))
    }
}

impl AstDebug for Command_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Command_ as C;
        match self {
            C::Assign(lvalues, rhs) => {
                lvalues.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }
            C::Mutate(lhs, rhs) => {
                w.write("*");
                lhs.ast_debug(w);
                w.write(" = ");
                rhs.ast_debug(w);
            }
            C::Abort(e) => {
                w.write("abort ");
                e.ast_debug(w);
            }
            C::Return(e) => {
                w.write("return ");
                e.ast_debug(w);
            }
            C::IgnoreAndPop { pop_num, exp } => {
                w.write("pop ");
                w.comma(0..*pop_num, |w, _| w.write("_"));
                w.write(" = ");
                exp.ast_debug(w);
            }
            C::Jump(lbl) => w.write(&format!("jump {}", lbl.0)),
            C::JumpIf {
                cond,
                if_true,
                if_false,
            } => {
                w.write("jump_if(");
                cond.ast_debug(w);
                w.write(&format!(") {} else {}", if_true.0, if_false.0));
            }
        }
    }
}

impl AstDebug for Exp {
    fn ast_debug(&self, w: &mut AstWriter) {
        let Exp { ty, exp } = self;
        w.annotate(|w| exp.ast_debug(w), ty)
    }
}

impl AstDebug for UnannotatedExp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use UnannotatedExp_ as E;
        match self {
            E::Unit => w.write("()"),
            E::Value(v) => v.ast_debug(w),
            E::Move {
                from_user: false,
                var: v,
            } => w.write(&format!("move {}", v)),
            E::Move {
                from_user: true,
                var: v,
            } => w.write(&format!("move@{}", v)),
            E::Copy {
                from_user: false,
                var: v,
            } => w.write(&format!("copy {}", v)),
            E::Copy {
                from_user: true,
                var: v,
            } => w.write(&format!("copy@{}", v)),
            E::ModuleCall(mcall) => {
                mcall.ast_debug(w);
            }
            E::Builtin(bf, rhs) => {
                bf.ast_debug(w);
                w.write("(");
                rhs.ast_debug(w);
                w.write(")");
            }
            E::Freeze(e) => {
                w.write("freeze(");
                e.ast_debug(w);
                w.write(")");
            }
            E::Pack(s, tys, fields) => {
                w.write(&format!("{}", s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, bt, e)| {
                    w.write(&format!("{}:", f));
                    bt.ast_debug(w);
                    w.write(": ");
                    e.ast_debug(w);
                });
                w.write("}");
            }

            E::ExpList(es) => {
                w.write("(");
                w.comma(es, |w, e| e.ast_debug(w));
                w.write(")");
            }

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
            E::Borrow(mut_, e, f) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                e.ast_debug(w);
                w.write(&format!(".{}", f));
            }
            E::BorrowLocal(mut_, v) => {
                w.write("&");
                if *mut_ {
                    w.write("mut ");
                }
                w.write(&format!("{}", v));
            }
            E::Cast(e, bt) => {
                w.write("(");
                e.ast_debug(w);
                w.write(" as ");
                bt.ast_debug(w);
                w.write(")");
            }
            E::UnresolvedError => w.write("_|_"),
            E::Unreachable => w.write("unreachable"),
        }
    }
}

impl AstDebug for ModuleCall {
    fn ast_debug(&self, w: &mut AstWriter) {
        let ModuleCall {
            module,
            name,
            type_arguments,
            acquires,
            arguments,
        } = self;
        w.write(&format!("{}::{}", module, name));
        if !acquires.is_empty() {
            w.write("[acquires: [");
            w.comma(acquires, |w, s| w.write(&format!("{}", s)));
            w.write("]], ");
        }
        w.write("<");
        type_arguments.ast_debug(w);
        w.write(">");
        w.write("(");
        arguments.ast_debug(w);
        w.write(")");
    }
}

impl AstDebug for BuiltinFunction_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use crate::naming::ast::BuiltinFunction_ as NF;
        use BuiltinFunction_ as F;
        let (n, bt) = match self {
            F::MoveToSender(bt) => (NF::MOVE_TO_SENDER, bt),
            F::MoveFrom(bt) => (NF::MOVE_FROM, bt),
            F::BorrowGlobal(true, bt) => (NF::BORROW_GLOBAL_MUT, bt),
            F::BorrowGlobal(false, bt) => (NF::BORROW_GLOBAL, bt),
            F::Exists(bt) => (NF::EXISTS, bt),
        };
        w.write(n);
        w.write("<");
        bt.ast_debug(w);
        w.write(">");
    }
}

impl AstDebug for ExpListItem {
    fn ast_debug(&self, w: &mut AstWriter) {
        match self {
            ExpListItem::Single(e, st) => w.annotate(|w| e.ast_debug(w), st),
            ExpListItem::Splat(_, e, ss) => {
                w.write("~");
                w.annotate(|w| e.ast_debug(w), ss)
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
        w.comma(self, |w, a| a.ast_debug(w));
        if parens {
            w.write(")");
        }
    }
}

impl AstDebug for LValue_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use LValue_ as L;
        match self {
            L::Ignore => w.write("_"),
            L::Var(v, st) => w.annotate(|w| w.write(&format!("{}", v)), st),

            L::Unpack(s, tys, fields) => {
                w.write(&format!("{}", s));
                w.write("<");
                tys.ast_debug(w);
                w.write(">");
                w.write("{");
                w.comma(fields, |w, (f, l)| {
                    w.write(&format!("{}: ", f));
                    l.ast_debug(w)
                });
                w.write("}");
            }
        }
    }
}
