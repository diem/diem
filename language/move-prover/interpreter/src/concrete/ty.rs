// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The type system in move-model is a fat type system that is designed to cover all cases that can
//! possibly appear in the whole bytecode transformation pipeline. Natually, this means that some
//! types are no longer applicable when the Move program reaches the end of the transformation.
//!
//! The type system for the interpreter is a strict subset of what is offered in the move-model.
//! In other word, it is slimmed down version of the type system in move-model and a very restricted
//! set of types that are only applicable to the interpreter. Doing so enables us to write code in a
//! more precise way. For example, a type argument can only be a `BaseType` and never a reference.
//! Therefore, `BaseType` is preferred over `Type` for if a struct field/function argument holds
//! a type argument (e.g., `struct FunctionContext {ty_args: Vec<BaseType>, ...}` is preferred over
//! `ty_args: Vec<Type>`, as the former is more descriptive and less error prone).

use std::fmt;

use bytecode::stackless_bytecode::Constant;
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
};
use move_model::{
    model::{GlobalEnv, ModuleId, StructId},
    ty as MT,
};

use crate::shared::ident::StructIdent;

// avoid dependency to `move_binary_format::file_format`
pub type TypeParameterIndex = u16;
pub type CodeOffset = u16;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum IntType {
    Num,
    U8,
    U64,
    U128,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PrimitiveType {
    Bool,
    Int(IntType),
    Address,
    Signer,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StructField {
    pub name: String,
    pub ty: BaseType,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct StructInstantiation {
    pub ident: StructIdent,
    pub insts: Vec<BaseType>,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Vector(Box<BaseType>),
    Struct(StructInstantiation),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    Base(BaseType),
    Reference(bool, BaseType),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PartialBaseType {
    Primitive(PrimitiveType),
    Parameter(TypeParameterIndex),
    Vector(Box<PartialBaseType>),
    Struct(PartialStructInstantiation),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialStructField {
    pub name: String,
    pub ty: PartialBaseType,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialStructInstantiation {
    pub ident: StructIdent,
    pub insts: Vec<PartialBaseType>,
    pub fields: Vec<PartialStructField>,
}

//**************************************************************************************************
// Display
//**************************************************************************************************

impl fmt::Display for IntType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Self::Num => "num",
            Self::U8 => "u8",
            Self::U64 => "u64",
            Self::U128 => "u128",
        };
        f.write_str(repr)
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Int(sub) => sub.fmt(f),
            Self::Address => write!(f, "address"),
            Self::Signer => write!(f, "signer"),
        }
    }
}

impl fmt::Display for StructInstantiation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inst_tokens: Vec<_> = self.insts.iter().map(|t| t.to_string()).collect();
        let field_tokens: Vec<_> = self
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name, f.ty))
            .collect();
        write!(
            f,
            "struct {}<{}> {{{}}}",
            self.ident,
            inst_tokens.join(", "),
            field_tokens.join(",")
        )
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(sub) => sub.fmt(f),
            Self::Vector(sub) => write!(f, "vector<{}>", sub),
            Self::Struct(inst) => inst.fmt(f),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(sub) => sub.fmt(f),
            Self::Reference(false, sub) => write!(f, "&{}", sub),
            Self::Reference(true, sub) => write!(f, "&mut {}", sub),
        }
    }
}

impl fmt::Display for PartialBaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(sub) => sub.fmt(f),
            Self::Parameter(idx) => write!(f, "#{}", idx),
            Self::Vector(sub) => write!(f, "vector<{}>", sub),
            Self::Struct(inst) => inst.fmt(f),
        }
    }
}

impl fmt::Display for PartialStructInstantiation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inst_tokens: Vec<_> = self.insts.iter().map(|t| t.to_string()).collect();
        let field_tokens: Vec<_> = self
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name, f.ty))
            .collect();
        write!(
            f,
            "struct {}<{}> {{{}}}",
            self.ident,
            inst_tokens.join(", "),
            field_tokens.join(",")
        )
    }
}

