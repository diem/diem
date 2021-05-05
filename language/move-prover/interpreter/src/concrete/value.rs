// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// TODO (mengxu) remove this when the module is in good shape
#![allow(dead_code)]

use num::{BigUint, ToPrimitive};
use std::collections::BTreeMap;

use move_core_types::account_address::AccountAddress;
use move_model::ast::TempIndex;

use crate::concrete::ty::{BaseType, StructInstantiation, Type};

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

    pub fn to_ref(&self) -> RefTypedValue {
        RefTypedValue {
            ty: &self.ty,
            val: &self.val,
            ptr: &self.ptr,
        }
    }

    pub fn decompose(self) -> (Type, BaseValue, Pointer) {
        (self.ty, self.val, self.ptr)
    }

    //
    // Operations
    //

    pub fn borrow_local(self, is_mut: bool, local_idx: TempIndex) -> TypedValue {
        TypedValue {
            ty: Type::Reference(is_mut, self.ty.into_base_type()),
            val: self.val,
            ptr: Pointer::Local(local_idx),
        }
    }

    pub fn read_ref(self) -> TypedValue {
        TypedValue {
            ty: Type::Base(self.ty.into_ref_type().1),
            val: self.val,
            ptr: Pointer::None,
        }
    }

    pub fn write_ref(self, ptr: Pointer) -> TypedValue {
        TypedValue {
            ty: Type::Reference(true, self.ty.into_base_type()),
            val: self.val,
            ptr,
        }
    }

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
        elems.insert(elem_num, elem_val);
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
        fields.insert(field_num, field_val);
        TypedValue {
            ty: Type::mk_ref_struct(inst, true),
            val: BaseValue::mk_struct(fields),
            ptr: struct_ptr,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefTypedValue<'a> {
    ty: &'a Type,
    val: &'a BaseValue,
    ptr: &'a Pointer,
}

impl<'a> RefTypedValue<'a> {
    pub fn get_ty(&self) -> &'a Type {
        self.ty
    }
    pub fn get_val(&self) -> &'a BaseValue {
        self.val
    }
    pub fn get_ptr(&self) -> &'a Pointer {
        self.ptr
    }

    pub fn deref(&self) -> TypedValue {
        TypedValue {
            ty: self.ty.clone(),
            val: self.val.clone(),
            ptr: self.ptr.clone(),
        }
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
    pub fn new_arg(name: String, val: TypedValue) -> Self {
        let (ty, val, ptr) = val.decompose();
        LocalSlot {
            name,
            ty,
            is_arg: true,
            content: Some((val, ptr)),
        }
    }
    pub fn new_tmp(name: String, ty: Type) -> Self {
        LocalSlot {
            name,
            ty,
            is_arg: false,
            content: None,
        }
    }

    pub fn get_type(&self) -> &Type {
        &self.ty
    }

    pub fn get_value(&self) -> RefTypedValue {
        let (val, ptr) = self.content.as_ref().unwrap();
        RefTypedValue {
            ty: &self.ty,
            val,
            ptr,
        }
    }
    pub fn put_value(&mut self, val: BaseValue, ptr: Pointer) {
        if cfg!(debug_assertions) {
            assert!(self.content.is_none());
        }
        self.content = Some((val, ptr));
    }
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
    fn get_resource(&self, key: &StructInstantiation) -> Option<BaseValue> {
        self.storage.get(key).cloned()
    }

    fn del_resource(&mut self, key: &StructInstantiation) -> Option<BaseValue> {
        self.storage.remove(key)
    }

    fn put_resource(&mut self, key: StructInstantiation, object: BaseValue) -> Option<BaseValue> {
        self.storage.insert(key, object)
    }

    fn has_resource(&self, key: &StructInstantiation) -> bool {
        self.storage.contains_key(key)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct GlobalState {
    accounts: BTreeMap<AccountAddress, AccountState>,
}

impl GlobalState {
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

    pub fn put_resource(
        &mut self,
        addr: AccountAddress,
        key: StructInstantiation,
        object: BaseValue,
    ) -> bool {
        self.accounts
            .entry(addr)
            .or_insert_with(AccountState::default)
            .put_resource(key, object)
            .is_none()
    }

    pub fn has_resource(&self, addr: &AccountAddress, key: &StructInstantiation) -> bool {
        self.accounts
            .get(addr)
            .map_or(false, |account| account.has_resource(key))
    }
}
