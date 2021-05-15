// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use num::{BigUint, ToPrimitive};
use std::collections::BTreeMap;

use move_core_types::{
    account_address::AccountAddress,
    effects::ChangeSet,
    value::{MoveStruct, MoveValue},
};
use move_model::ast::{MemoryLabel, TempIndex};

use crate::{
    concrete::ty::{BaseType, PartialStructInstantiation, StructInstantiation, Type},
    shared::ident::StructIdent,
};

//**************************************************************************************************
// Value core
//**************************************************************************************************

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum BaseValue {
    Bool(bool),
    Int(BigUint),
    Address(AccountAddress),
    Signer(AccountAddress),
    Vector(Vec<BaseValue>),
    Struct(Vec<BaseValue>),
}

impl BaseValue {
    pub fn mk_bool(v: bool) -> Self {
        Self::Bool(v)
    }
    pub fn mk_u8(v: u8) -> Self {
        Self::Int(BigUint::from(v))
    }
    pub fn mk_u64(v: u64) -> Self {
        Self::Int(BigUint::from(v))
    }
    pub fn mk_u128(v: u128) -> Self {
        Self::Int(BigUint::from(v))
    }
    pub fn mk_num(v: BigUint) -> Self {
        Self::Int(v)
    }
    pub fn mk_address(v: AccountAddress) -> Self {
        Self::Address(v)
    }
    pub fn mk_signer(v: AccountAddress) -> Self {
        Self::Signer(v)
    }
    pub fn mk_vector(v: Vec<BaseValue>) -> Self {
        Self::Vector(v)
    }
    pub fn mk_struct(v: Vec<BaseValue>) -> Self {
        Self::Struct(v)
    }