//**************************************************************************************************
// Implementation
//**************************************************************************************************

impl StructInstantiation {
    pub fn to_move_struct_tag(&self) -> StructTag {
        let type_args = self.insts.iter().map(|e| e.to_move_type_tag()).collect();
        StructTag {
            address: self.ident.module.address,
            module: Identifier::new(self.ident.module.name.as_str()).unwrap(),
            name: Identifier::new(self.ident.name.as_str()).unwrap(),
            type_params: type_args,
        }
    }

    pub fn to_move_struct_layout(&self) -> MoveStructLayout {
        MoveStructLayout::new(
            self.fields
                .iter()
                .map(|e| e.ty.to_move_type_layout())
                .collect(),
        )
    }
}

impl BaseType {
    //
    // factory
    //

    pub fn mk_bool() -> Self {
        BaseType::Primitive(PrimitiveType::Bool)
    }

    pub fn mk_u8() -> Self {
        BaseType::Primitive(PrimitiveType::Int(IntType::U8))
    }

    pub fn mk_u64() -> Self {
        BaseType::Primitive(PrimitiveType::Int(IntType::U64))
    }

    pub fn mk_u128() -> Self {
        BaseType::Primitive(PrimitiveType::Int(IntType::U128))
    }

    pub fn mk_num() -> Self {
        BaseType::Primitive(PrimitiveType::Int(IntType::Num))
    }

    pub fn mk_address() -> Self {
        BaseType::Primitive(PrimitiveType::Address)
    }

    pub fn mk_signer() -> Self {
        BaseType::Primitive(PrimitiveType::Signer)
    }

    pub fn mk_vector(elem: BaseType) -> Self {
        BaseType::Vector(Box::new(elem))
    }

    pub fn mk_struct(inst: StructInstantiation) -> Self {
        BaseType::Struct(inst)
    }

    pub fn into_ref_type(self, is_mut: bool) -> Type {
        Type::Reference(is_mut, self)
    }

    //
    // helpers
    //

