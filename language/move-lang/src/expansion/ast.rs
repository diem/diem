// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parser::ast::{
        Ability, Ability_, BinOp, ConstantName, Field, FunctionName, FunctionVisibility,
        ModuleIdent, QuantKind, SpecApplyPattern, SpecBlockTarget, SpecConditionKind, StructName,
        UnaryOp, Var,
    },
    shared::{ast_debug::*, unique_map::UniqueMap, unique_set::UniqueSet, *},
};
use move_ir_types::location::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt,
};

//**************************************************************************************************
// Program
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Program {
    pub modules: UniqueMap<ModuleIdent, ModuleDefinition>,
    pub scripts: BTreeMap<String, Script>,
}

//**************************************************************************************************
// Scripts
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct Script {
    pub loc: Loc,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub function_name: FunctionName,
    pub function: Function,
    pub specs: Vec<SpecBlock>,
}

//**************************************************************************************************
// Modules
//**************************************************************************************************

#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    pub loc: Loc,
    pub is_source_module: bool,
    /// `dependency_order` is the topological order/rank in the dependency graph.
    /// `dependency_order` is initialized at `0` and set in the uses pass
    pub dependency_order: usize,
    pub friends: UniqueMap<ModuleIdent, Loc>,
    pub structs: UniqueMap<StructName, StructDefinition>,
    pub functions: UniqueMap<FunctionName, Function>,
    pub constants: UniqueMap<ConstantName, Constant>,
    pub specs: Vec<SpecBlock>,
}

//**************************************************************************************************
// Structs
//**************************************************************************************************

pub type Fields<T> = UniqueMap<Field, (usize, T)>;

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub loc: Loc,
    pub abilities: AbilitySet,
    pub type_parameters: Vec<(Name, AbilitySet)>,
    pub fields: StructFields,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StructFields {
    Defined(Fields<Type>),
    Native(Loc),
}

//**************************************************************************************************
// Functions
//**************************************************************************************************

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionSignature {
    pub type_parameters: Vec<(Name, AbilitySet)>,
    pub parameters: Vec<(Var, Type)>,
    pub return_type: Type,
}

#[derive(PartialEq, Clone, Debug)]
pub enum FunctionBody_ {
    Defined(Sequence),
    Native,
}
pub type FunctionBody = Spanned<FunctionBody_>;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SpecId(usize);

#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub loc: Loc,
    pub visibility: FunctionVisibility,
    pub signature: FunctionSignature,
    pub acquires: Vec<ModuleAccess>,
    pub body: FunctionBody,
    pub specs: BTreeMap<SpecId, SpecBlock>,
}

//**************************************************************************************************
// Constants
//**************************************************************************************************

#[derive(PartialEq, Clone, Debug)]
pub struct Constant {
    pub loc: Loc,
    pub signature: Type,
    pub value: Exp,
}

//**************************************************************************************************
// Specification Blocks
//**************************************************************************************************

