// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::fmt;

use bytecode::stackless_bytecode::Constant;
use move_binary_format::file_format::TypeParameterIndex;

use crate::shared::ident::StructIdent;

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
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Vector(Box<BaseType>),
    Struct(StructInstantiation),
    Parameter(TypeParameterIndex),
    Binding(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    Base(BaseType),
    Reference(bool, BaseType),
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
        let tokens: Vec<_> = self
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name, f.ty))
            .collect();
        write!(f, "struct {} {{{}}}", self.ident, tokens.join(","))
    }
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(sub) => sub.fmt(f),
            Self::Vector(sub) => write!(f, "vector<{}>", sub),
            Self::Struct(inst) => inst.fmt(f),
            Self::Parameter(index) => write!(f, "#{}", index),
            Self::Binding(symbol) => write!(f, "{{{}}}", symbol),
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

//**************************************************************************************************
// Implementation
//**************************************************************************************************

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

    pub fn mk_vector(elem_ty: BaseType) -> Self {
        BaseType::Vector(Box::new(elem_ty))
    }

    pub fn mk_struct(inst: StructInstantiation) -> Self {
        BaseType::Struct(inst)
    }

    pub fn mk_parameter(index: TypeParameterIndex) -> Self {
        BaseType::Parameter(index)
    }

    pub fn mk_binding(symbol: String) -> Self {
        BaseType::Binding(symbol)
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

    pub fn is_parameter(&self) -> bool {
        matches!(self, BaseType::Parameter(_))
    }

    pub fn is_binding(&self) -> bool {
        matches!(self, BaseType::Binding(_))
    }

    pub fn get_vector_element_type(&self) -> Option<&BaseType> {
        match self {
            BaseType::Vector(elem_ty) => Some(elem_ty.as_ref()),
            _ => None,
        }
    }

    pub fn get_struct_instantiation(&self) -> Option<&StructInstantiation> {
        match self {
            BaseType::Struct(inst) => Some(inst),
            _ => None,
        }
    }

    pub fn get_parameter_index(&self) -> Option<TypeParameterIndex> {
        match self {
            BaseType::Parameter(index) => Some(*index),
            _ => None,
        }
    }

    pub fn get_binding_symbol(&self) -> Option<&str> {
        match self {
            BaseType::Binding(symbol) => Some(symbol),
            _ => None,
        }
    }

    pub fn is_vector_of(&self, elem_ty: &BaseType) -> bool {
        self.get_vector_element_type()
            .map_or(false, |self_ty| self_ty == elem_ty)
    }

    pub fn is_struct_of(&self, inst: &StructInstantiation) -> bool {
        self.get_struct_instantiation()
            .map_or(false, |self_inst| self_inst == inst)
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
}

#[allow(dead_code)]
impl Type {
    //
    // factory
    //

    pub fn mk_bool() -> Self {
        Type::Base(BaseType::mk_bool())
    }

    pub fn mk_u8() -> Self {
        Type::Base(BaseType::mk_u8())
    }

    pub fn mk_u64() -> Self {
        Type::Base(BaseType::mk_u64())
    }

    pub fn mk_u128() -> Self {
        Type::Base(BaseType::mk_u128())
    }

    pub fn mk_num() -> Self {
        Type::Base(BaseType::mk_num())
    }

    pub fn mk_address() -> Self {
        Type::Base(BaseType::mk_address())
    }

    pub fn mk_signer() -> Self {
        Type::Base(BaseType::mk_signer())
    }

    pub fn mk_vector(elem_ty: BaseType) -> Self {
        Type::Base(BaseType::mk_vector(elem_ty))
    }

    pub fn mk_struct(inst: StructInstantiation) -> Self {
        Type::Base(BaseType::mk_struct(inst))
    }

    pub fn mk_parameter(index: TypeParameterIndex) -> Self {
        Type::Base(BaseType::mk_parameter(index))
    }

    pub fn mk_binding(symbol: String) -> Self {
        Type::Base(BaseType::mk_binding(symbol))
    }

    //
    // helpers
    //

    pub fn is_base(&self) -> bool {
        matches!(self, Type::Base(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Type::Reference(_, _))
    }

    pub fn is_imm_ref(&self) -> bool {
        matches!(self, Type::Reference(false, _))
    }

    pub fn is_mut_ref(&self) -> bool {
        matches!(self, Type::Reference(true, _))
    }

    pub fn get_base_type(&self) -> Option<&BaseType> {
        match self {
            Type::Base(base_ty) => Some(base_ty),
            _ => None,
        }
    }

    pub fn get_ref_type(&self) -> Option<(bool, &BaseType)> {
        match self {
            Type::Reference(is_mut, base_ty) => Some((*is_mut, base_ty)),
            _ => None,
        }
    }

    pub fn is_bool(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_bool())
    }

    pub fn is_u8(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_u8())
    }

    pub fn is_u64(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_u64())
    }

    pub fn is_u128(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_u128())
    }

    pub fn is_num(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_num())
    }

    pub fn is_int(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_int())
    }

    pub fn is_address(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_address())
    }

    pub fn is_signer(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_signer())
    }

    pub fn is_vector(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_vector())
    }

    pub fn is_struct(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_struct())
    }

    pub fn is_parameter(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_parameter())
    }

    pub fn is_binding(&self) -> bool {
        self.get_base_type().map_or(false, |ty| ty.is_binding())
    }

    pub fn get_vector_element_type(&self) -> Option<&BaseType> {
        self.get_base_type()
            .and_then(|ty| ty.get_vector_element_type())
    }

    pub fn get_struct_instantiation(&self) -> Option<&StructInstantiation> {
        self.get_base_type()
            .and_then(|ty| ty.get_struct_instantiation())
    }

    pub fn get_parameter_index(&self) -> Option<TypeParameterIndex> {
        self.get_base_type().and_then(|ty| ty.get_parameter_index())
    }

    pub fn get_binding_symbol(&self) -> Option<&str> {
        self.get_base_type().and_then(|ty| ty.get_binding_symbol())
    }

    pub fn is_base_of(&self, ty: &BaseType) -> bool {
        self.get_base_type().map_or(false, |self_ty| self_ty == ty)
    }

    pub fn is_struct_of(&self, inst: &StructInstantiation) -> bool {
        self.get_struct_instantiation()
            .map_or(false, |self_inst| self_inst == inst)
    }

    pub fn is_ref_of(&self, ty: &BaseType, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty == ty && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_bool(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_bool() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_u8(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_u8() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_u64(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_u64() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_u128(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_u64() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_num(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_num() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_address(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_address() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_signer(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_signer() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_vector(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_vector() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_struct(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_struct() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_parameter(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_parameter() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_of_binding(&self, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_binding() && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_vector_of(&self, elem_ty: &BaseType, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_vector_of(elem_ty) && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
    }

    pub fn is_ref_struct_of(&self, inst: &StructInstantiation, is_mut_opt: Option<bool>) -> bool {
        self.get_ref_type().map_or(false, |(self_is_mut, self_ty)| {
            self_ty.is_struct_of(inst) && is_mut_opt.map_or(true, |is_mut| is_mut == self_is_mut)
        })
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