    pub fn is_bool(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Bool))
    }

    pub fn is_u8(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Int(IntType::U8)))
    }

    pub fn is_u64(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Int(IntType::U64)))
    }

    pub fn is_u128(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Int(IntType::U128)))
    }

    pub fn is_num(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Int(IntType::Num)))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Int(_)))
    }

    pub fn is_address(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Address))
    }

    pub fn is_signer(&self) -> bool {
        matches!(self, BaseType::Primitive(PrimitiveType::Signer))
    }

    pub fn is_vector(&self) -> bool {
        matches!(self, BaseType::Vector(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, BaseType::Struct(_))
    }

    pub fn get_vector_elem(&self) -> &BaseType {
        match self {
            BaseType::Vector(elem_ty) => elem_ty.as_ref(),
            _ => unreachable!(),
        }
    }

    pub fn get_struct_inst(&self) -> &StructInstantiation {
        match self {
            BaseType::Struct(inst) => inst,
            _ => unreachable!(),
        }
    }

    pub fn into_vector_elem(self) -> BaseType {
        match self {
            BaseType::Vector(elem_ty) => *elem_ty,
            _ => unreachable!(),
        }
    }

    pub fn into_struct_inst(self) -> StructInstantiation {
        match self {
            BaseType::Struct(inst) => inst,
            _ => unreachable!(),
        }
    }

    pub fn is_vector_of(&self, elem: &BaseType) -> bool {
        self.is_vector() && self.get_vector_elem() == elem
    }

    pub fn is_struct_of(&self, inst: &StructInstantiation) -> bool {
        self.is_struct() && self.get_struct_inst() == inst
    }

    //
    // checking
    //

    pub fn is_compatible_for_assign(&self, other: &BaseType) -> bool {
        match (self, other) {
            (
                BaseType::Primitive(PrimitiveType::Int(IntType::Num)),
                BaseType::Primitive(PrimitiveType::Int(_)),
            ) => true,
            (
                BaseType::Primitive(PrimitiveType::Int(_)),
                BaseType::Primitive(PrimitiveType::Int(IntType::Num)),
            ) => true,
            _ => self == other,
        }
    }

    #[allow(dead_code)]
    pub fn is_compatible_for_equality(&self, other: &BaseType) -> bool {
        self.is_compatible_for_assign(other)
    }

    pub fn is_compatible_for_arithmetic(&self, lhs: &BaseType, rhs: &BaseType) -> bool {
        self.is_int()
            && self.is_compatible_for_assign(lhs)
            && self.is_compatible_for_assign(rhs)
            && lhs.is_compatible_for_assign(rhs)
    }

    pub fn is_compatible_for_bitwise(&self, lhs: &BaseType, rhs: &BaseType) -> bool {
        match (self, lhs, rhs) {
            (BaseType::Primitive(PrimitiveType::Int(IntType::Num)), _, _) => false,
            (
                BaseType::Primitive(PrimitiveType::Int(self_ty)),
                BaseType::Primitive(PrimitiveType::Int(lhs_ty)),
                BaseType::Primitive(PrimitiveType::Int(rhs_ty)),
            ) => self_ty == lhs_ty && self_ty == rhs_ty,
            _ => false,
        }
    }

    pub fn is_compatible_for_comparison(&self, other: &BaseType) -> bool {
        self.is_int() && self.is_compatible_for_assign(other)
    }

    pub fn is_compatible_for_bitshift(&self, other: &BaseType) -> bool {
        match (self, other) {
            (BaseType::Primitive(PrimitiveType::Int(IntType::Num)), _) => false,
            (
                BaseType::Primitive(PrimitiveType::Int(self_ty)),
                BaseType::Primitive(PrimitiveType::Int(other_ty)),
            ) => self_ty == other_ty,
            _ => false,
        }
    }

    pub fn is_compatible_for_constant(&self, value: &Constant) -> bool {
        match (self, value) {
            (BaseType::Primitive(PrimitiveType::Bool), Constant::Bool(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::U8)), Constant::U8(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::U64)), Constant::U64(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::U128)), Constant::U128(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::Num)), Constant::U8(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::Num)), Constant::U64(_)) => true,
            (BaseType::Primitive(PrimitiveType::Int(IntType::Num)), Constant::U128(_)) => true,
            (BaseType::Primitive(PrimitiveType::Address), Constant::Address(_)) => true,
            (BaseType::Vector(elem_ty), Constant::ByteArray(_)) => {
                elem_ty.as_ref() == &BaseType::mk_u8()
            }
            _ => false,
        }
    }

    //
    // conversion
    //

    pub fn to_move_type_tag(&self) -> TypeTag {
        match self {
            BaseType::Primitive(PrimitiveType::Bool) => TypeTag::Bool,
            BaseType::Primitive(PrimitiveType::Int(IntType::U8)) => TypeTag::U8,
            BaseType::Primitive(PrimitiveType::Int(IntType::U64)) => TypeTag::U64,
            BaseType::Primitive(PrimitiveType::Int(IntType::U128)) => TypeTag::U128,
            BaseType::Primitive(PrimitiveType::Int(IntType::Num)) => unreachable!(),
            BaseType::Primitive(PrimitiveType::Address) => TypeTag::Address,
            BaseType::Primitive(PrimitiveType::Signer) => TypeTag::Signer,
            BaseType::Vector(elem) => TypeTag::Vector(Box::new(elem.to_move_type_tag())),
            BaseType::Struct(inst) => TypeTag::Struct(inst.to_move_struct_tag()),
        }
    }

    pub fn to_move_type_layout(&self) -> MoveTypeLayout {
        match self {
            BaseType::Primitive(PrimitiveType::Bool) => MoveTypeLayout::Bool,
            BaseType::Primitive(PrimitiveType::Int(IntType::U8)) => MoveTypeLayout::U8,
            BaseType::Primitive(PrimitiveType::Int(IntType::U64)) => MoveTypeLayout::U64,
            BaseType::Primitive(PrimitiveType::Int(IntType::U128)) => MoveTypeLayout::U128,
            BaseType::Primitive(PrimitiveType::Int(IntType::Num)) => unreachable!(),
            BaseType::Primitive(PrimitiveType::Address) => MoveTypeLayout::Address,
            BaseType::Primitive(PrimitiveType::Signer) => MoveTypeLayout::Signer,
            BaseType::Vector(elem) => MoveTypeLayout::Vector(Box::new(elem.to_move_type_layout())),
            BaseType::Struct(inst) => MoveTypeLayout::Struct(inst.to_move_struct_layout()),
        }
    }
}