#[derive(Debug, Clone, PartialEq)]
pub struct SpecBlock_ {
    pub target: SpecBlockTarget,
    pub members: Vec<SpecBlockMember>,
}
pub type SpecBlock = Spanned<SpecBlock_>;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum SpecBlockMember_ {
    Condition {
        kind: SpecConditionKind,
        properties: Vec<PragmaProperty>,
        exp: Exp,
        additional_exps: Vec<Exp>,
    },
    Function {
        uninterpreted: bool,
        name: FunctionName,
        signature: FunctionSignature,
        body: FunctionBody,
    },
    Variable {
        is_global: bool,
        name: Name,
        type_parameters: Vec<(Name, AbilitySet)>,
        type_: Type,
    },
    Let {
        name: Name,
        def: Exp,
    },
    Include {
        properties: Vec<PragmaProperty>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct PragmaProperty_ {
    pub name: Name,
    pub value: Option<PragmaValue>,
}
pub type PragmaProperty = Spanned<PragmaProperty_>;

#[derive(Debug, Clone, PartialEq)]
pub enum PragmaValue {
    Literal(Value),
    Ident(ModuleAccess),
}

//**************************************************************************************************
// Types
//**************************************************************************************************

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AbilitySet(UniqueSet<Ability>);

#[derive(Debug, PartialEq, Clone)]
pub enum ModuleAccess_ {
    Name(Name),
    ModuleAccess(ModuleIdent, Name),
}
pub type ModuleAccess = Spanned<ModuleAccess_>;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum LValue_ {
    Var(ModuleAccess, Option<Vec<Type>>),
    Unpack(ModuleAccess, Option<Vec<Type>>, Fields<LValue>),
}
pub type LValue = Spanned<LValue_>;
pub type LValueList_ = Vec<LValue>;
pub type LValueList = Spanned<LValueList_>;

pub type LValueWithRange_ = (LValue, Exp);
pub type LValueWithRange = Spanned<LValueWithRange_>;
pub type LValueWithRangeList_ = Vec<LValueWithRange>;
pub type LValueWithRangeList = Spanned<LValueWithRangeList_>;

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ExpDotted_ {
    Exp(Exp),
    Dot(Box<ExpDotted>, Name),
}
pub type ExpDotted = Spanned<ExpDotted_>;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Exp_ {
    Value(Value),
    InferredNum(u128),
    Move(Var),
    Copy(Var),

    Name(ModuleAccess, Option<Vec<Type>>),
    Call(ModuleAccess, Option<Vec<Type>>, Spanned<Vec<Exp>>),
    Pack(ModuleAccess, Option<Vec<Type>>, Fields<Exp>),

    IfElse(Box<Exp>, Box<Exp>, Box<Exp>),
    While(Box<Exp>, Box<Exp>),
    Loop(Box<Exp>),
    Block(Sequence),
    Lambda(LValueList, Box<Exp>), // spec only
    Quant(
        QuantKind,
        LValueWithRangeList,
        Vec<Vec<Exp>>,
        Option<Box<Exp>>,
        Box<Exp>,
    ), // spec only

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
    Unit {
        trailing: bool,
    },

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
#[derive(Debug, Clone, PartialEq)]
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

impl AbilitySet {
    /// All abilities
    pub const ALL: [Ability_; 4] = [
        Ability_::Copy,
        Ability_::Drop,
        Ability_::Store,
        Ability_::Key,
    ];
    /// Abilities for bool, u8, u64, u128, and address
    pub const PRIMITIVES: [Ability_; 3] = [Ability_::Copy, Ability_::Drop, Ability_::Store];
    /// Abilities for &_ and &mut _
    pub const REFERENCES: [Ability_; 2] = [Ability_::Copy, Ability_::Drop];
    /// Abilities for signer
    pub const SIGNER: [Ability_; 1] = [Ability_::Drop];
    /// Abilities for vector<_>, note they are predicated on the type argument
    pub const COLLECTION: [Ability_; 3] = [Ability_::Copy, Ability_::Drop, Ability_::Store];

    pub fn empty() -> Self {
        AbilitySet(UniqueSet::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn add(&mut self, a: Ability) -> Result<(), Loc> {
        self.0.add(a).map_err(|(_a, loc)| loc)
    }

    pub fn has_ability(&self, a: &Ability) -> bool {
        self.0.contains(&a)
    }

    pub fn has_ability_(&self, a: Ability_) -> bool {
        self.0.contains_(&a)
    }

    // intersection of two sets. Keeps the loc of the first set
    pub fn intersect(&self, other: &Self) -> Self {
        Self(self.0.intersect(&other.0))
    }

    // union of two sets. Prefers the loc of the first set
    pub fn union(&self, other: &Self) -> Self {
        Self(self.0.union(&other.0))
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_subset(&other.0)
    }

    pub fn iter(&self) -> AbilitySetIter {
        self.into_iter()
    }

    pub fn from_abilities(
        iter: impl IntoIterator<Item = Ability>,
    ) -> Result<Self, (Ability_, Loc, Loc)> {
        Ok(Self(UniqueSet::from_elements(iter)?))
    }

    pub fn from_abilities_(
        loc: Loc,
        iter: impl IntoIterator<Item = Ability_>,
    ) -> Result<Self, (Ability_, Loc, Loc)> {
        Ok(Self(UniqueSet::from_elements_(loc, iter)?))
    }

    pub fn all(loc: Loc) -> Self {
        Self::from_abilities_(loc, Self::ALL.to_vec()).unwrap()
    }

    pub fn primitives(loc: Loc) -> Self {
        Self::from_abilities_(loc, Self::PRIMITIVES.to_vec()).unwrap()
    }

    pub fn references(loc: Loc) -> Self {
        Self::from_abilities_(loc, Self::REFERENCES.to_vec()).unwrap()
    }

    pub fn signer(loc: Loc) -> Self {
        Self::from_abilities_(loc, Self::SIGNER.to_vec()).unwrap()
    }

    pub fn collection(loc: Loc) -> Self {
        Self::from_abilities_(loc, Self::COLLECTION.to_vec()).unwrap()
    }
}

//**************************************************************************************************
// Iter
//**************************************************************************************************

pub struct AbilitySetIter<'a>(unique_set::Iter<'a, Ability>);

impl<'a> Iterator for AbilitySetIter<'a> {
    type Item = Ability;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(loc, a_)| sp(loc, *a_))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> IntoIterator for &'a AbilitySet {
    type Item = Ability;
    type IntoIter = AbilitySetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AbilitySetIter(self.0.iter())
    }
}

pub struct AbilitySetIntoIter(unique_set::IntoIter<Ability>);

impl Iterator for AbilitySetIntoIter {
    type Item = Ability;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> IntoIterator for AbilitySet {
    type Item = Ability;
    type IntoIter = AbilitySetIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        AbilitySetIntoIter(self.0.into_iter())
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
        for (m, mdef) in modules.key_cloned_iter() {
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
            constants,
            function_name,
            function,
            specs,
        } = self;
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
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
            dependency_order,
            friends,
            structs,
            functions,
            constants,
            specs,
        } = self;
        w.writeln(if *is_source_module {
            "source module"
        } else {
            "library module"
        });
        w.writeln(&format!("dependency order #{}", dependency_order));
        for (mident, _loc) in friends.key_cloned_iter() {
            w.write(&format!("friend {};", mident));
            w.new_line();
        }
        for sdef in structs.key_cloned_iter() {
            sdef.ast_debug(w);
            w.new_line();
        }
        for cdef in constants.key_cloned_iter() {
            cdef.ast_debug(w);
            w.new_line();
        }
        for fdef in functions.key_cloned_iter() {
            fdef.ast_debug(w);
            w.new_line();
        }
        for spec in specs {
            spec.ast_debug(w);
            w.new_line();
        }
    }
}

pub fn ability_modifiers_ast_debug(w: &mut AstWriter, abilities: &AbilitySet) {
    if !abilities.is_empty() {
        w.write(" has ");
        w.list(abilities, " ", |w, ab| {
            ab.ast_debug(w);
            false
        });
    }
}

impl AstDebug for (StructName, &StructDefinition) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            StructDefinition {
                loc: _loc,
                abilities,
                type_parameters,
                fields,
            },
        ) = self;

        if let StructFields::Native(_) = fields {
            w.write("native ");
        }

        w.write(&format!("struct {}", name));
        type_parameters.ast_debug(w);
        ability_modifiers_ast_debug(w, abilities);
        if let StructFields::Defined(fields) = fields {
            w.block(|w| {
                w.list(fields, ",", |w, (_, f, idx_st)| {
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
            SpecBlockMember_::Condition {
                kind,
                properties: _,
                exp,
                additional_exps,
            } => {
                kind.ast_debug(w);
                exp.ast_debug(w);
                w.list(additional_exps, ",", |w, e| {
                    e.ast_debug(w);
                    true
                });
            }
            SpecBlockMember_::Function {
                uninterpreted,
                signature,
                name,
                body,
            } => {
                if *uninterpreted {
                    w.write("uninterpreted ")
                } else if let FunctionBody_::Native = &body.value {
                    w.write("native ");
                }
                w.write(&format!("define {}", name));
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
            SpecBlockMember_::Let { name, def } => {
                w.write(&format!("let {} = ", name));
                def.ast_debug(w);
            }
            SpecBlockMember_::Include { properties: _, exp } => {
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

impl AstDebug for PragmaProperty_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        w.write(&self.name.value);
        if let Some(value) = &self.value {
            w.write(" = ");
            match value {
                PragmaValue::Literal(l) => l.ast_debug(w),
                PragmaValue::Ident(i) => i.ast_debug(w),
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
        w.write(&format!("fun {}", name));
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

impl AstDebug for (ConstantName, &Constant) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (
            name,
            Constant {
                loc: _loc,
                signature,
                value,
            },
        ) = self;
        w.write(&format!("const {}:", name));
        signature.ast_debug(w);
        w.write(" = ");
        value.ast_debug(w);
        w.write(";");
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

impl AstDebug for Vec<(Name, AbilitySet)> {
    fn ast_debug(&self, w: &mut AstWriter) {
        if !self.is_empty() {
            w.write("<");
            w.comma(self, |w, tp| tp.ast_debug(w));
            w.write(">")
        }
    }
}

pub fn ability_constraints_ast_debug(w: &mut AstWriter, abilities: &AbilitySet) {
    if !abilities.is_empty() {
        w.write(": ");
        w.list(abilities, "+", |w, ab| {
            ab.ast_debug(w);
            false
        })
    }
}

impl AstDebug for (Name, AbilitySet) {
    fn ast_debug(&self, w: &mut AstWriter) {
        let (n, abilities) = self;
        w.write(&n.value);
        ability_constraints_ast_debug(w, abilities)
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

impl AstDebug for Exp_ {
    fn ast_debug(&self, w: &mut AstWriter) {
        use Exp_ as E;
        match self {
            E::Unit { trailing } if !trailing => w.write("()"),
            E::Unit {
                trailing: _trailing,
            } => w.write("/*()*/"),
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
                w.comma(fields, |w, (_, f, idx_e)| {
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
            E::Quant(kind, sp!(_, rs), trs, c_opt, e) => {
                kind.ast_debug(w);
                w.write(" ");
                rs.ast_debug(w);
                trs.ast_debug(w);
                if let Some(c) = c_opt {
                    w.write(" where ");
                    c.ast_debug(w);
                }
                w.write(" : ");
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
                    w.write(" uses [");
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
                w.comma(fields, |w, (_, f, idx_b)| {
                    let (idx, b) = idx_b;
                    w.write(&format!("{}#{}: ", idx, f));
                    b.ast_debug(w);
                });
                w.write("}");
            }
        }
    }
}

impl AstDebug for Vec<LValueWithRange> {
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

impl AstDebug for (LValue, Exp) {
    fn ast_debug(&self, w: &mut AstWriter) {
        self.0.ast_debug(w);
        w.write(" in ");
        self.1.ast_debug(w);
    }
}

impl AstDebug for Vec<Vec<Exp>> {
    fn ast_debug(&self, w: &mut AstWriter) {
        for trigger in self {
            w.write("{");
            w.comma(trigger, |w, b| b.ast_debug(w));
            w.write("}");
        }
    }
}