    pub fn into_bool(self) -> bool {
        match self {
            Self::Bool(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_u8(self) -> u8 {
        match self {
            Self::Int(v) => v.to_u8().unwrap(),
            _ => unreachable!(),
        }
    }
    pub fn into_u64(self) -> u64 {
        match self {
            Self::Int(v) => v.to_u64().unwrap(),
            _ => unreachable!(),
        }
    }
    pub fn into_u128(self) -> u128 {
        match self {
            Self::Int(v) => v.to_u128().unwrap(),
            _ => unreachable!(),
        }
    }
    pub fn into_num(self) -> BigUint {
        match self {
            Self::Int(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_int(self) -> BigUint {
        match self {
            Self::Int(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_address(self) -> AccountAddress {
        match self {
            Self::Address(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_signer(self) -> AccountAddress {
        match self {
            Self::Signer(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_vector(self) -> Vec<BaseValue> {
        match self {
            Self::Vector(v) => v,
            _ => unreachable!(),
        }
    }
    pub fn into_struct(self) -> Vec<BaseValue> {
        match self {
            Self::Struct(v) => v,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Pointer {
    None,
    Global(AccountAddress),
    Local(TempIndex),
    RefWhole(TempIndex),
    RefField(TempIndex, usize),
    RefElement(TempIndex, usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TypedValue {
    ty: Type,
    val: BaseValue,
    ptr: Pointer,
}

#[allow(dead_code)]
impl TypedValue {
    //
    // value creation
    //

    pub fn mk_bool(v: bool) -> Self {
        Self {
            ty: Type::mk_bool(),
            val: BaseValue::mk_bool(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_u8(v: u8) -> Self {
        Self {
            ty: Type::mk_u8(),
            val: BaseValue::mk_u8(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_u64(v: u64) -> Self {
        Self {
            ty: Type::mk_u64(),
            val: BaseValue::mk_u64(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_u128(v: u128) -> Self {
        Self {
            ty: Type::mk_u128(),
            val: BaseValue::mk_u128(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_num(v: BigUint) -> Self {
        Self {
            ty: Type::mk_num(),
            val: BaseValue::mk_num(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_address(v: AccountAddress) -> Self {
        Self {
            ty: Type::mk_address(),
            val: BaseValue::mk_address(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_signer(v: AccountAddress) -> Self {
        Self {
            ty: Type::mk_signer(),
            val: BaseValue::mk_signer(v),
            ptr: Pointer::None,
        }
    }
    pub fn mk_vector(elem: BaseType, v: Vec<TypedValue>) -> Self {
        if cfg!(debug_assertions) {
            for e in &v {
                assert_eq!(e.ty.get_base_type(), &elem);
            }
        }
        Self {
            ty: Type::mk_vector(elem),
            val: BaseValue::mk_vector(v.into_iter().map(|e| e.val).collect()),
            ptr: Pointer::None,
        }
    }
    pub fn mk_struct(inst: StructInstantiation, v: Vec<TypedValue>) -> Self {
        if cfg!(debug_assertions) {
            assert_eq!(inst.fields.len(), v.len());
            for (e, field) in v.iter().zip(inst.fields.iter()) {
                assert_eq!(e.ty.get_base_type(), &field.ty);
            }
        }
        Self {
            ty: Type::mk_struct(inst),
            val: BaseValue::mk_struct(v.into_iter().map(|e| e.val).collect()),
            ptr: Pointer::None,
        }
    }

    pub fn mk_ref_bool(v: bool, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_bool(is_mut),
            val: BaseValue::mk_bool(v),
            ptr,
        }
    }
    pub fn mk_ref_u8(v: u8, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_u8(is_mut),
            val: BaseValue::mk_u8(v),
            ptr,
        }
    }
    pub fn mk_ref_u64(v: u64, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_u64(is_mut),
            val: BaseValue::mk_u64(v),
            ptr,
        }
    }
    pub fn mk_ref_u128(v: u128, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_u128(is_mut),
            val: BaseValue::mk_u128(v),
            ptr,
        }
    }
    pub fn mk_ref_num(v: BigUint, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_num(is_mut),
            val: BaseValue::mk_num(v),
            ptr,
        }
    }
    pub fn mk_ref_address(v: AccountAddress, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_address(is_mut),
            val: BaseValue::mk_address(v),
            ptr,
        }
    }
    pub fn mk_ref_signer(v: AccountAddress, is_mut: bool, ptr: Pointer) -> Self {
        Self {
            ty: Type::mk_ref_signer(is_mut),
            val: BaseValue::mk_signer(v),
            ptr,
        }
    }
    pub fn mk_ref_vector(elem: BaseType, v: Vec<TypedValue>, is_mut: bool, ptr: Pointer) -> Self {
        if cfg!(debug_assertions) {
            for e in &v {
                assert_eq!(e.ty.get_base_type(), &elem);
            }
        }
        Self {
            ty: Type::mk_ref_vector(elem, is_mut),
            val: BaseValue::mk_vector(v.into_iter().map(|e| e.val).collect()),
            ptr,
        }
    }
    pub fn mk_ref_struct(
        inst: StructInstantiation,
        v: Vec<TypedValue>,
        is_mut: bool,
        ptr: Pointer,
    ) -> Self {
        if cfg!(debug_assertions) {
            assert_eq!(inst.fields.len(), v.len());
            for (e, field) in v.iter().zip(inst.fields.iter()) {
                assert_eq!(e.ty.get_base_type(), &field.ty);
            }
        }
        Self {
            ty: Type::mk_ref_struct(inst, is_mut),
            val: BaseValue::mk_struct(v.into_iter().map(|e| e.val).collect()),
            ptr,
        }
    }

    //
    // value casting
    //

    pub fn into_bool(self) -> bool {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_bool());
        }
        self.val.into_bool()
    }
    pub fn into_u8(self) -> u8 {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_u8());
        }
        self.val.into_u8()
    }
    pub fn into_u64(self) -> u64 {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_u64());
        }
        self.val.into_u64()
    }
    pub fn into_u128(self) -> u128 {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_u128());
        }
        self.val.into_u128()
    }
    pub fn into_num(self) -> BigUint {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_num());
        }
        self.val.into_num()
    }
    pub fn into_int(self) -> BigUint {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_int());
        }
        self.val.into_int()
    }
    pub fn into_address(self) -> AccountAddress {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_address());
        }
        self.val.into_address()
    }
    pub fn into_signer(self) -> AccountAddress {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_signer());
        }
        self.val.into_signer()
    }
    pub fn into_vector(self) -> Vec<BaseValue> {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_vector());
        }
        self.val.into_vector()
    }
    pub fn into_struct(self) -> Vec<BaseValue> {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_struct());
        }
        self.val.into_struct()
    }

    pub fn into_ref_bool(self) -> (bool, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_bool(None));
        }
        (self.val.into_bool(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_u8(self) -> (u8, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_u8(None));
        }
        (self.val.into_u8(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_u64(self) -> (u64, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_u64(None));
        }
        (self.val.into_u64(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_u128(self) -> (u128, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_u128(None));
        }
        (self.val.into_u128(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_num(self) -> (BigUint, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_num(None));
        }
        (self.val.into_num(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_address(self) -> (AccountAddress, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_address(None));
        }
        (self.val.into_address(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_signer(self) -> (AccountAddress, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_signer(None));
        }
        (self.val.into_signer(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_vector(self) -> (Vec<BaseValue>, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_vector(None));
        }
        (self.val.into_vector(), self.ty.into_ref_type().0, self.ptr)
    }
    pub fn into_ref_struct(self) -> (Vec<BaseValue>, bool, Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.ty.is_ref_struct(None));
        }
        (self.val.into_struct(), self.ty.into_ref_type().0, self.ptr)
    }

    //
    // Getters
    //

    pub fn get_ty(&self) -> &Type {
        &self.ty
    }
    pub fn get_val(&self) -> &BaseValue {
        &self.val
    }
    pub fn get_ptr(&self) -> &Pointer {
        &self.ptr
    }
    pub fn decompose(self) -> (Type, BaseValue, Pointer) {
        (self.ty, self.val, self.ptr)
    }

    //
    // Operations
    //

    /// Cast this value into a compatible type. Cast `ty` must be compatible
    pub fn assign_cast(self, ty: Type) -> TypedValue {
        if cfg!(debug_assertions) {
            assert!(ty.is_compatible_for_assign(&self.ty));
        }
        TypedValue {
            ty,
            val: self.val,
            ptr: self.ptr,
        }
    }

    /// Create a reference to the base value
    pub fn borrow_local(self, is_mut: bool, local_idx: TempIndex) -> TypedValue {
        TypedValue {
            ty: Type::Reference(is_mut, self.ty.into_base_type()),
            val: self.val,
            ptr: Pointer::Local(local_idx),
        }
    }

    /// Read the reference and create a base value
    pub fn read_ref(self) -> TypedValue {
        TypedValue {
            ty: Type::Base(self.ty.into_ref_type().1),
            val: self.val,
            ptr: Pointer::None,
        }
    }

    /// Create a mutable reference to this base value
    pub fn write_ref(self, ptr: Pointer) -> TypedValue {
        TypedValue {
            ty: Type::Reference(true, self.ty.into_base_type()),
            val: self.val,
            ptr,
        }
    }

    /// Convert the mutable reference into immutable
    pub fn freeze_ref(self) -> TypedValue {
        let (is_mut, base_ty) = self.ty.into_ref_type();
        if cfg!(debug_assertions) {
            assert!(is_mut);
        }
        TypedValue {
            ty: Type::Reference(false, base_ty),
            val: self.val,
            ptr: self.ptr,
        }
    }

    pub fn get_vector_element(self, elem_num: usize) -> Option<TypedValue> {
        let elem_ty = self.ty.into_vector_elem();
        let val = match self.val {
            BaseValue::Vector(mut v) => {
                if elem_num >= v.len() {
                    return None;
                }
                v.remove(elem_num)
            }
            _ => unreachable!(),
        };
        Some(TypedValue {
            ty: Type::Base(elem_ty),
            val,
            ptr: Pointer::None,
        })
    }

    pub fn borrow_ref_vector_element(
        self,
        elem_num: usize,
        is_mut: bool,
        local_idx: TempIndex,
    ) -> Option<TypedValue> {
        if cfg!(debug_assertions) {
            let (self_is_mut, _) = self.ty.get_ref_type();
            assert!(self_is_mut || !is_mut);
        }
        let elem_ty = self.ty.into_ref_vector_elem(None);
        let val = match self.val {
            BaseValue::Vector(mut v) => {
                if elem_num >= v.len() {
                    return None;
                }
                v.remove(elem_num)
            }
            _ => unreachable!(),
        };
        Some(TypedValue {
            ty: Type::Reference(is_mut, elem_ty),
            val,
            ptr: Pointer::RefElement(local_idx, elem_num),
        })
    }

    pub fn update_ref_vector_push_back(self, elem_val: TypedValue) -> TypedValue {
        let (elem_ty, elem_val, _) = elem_val.decompose();
        let (vec_ty, vec_val, vec_ptr) = self.decompose();
        let elem = vec_ty.into_ref_vector_elem(Some(true));
        if cfg!(debug_assertions) {
            assert!(elem_ty.is_base_of(&elem));
        }
        let mut elems = vec_val.into_vector();
        elems.push(elem_val);
        TypedValue {
            ty: Type::mk_ref_vector(elem, true),
            val: BaseValue::mk_vector(elems),
            ptr: vec_ptr,
        }
    }

    pub fn update_ref_vector_pop_back(self) -> Option<(TypedValue, TypedValue)> {
        let (vec_ty, vec_val, vec_ptr) = self.decompose();
        let elem_ty = vec_ty.into_ref_vector_elem(Some(true));
        let mut elems = vec_val.into_vector();
        match elems.pop() {
            None => None,
            Some(elem_val) => {
                let elem = TypedValue {
                    ty: Type::Base(elem_ty.clone()),
                    val: elem_val,
                    ptr: Pointer::None,
                };
                let new_vec = TypedValue {
                    ty: Type::mk_ref_vector(elem_ty, true),
                    val: BaseValue::mk_vector(elems),
                    ptr: vec_ptr,
                };
                Some((new_vec, elem))
            }
        }
    }

    pub fn update_ref_vector_swap(self, lhs: usize, rhs: usize) -> Option<TypedValue> {
        let (vec_ty, vec_val, vec_ptr) = self.decompose();
        if cfg!(debug_assertions) {
            assert!(vec_ty.is_ref_vector(Some(true)));
        }
        let mut elems = vec_val.into_vector();
        if lhs >= elems.len() || rhs >= elems.len() {
            return None;
        }
        elems.swap(lhs, rhs);
        let new_vec = TypedValue {
            ty: vec_ty,
            val: BaseValue::mk_vector(elems),
            ptr: vec_ptr,
        };
        Some(new_vec)
    }

    pub fn update_ref_vector_element(self, elem_val: TypedValue) -> TypedValue {
        let (elem_ty, elem_val, elem_ptr) = elem_val.decompose();
        let (vec_ty, vec_val, vec_ptr) = self.decompose();
        let elem = vec_ty.into_ref_vector_elem(Some(true));
        if cfg!(debug_assertions) {
            assert!(elem_ty.is_ref_of(&elem, Some(true)));
        }
        let elem_num = match elem_ptr {
            Pointer::RefElement(_, elem_num) => elem_num,
            _ => unreachable!(),
        };
        let mut elems = vec_val.into_vector();
        *elems.get_mut(elem_num).unwrap() = elem_val;
        TypedValue {
            ty: Type::mk_ref_vector(elem, true),
            val: BaseValue::mk_vector(elems),
            ptr: vec_ptr,
        }
    }

    pub fn unpack_struct(self) -> Vec<TypedValue> {
        let fields = self.ty.into_struct_inst().fields;
        match self.val {
            BaseValue::Struct(v) => v
                .into_iter()
                .zip(fields)
                .map(|(field_val, field_info)| TypedValue {
                    ty: Type::Base(field_info.ty),
                    val: field_val,
                    ptr: Pointer::None,
                })
                .collect(),
            _ => unreachable!(),
        }
    }

    pub fn unpack_struct_field(self, field_num: usize) -> TypedValue {
        let field = self.ty.into_struct_inst().fields.remove(field_num);
        let val = match self.val {
            BaseValue::Struct(mut v) => v.remove(field_num),
            _ => unreachable!(),
        };
        TypedValue {
            ty: Type::Base(field.ty),
            val,
            ptr: Pointer::None,
        }
    }

    pub fn unpack_ref_struct_field(self, field_num: usize, is_mut_opt: Option<bool>) -> TypedValue {
        let field = self
            .ty
            .into_ref_struct_inst(is_mut_opt)
            .fields
            .remove(field_num);
        let val = match self.val {
            BaseValue::Struct(mut v) => v.remove(field_num),
            _ => unreachable!(),
        };
        TypedValue {
            ty: Type::Base(field.ty),
            val,
            ptr: Pointer::None,
        }
    }

    pub fn borrow_ref_struct_field(
        self,
        field_num: usize,
        is_mut: bool,
        local_idx: TempIndex,
    ) -> TypedValue {
        if cfg!(debug_assertions) {
            let (self_is_mut, _) = self.ty.get_ref_type();
            assert!(self_is_mut || !is_mut);
        }
        let field = self.ty.into_ref_struct_inst(None).fields.remove(field_num);
        let val = match self.val {
            BaseValue::Struct(mut v) => v.remove(field_num),
            _ => unreachable!(),
        };
        TypedValue {
            ty: Type::Reference(is_mut, field.ty),
            val,
            ptr: Pointer::RefField(local_idx, field_num),
        }
    }

    pub fn update_ref_struct_field(self, field_num: usize, field_val: TypedValue) -> TypedValue {
        let (field_ty, field_val, field_ptr) = field_val.decompose();
        let (struct_ty, struct_val, struct_ptr) = self.decompose();
        let inst = struct_ty.into_ref_struct_inst(Some(true));
        if cfg!(debug_assertions) {
            assert!(matches!(field_ptr, Pointer::RefField(_, ref_field) if ref_field == field_num));
            assert!(field_ty.is_ref_of(&inst.fields.get(field_num).unwrap().ty, Some(true)));
        }
        let mut fields = struct_val.into_struct();
        *fields.get_mut(field_num).unwrap() = field_val;
        TypedValue {
            ty: Type::mk_ref_struct(inst, true),
            val: BaseValue::mk_struct(fields),
            ptr: struct_ptr,
        }
    }

    fn into_move_value(self) -> MoveValue {
        match self.val {
            BaseValue::Bool(v) => MoveValue::Bool(v),
            BaseValue::Int(v) => {
                if self.ty.is_u8() {
                    MoveValue::U8(v.to_u8().unwrap())
                } else if self.ty.is_u64() {
                    MoveValue::U64(v.to_u64().unwrap())
                } else {
                    if cfg!(debug_assertions) {
                        assert!(self.ty.is_u128());
                    }
                    MoveValue::U128(v.to_u128().unwrap())
                }
            }
            BaseValue::Address(v) => MoveValue::Address(v),
            BaseValue::Signer(v) => MoveValue::Signer(v),
            BaseValue::Vector(v) => {
                let elem_ty = self.ty.into_vector_elem();
                let move_elems = v
                    .into_iter()
                    .map(|elem| {
                        let full_elem = TypedValue {
                            ty: Type::Base(elem_ty.clone()),
                            val: elem,
                            ptr: Pointer::None,
                        };
                        full_elem.into_move_value()
                    })
                    .collect();
                MoveValue::Vector(move_elems)
            }
            BaseValue::Struct(v) => {
                let move_fields = v
                    .into_iter()
                    .zip(self.ty.into_struct_inst().fields)
                    .map(|(field_val, field_info)| {
                        let full_field = TypedValue {
                            ty: Type::Base(field_info.ty),
                            val: field_val,
                            ptr: Pointer::None,
                        };
                        full_field.into_move_value()
                    })
                    .collect();
                MoveValue::Struct(MoveStruct::new(move_fields))
            }
        }
    }

    /// Into BCS-serialized bytes
    pub fn into_bcs_bytes(self) -> Option<Vec<u8>> {
        self.into_move_value().simple_serialize()
    }
}

//**************************************************************************************************
// Local state
//**************************************************************************************************

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LocalSlot {
    name: String,
    ty: Type,
    is_arg: bool,
    content: Option<(BaseValue, Pointer)>,
}

impl LocalSlot {
    /// Create a local slot that holds a function argument
    pub fn new_arg(name: String, val: TypedValue) -> Self {
        let (ty, val, ptr) = val.decompose();
        LocalSlot {
            name,
            ty,
            is_arg: true,
            content: Some((val, ptr)),
        }
    }
    /// Create a local slot that holds a function temporary
    pub fn new_tmp(name: String, ty: Type) -> Self {
        LocalSlot {
            name,
            ty,
            is_arg: false,
            content: None,
        }
    }

    /// Get the type of this local slot
    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    /// Check whether this local slot holds a value
    pub fn has_value(&self) -> bool {
        self.content.is_some()
    }
    /// Get the value held in this local slot. Panics if the slot does not hold a value
    pub fn get_value(&self) -> TypedValue {
        let (val, ptr) = self.content.as_ref().unwrap();
        TypedValue {
            ty: self.ty.clone(),
            val: val.clone(),
            ptr: ptr.clone(),
        }
    }
    /// Put the value held in this local slot. Override if the slot already holds a value
    pub fn put_value_override(&mut self, val: TypedValue) {
        let (ty, val, ptr) = val.decompose();
        if cfg!(debug_assertions) {
            assert_eq!(ty, self.ty);
        }
        self.content = Some((val, ptr));
    }
    /// Put the value held in this local slot. Panics if the slot already holds a value
    pub fn put_value(&mut self, val: TypedValue) {
        if cfg!(debug_assertions) {
            assert!(self.content.is_none());
        }
        self.put_value_override(val);
    }
    /// Delete the value held in this local slot. Panics if the slot does not hold a value
    pub fn del_value(&mut self) -> TypedValue {
        let (val, ptr) = self.content.take().unwrap();
        TypedValue {
            ty: self.ty.clone(),
            val,
            ptr,
        }
    }
}

//**************************************************************************************************
// Global state
//**************************************************************************************************

#[derive(Debug, Clone, Default, Eq, PartialEq)]
struct AccountState {
    storage: BTreeMap<StructInstantiation, BaseValue>,
}

impl AccountState {
    /// Get a resource from the address, return None of the resource does not exist
    fn get_resource(&self, key: &StructInstantiation) -> Option<BaseValue> {
        self.storage.get(key).cloned()
    }

    /// Remove a resource from the address, return the old resource if exists
    fn del_resource(&mut self, key: &StructInstantiation) -> Option<BaseValue> {
        self.storage.remove(key)
    }

    /// Put a resource into the address, return the old resource if exists
    fn put_resource(&mut self, key: &StructInstantiation, object: BaseValue) -> Option<BaseValue> {
        self.storage.insert(key.clone(), object)
    }

    /// Check whether the address has a resource
    fn has_resource(&self, key: &StructInstantiation) -> bool {
        self.storage.contains_key(key)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct GlobalState {
    accounts: BTreeMap<AccountAddress, AccountState>,
    events: BTreeMap<Vec<u8>, BTreeMap<u64, TypedValue>>,
}

impl GlobalState {
    /// Get a reference to a resource from the address, return None of the resource does not exist
    pub fn get_resource(
        &self,
        is_mut_opt: Option<bool>,
        addr: AccountAddress,
        key: StructInstantiation,
    ) -> Option<TypedValue> {
        self.accounts.get(&addr).and_then(|account| {
            account.get_resource(&key).map(|val| {
                let ty = match is_mut_opt {
                    None => Type::mk_struct(key),
                    Some(is_mut) => Type::mk_ref_struct(key, is_mut),
                };
                TypedValue {
                    ty,
                    val,
                    ptr: Pointer::Global(addr),
                }
            })
        })
    }

    /// Remove a resource from the address, return the old resource (as struct) if exists
    pub fn del_resource(
        &mut self,
        addr: AccountAddress,
        key: StructInstantiation,
    ) -> Option<TypedValue> {
        self.accounts.get_mut(&addr).and_then(|account| {
            account.del_resource(&key).map(|val| TypedValue {
                ty: Type::mk_struct(key),
                val,
                ptr: Pointer::None,
            })
        })
    }

    /// Put a resource into the address, return the old resource (as struct) if exists
    pub fn put_resource(
        &mut self,
        addr: AccountAddress,
        key: StructInstantiation,
        object: TypedValue,
    ) -> Option<TypedValue> {
        if cfg!(debug_assertions) {
            assert_eq!(key, object.ty.into_struct_inst());
        }
        self.accounts
            .entry(addr)
            .or_insert_with(AccountState::default)
            .put_resource(&key, object.val)
            .map(|val| TypedValue {
                ty: Type::mk_struct(key),
                val,
                ptr: Pointer::None,
            })
    }

    /// Check whether the address has a resource
    pub fn has_resource(&self, addr: &AccountAddress, key: &StructInstantiation) -> bool {
        self.accounts
            .get(addr)
            .map_or(false, |account| account.has_resource(key))
    }

    /// Emit an event to the event store
    pub fn emit_event(&mut self, guid: Vec<u8>, seq: u64, msg: TypedValue) {
        let res = self
            .events
            .entry(guid)
            .or_insert_with(BTreeMap::new)
            .insert(seq, msg);
        if cfg!(debug_assertions) {
            assert!(res.is_none());
        }
    }

    /// Calculate the delta (i.e., a ChangeSet) against the old state
    pub fn delta(&self, old_state: &GlobalState) -> ChangeSet {
        fn bcs_serialize_resource(key: &StructInstantiation, val: &BaseValue) -> Vec<u8> {
            let typed_val = TypedValue {
                ty: Type::mk_struct(key.clone()),
                val: val.clone(),
                ptr: Pointer::None,
            };
            typed_val.into_bcs_bytes().unwrap()
        }

        let mut change_set = ChangeSet::new();
        let empty_account_state = AccountState::default();

        // collect added / modified resources
        for (addr, account_state) in &self.accounts {
            let old_account_state = old_state.accounts.get(addr).unwrap_or(&empty_account_state);
            for (key, val) in &account_state.storage {
                match old_account_state.storage.get(key) {
                    None => change_set
                        .publish_resource(
                            *addr,
                            key.to_move_struct_tag(),
                            bcs_serialize_resource(key, val),
                        )
                        .unwrap(),
                    Some(old_val) => {
                        if val != old_val {
                            change_set.publish_or_overwrite_resource(
                                *addr,
                                key.to_move_struct_tag(),
                                bcs_serialize_resource(key, val),
                            );
                        }
                    }
                }
            }
        }

        // collect deleted resources
        for (old_addr, old_account_state) in &old_state.accounts {
            let account_state = self.accounts.get(old_addr).unwrap_or(&empty_account_state);
            for old_key in old_account_state.storage.keys() {
                if !account_state.storage.contains_key(old_key) {
                    change_set
                        .unpublish_resource(*old_addr, old_key.to_move_struct_tag())
                        .unwrap();
                }
            }
        }

        change_set
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct EvalState {
    // global resources specifically marked as saved
    saved_memory: BTreeMap<
        MemoryLabel,
        BTreeMap<StructIdent, BTreeMap<StructInstantiation, BTreeMap<AccountAddress, BaseValue>>>,
    >,
}

impl EvalState {
    pub fn save_memory(
        &mut self,
        label: MemoryLabel,
        partial_inst: PartialStructInstantiation,
        global_state: &GlobalState,
    ) {
        let mut per_struct_map = BTreeMap::new();
        for (addr, state) in &global_state.accounts {
            for (inst, val) in &state.storage {
                if inst.ident == partial_inst.ident {
                    per_struct_map
                        .entry(inst.clone())
                        .or_insert_with(BTreeMap::new)
                        .insert(*addr, val.clone());
                }
            }
        }
        self.saved_memory
            .entry(label)
            .and_modify(|per_label_map| per_label_map.clear())
            .or_insert_with(BTreeMap::new)
            .insert(partial_inst.ident, per_struct_map);
    }
}