macro_rules! gen {
    (
        $mk_base:ident, $mk_ref:ident,
        $is_base:ident, $is_ref:ident
    ) => {
        pub fn $mk_base() -> Self {
            Self::Base(BaseType::$mk_base())
        }
        pub fn $mk_ref(is_mut: bool) -> Self {
            Self::Reference(is_mut, BaseType::$mk_base())
        }

        pub fn $is_base(&self) -> bool {
            self.is_base() && self.get_base_type().$is_base()
        }
        pub fn $is_ref(&self, is_mut_opt: Option<bool>) -> bool {
            self.is_ref(is_mut_opt) && self.get_ref_type().1.$is_base()
        }
    };
    (
        $mk_base:ident, $mk_ref:ident,
        $is_base:ident, $is_ref:ident,
        $p:ident, $t:ty,
        $get_base_p:ident, $get_ref_p:ident,
        $into_base_p:ident, $into_ref_p:ident,
        $is_base_of:ident, $is_ref_of:ident
    ) => {
        pub fn $mk_base($p: $t) -> Self {
            Self::Base(BaseType::$mk_base($p))
        }
        pub fn $mk_ref($p: $t, is_mut: bool) -> Self {
            Self::Reference(is_mut, BaseType::$mk_base($p))
        }

        pub fn $is_base(&self) -> bool {
            self.is_base() && self.get_base_type().$is_base()
        }
        pub fn $is_ref(&self, is_mut_opt: Option<bool>) -> bool {
            self.is_ref(is_mut_opt) && self.get_ref_type().1.$is_base()
        }

        pub fn $get_base_p(&self) -> &$t {
            self.get_base_type().$get_base_p()
        }
        pub fn $get_ref_p(&self, is_mut_opt: Option<bool>) -> &$t {
            let (self_is_mut, self_ty) = self.get_ref_type();
            match is_mut_opt {
                None => (),
                Some(is_mut) => {
                    if cfg!(debug_assertions) {
                        assert_eq!(self_is_mut, is_mut);
                    }
                }
            }
            self_ty.$get_base_p()
        }

        pub fn $into_base_p(self) -> $t {
            self.into_base_type().$into_base_p()
        }
        pub fn $into_ref_p(self, is_mut_opt: Option<bool>) -> $t {
            let (self_is_mut, self_ty) = self.into_ref_type();
            match is_mut_opt {
                None => (),
                Some(is_mut) => {
                    if cfg!(debug_assertions) {
                        assert_eq!(self_is_mut, is_mut);
                    }
                }
            }
            self_ty.$into_base_p()
        }

        pub fn $is_base_of(&self, $p: &$t) -> bool {
            self.is_base() && self.get_base_type().$is_base_of($p)
        }
        pub fn $is_ref_of(&self, $p: &$t, is_mut_opt: Option<bool>) -> bool {
            self.is_ref(is_mut_opt) && self.get_ref_type().1.$is_base_of($p)
        }
    };
}

#[allow(dead_code)]
impl Type {
    //
    // factory
    //

    gen!(mk_bool, mk_ref_bool, is_bool, is_ref_bool);
    gen!(mk_u8, mk_ref_u8, is_u8, is_ref_u8);
    gen!(mk_u64, mk_ref_u64, is_u64, is_ref_u64);
    gen!(mk_u128, mk_ref_u128, is_u128, is_ref_u128);
    gen!(mk_num, mk_ref_num, is_num, is_ref_num);
    gen!(mk_address, mk_ref_address, is_address, is_ref_address);
    gen!(mk_signer, mk_ref_signer, is_signer, is_ref_signer);
    gen!(
        mk_vector,
        mk_ref_vector,
        is_vector,
        is_ref_vector,
        elem,
        BaseType,
        get_vector_elem,
        get_ref_vector_elem,
        into_vector_elem,
        into_ref_vector_elem,
        is_vector_of,
        is_ref_vector_of
    );
    gen!(
        mk_struct,
        mk_ref_struct,
        is_struct,
        is_ref_struct,
        inst,
        StructInstantiation,
        get_struct_inst,
        get_ref_struct_inst,
        into_struct_inst,
        into_ref_struct_inst,
        is_struct_of,
        is_ref_struct_of
    );

    //
    // helpers
    //

    pub fn is_base(&self) -> bool {
        matches!(self, Type::Base(_))
    }

    pub fn is_ref(&self, is_mut_opt: Option<bool>) -> bool {
        matches!(
            self,
            Type::Reference(self_is_mut, _)
            if is_mut_opt.map_or(true, |is_mut| *self_is_mut == is_mut)
        )
    }

    pub fn get_base_type(&self) -> &BaseType {
        match self {
            Type::Base(base_ty) => base_ty,
            _ => unreachable!(),
        }
    }

    pub fn get_ref_type(&self) -> (bool, &BaseType) {
        match self {
            Type::Reference(is_mut, base_ty) => (*is_mut, base_ty),
            _ => unreachable!(),
        }
    }

    pub fn into_base_type(self) -> BaseType {
        match self {
            Type::Base(base_ty) => base_ty,
            _ => unreachable!(),
        }
    }

    pub fn into_ref_type(self) -> (bool, BaseType) {
        match self {
            Type::Reference(is_mut, base_ty) => (is_mut, base_ty),
            _ => unreachable!(),
        }
    }

    pub fn is_base_of(&self, ty: &BaseType) -> bool {
        self.is_base() && self.get_base_type() == ty
    }

    pub fn is_ref_of(&self, ty: &BaseType, is_mut_opt: Option<bool>) -> bool {
        self.is_ref(is_mut_opt) && self.get_ref_type().1 == ty
    }

    pub fn is_int(&self) -> bool {
        self.is_base() && self.get_base_type().is_int()
    }

    pub fn is_ref_int(&self, is_mut_opt: Option<bool>) -> bool {
        self.is_ref(is_mut_opt) && self.get_ref_type().1.is_int()
    }

    //
    // checking
    //

    pub fn is_compatible_for_assign(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Base(base_ty_1), Type::Base(base_ty_2)) => {
                base_ty_1.is_compatible_for_assign(base_ty_2)
            }
            (Type::Reference(is_mut_1, base_ty_1), Type::Reference(is_mut_2, base_ty_2)) => {
                is_mut_1 == is_mut_2 && base_ty_1.is_compatible_for_assign(base_ty_2)
            }
            _ => false,
        }
    }

    pub fn is_compatible_for_equality(&self, other: &Type) -> bool {
        self.is_compatible_for_assign(other)
    }

    pub fn is_compatible_for_arithmetic(&self, lhs: &Type, rhs: &Type) -> bool {
        match (self, lhs, rhs) {
            (Type::Base(self_ty), Type::Base(lhs_ty), Type::Base(rhs_ty)) => {
                self_ty.is_compatible_for_arithmetic(lhs_ty, rhs_ty)
            }
            _ => false,
        }
    }

    pub fn is_compatible_for_bitwise(&self, lhs: &Type, rhs: &Type) -> bool {
        match (self, lhs, rhs) {
            (Type::Base(self_ty), Type::Base(lhs_ty), Type::Base(rhs_ty)) => {
                self_ty.is_compatible_for_bitwise(lhs_ty, rhs_ty)
            }
            _ => false,
        }
    }

    pub fn is_compatible_for_comparison(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Base(self_ty), Type::Base(other_ty)) => {
                self_ty.is_compatible_for_comparison(other_ty)
            }
            _ => false,
        }
    }

    pub fn is_compatible_for_bitshift(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Base(self_ty), Type::Base(other_ty)) => {
                self_ty.is_compatible_for_bitshift(other_ty)
            }
            _ => false,
        }
    }

    pub fn is_compatible_for_constant(&self, value: &Constant) -> bool {
        match self {
            Type::Base(base_ty) => base_ty.is_compatible_for_constant(value),
            _ => false,
        }
    }

    pub fn is_compatible_for_abort_code(&self) -> bool {
        self.is_u64() || self.is_num()
    }
}

impl PartialBaseType {
    //
    // factory
    //

    pub fn mk_bool() -> Self {
        Self::Primitive(PrimitiveType::Bool)
    }

    pub fn mk_u8() -> Self {
        Self::Primitive(PrimitiveType::Int(IntType::U8))
    }

    pub fn mk_u64() -> Self {
        Self::Primitive(PrimitiveType::Int(IntType::U64))
    }

    pub fn mk_u128() -> Self {
        Self::Primitive(PrimitiveType::Int(IntType::U128))
    }

    pub fn mk_num() -> Self {
        Self::Primitive(PrimitiveType::Int(IntType::Num))
    }

    pub fn mk_address() -> Self {
        Self::Primitive(PrimitiveType::Address)
    }

    pub fn mk_signer() -> Self {
        Self::Primitive(PrimitiveType::Signer)
    }

    pub fn mk_parameter(idx: TypeParameterIndex) -> Self {
        Self::Parameter(idx)
    }

    pub fn mk_vector(elem: Self) -> Self {
        Self::Vector(Box::new(elem))
    }

    pub fn mk_struct(inst: PartialStructInstantiation) -> Self {
        Self::Struct(inst)
    }
}

//**************************************************************************************************
// Conversion
//**************************************************************************************************

pub fn convert_model_local_type(env: &GlobalEnv, ty: &MT::Type, subst: &[BaseType]) -> Type {
    match ty {
        MT::Type::Primitive(..)
        | MT::Type::Vector(..)
        | MT::Type::Struct(..)
        | MT::Type::TypeParameter(..) => Type::Base(convert_model_base_type(env, ty, subst)),
        MT::Type::Reference(is_mut, base_ty) => {
            convert_model_base_type(env, base_ty, subst).into_ref_type(*is_mut)
        }
        _ => unreachable!(),
    }
}

pub fn convert_model_base_type(env: &GlobalEnv, ty: &MT::Type, subst: &[BaseType]) -> BaseType {
    match ty {
        MT::Type::Primitive(MT::PrimitiveType::Bool) => BaseType::mk_bool(),
        MT::Type::Primitive(MT::PrimitiveType::U8) => BaseType::mk_u8(),
        MT::Type::Primitive(MT::PrimitiveType::U64) => BaseType::mk_u64(),
        MT::Type::Primitive(MT::PrimitiveType::U128) => BaseType::mk_u128(),
        MT::Type::Primitive(MT::PrimitiveType::Num) => BaseType::mk_num(),
        MT::Type::Primitive(MT::PrimitiveType::Address) => BaseType::mk_address(),
        MT::Type::Primitive(MT::PrimitiveType::Signer) => BaseType::mk_signer(),
        MT::Type::Vector(elem) => BaseType::mk_vector(convert_model_base_type(env, elem, subst)),
        MT::Type::Struct(module_id, struct_id, ty_insts) => BaseType::mk_struct(
            convert_model_struct_type(env, *module_id, *struct_id, ty_insts, subst),
        ),
        MT::Type::TypeParameter(index) => subst.get(*index as usize).unwrap().clone(),
        _ => unreachable!(),
    }
}

pub fn convert_model_struct_type(
    env: &GlobalEnv,
    module_id: ModuleId,
    struct_id: StructId,
    ty_args: &[MT::Type],
    subst: &[BaseType],
) -> StructInstantiation {
    // derive struct identity
    let struct_env = env.get_struct(module_id.qualified(struct_id));
    let ident = StructIdent::new(&struct_env);

    // check type arguments
    if cfg!(debug_assertions) {
        assert_eq!(struct_env.get_type_parameters().len(), ty_args.len());
        // TODO (mengxu) verify type constraints
    }

    // convert instantiations
    let insts: Vec<_> = ty_args
        .iter()
        .map(|ty_arg| convert_model_base_type(env, ty_arg, subst))
        .collect();

    // collect fields
    let fields = struct_env
        .get_fields()
        .map(|field_env| {
            let field_name = env.symbol_pool().string(field_env.get_name()).to_string();
            let field_ty = convert_model_base_type(env, &field_env.get_type(), &insts);
            StructField {
                name: field_name,
                ty: field_ty,
            }
        })
        .collect();

    // return the information for constructing the struct type
    StructInstantiation {
        ident,
        insts,
        fields,
    }
}

pub fn convert_model_partial_base_type(env: &GlobalEnv, ty: &MT::Type) -> PartialBaseType {
    match ty {
        MT::Type::Primitive(MT::PrimitiveType::Bool) => PartialBaseType::mk_bool(),
        MT::Type::Primitive(MT::PrimitiveType::U8) => PartialBaseType::mk_u8(),
        MT::Type::Primitive(MT::PrimitiveType::U64) => PartialBaseType::mk_u64(),
        MT::Type::Primitive(MT::PrimitiveType::U128) => PartialBaseType::mk_u128(),
        MT::Type::Primitive(MT::PrimitiveType::Num) => PartialBaseType::mk_num(),
        MT::Type::Primitive(MT::PrimitiveType::Address) => PartialBaseType::mk_address(),
        MT::Type::Primitive(MT::PrimitiveType::Signer) => PartialBaseType::mk_signer(),
        MT::Type::Vector(elem) => {
            PartialBaseType::mk_vector(convert_model_partial_base_type(env, elem))
        }
        MT::Type::Struct(module_id, struct_id, ty_insts) => PartialBaseType::mk_struct(
            convert_model_partial_struct_type(env, *module_id, *struct_id, ty_insts),
        ),
        MT::Type::TypeParameter(index) => PartialBaseType::mk_parameter(*index),
        _ => unreachable!(),
    }
}

pub fn convert_model_partial_struct_type(
    env: &GlobalEnv,
    module_id: ModuleId,
    struct_id: StructId,
    ty_args: &[MT::Type],
) -> PartialStructInstantiation {
    // derive struct identity
    let struct_env = env.get_struct(module_id.qualified(struct_id));
    let ident = StructIdent::new(&struct_env);

    // check type arguments
    if cfg!(debug_assertions) {
        assert_eq!(struct_env.get_type_parameters().len(), ty_args.len());
        // TODO (mengxu) verify type constraints
    }

    // convert instantiations
    let insts: Vec<_> = ty_args
        .iter()
        .map(|ty_arg| convert_model_partial_base_type(env, ty_arg))
        .collect();

    // collect fields
    let fields = struct_env
        .get_fields()
        .map(|field_env| {
            let field_name = env.symbol_pool().string(field_env.get_name()).to_string();
            let field_ty = convert_model_partial_base_type(env, &field_env.get_type());
            PartialStructField {
                name: field_name,
                ty: field_ty,
            }
        })
        .collect();

    // return the information for constructing the struct type
    PartialStructInstantiation {
        ident,
        insts,
        fields,
    }
}
